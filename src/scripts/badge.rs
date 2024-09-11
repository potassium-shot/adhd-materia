use std::{
	cell::Cell,
	collections::HashMap,
	rc::{Rc, Weak},
	sync::LazyLock,
};

use crate::data_dir::{DataDir, DataDirError};

use super::list::{ScriptEditor, ScriptList};

static BADGES_COLOR_HASH: LazyLock<colorhash::ColorHash> =
	LazyLock::new(|| colorhash::ColorHash::new());

pub struct Badge<'list, 'pressed> {
	pressed: &'pressed mut bool,
	script_name: &'list str,
}

impl<'list, 'pressed> Badge<'list, 'pressed> {
	pub fn new(script_name: &'list str, pressed: &'pressed mut bool) -> Self {
		Self {
			script_name,
			pressed,
		}
	}
}

impl egui::Widget for Badge<'_, '_> {
	fn ui(self, ui: &mut egui::Ui) -> egui::Response {
		ui.add_visible_ui(true, |ui| {
			if *self.pressed {
				let col = BADGES_COLOR_HASH.rgb(self.script_name);

				let mut new_stroke = ui.visuals_mut().widgets.hovered.bg_stroke;
				new_stroke.color =
					egui::Color32::from_rgb(col.red() as u8, col.green() as u8, col.blue() as u8);
				new_stroke.width *= 2.0;
				ui.visuals_mut().widgets.hovered.bg_stroke = new_stroke;
				ui.visuals_mut().widgets.inactive.bg_stroke = new_stroke;
			}

			let response = ui.button(egui::RichText::new(self.script_name).size(16.0));

			if response.clicked() {
				*self.pressed = !*self.pressed;
			}

			response
		})
		.inner
	}
}

pub trait BadgeType {
	fn get_path_from_data_dir(data_dir: &'static DataDir) -> &'static std::path::Path;

	fn get_path() -> Result<&'static std::path::Path, &'static DataDirError> {
		Ok(Self::get_path_from_data_dir(crate::data_dir()?))
	}

	fn allow_multiple() -> bool;
}

pub struct BadgeList<T> {
	list: HashMap<String, Rc<Cell<bool>>>,
	last_set: Cell<Weak<Cell<bool>>>,
	changed: Cell<bool>,
	_t: std::marker::PhantomData<T>,
}

impl<T: BadgeType> BadgeList<T> {
	pub fn new() -> Result<Self, &'static DataDirError> {
		Ok(Self {
			list: ScriptList::new()?
				.0
				.scripts_mut()
				.map(|script: &mut ScriptEditor<T>| {
					(script.script.name.clone(), Rc::new(Cell::new(false)))
				})
				.collect(),
			last_set: Cell::new(Weak::new()),
			changed: Cell::new(true),
			_t: std::marker::PhantomData,
		})
	}

	pub fn iter(&self) -> impl Iterator<Item = (&str, bool)> {
		self.list.iter().map(|(k, v)| (k.as_str(), v.get()))
	}

	pub fn iter_mut(&mut self) -> impl Iterator<Item = (&str, BadgeListEnabledRef<T>)> {
		self.list.iter().map(|(k, v)| {
			(
				k.as_str(),
				BadgeListEnabledRef {
					enabled: v,
					last_set_ref: &self.last_set,
					changed_ref: &self.changed,
					_t: std::marker::PhantomData,
				},
			)
		})
	}

	pub fn check_changed(&mut self) -> bool {
		if self.changed.get() {
			self.changed.set(false);
			true
		} else {
			false
		}
	}
}

pub struct BadgeListEnabledRef<'list, T> {
	enabled: &'list Rc<Cell<bool>>,
	last_set_ref: &'list Cell<Weak<Cell<bool>>>,
	changed_ref: &'list Cell<bool>,
	_t: std::marker::PhantomData<T>,
}

impl<T: BadgeType> BadgeListEnabledRef<'_, T> {
	pub fn get(&self) -> bool {
		self.enabled.get()
	}

	pub fn set(&mut self, value: bool) {
		if value != self.enabled.get() {
			if !T::allow_multiple() && value {
				if let Some(last_set) = self.last_set_ref.take().upgrade() {
					last_set.set(false);
				}

				self.last_set_ref.set(Rc::downgrade(&self.enabled));
			}

			self.enabled.set(value);
			self.changed_ref.set(true);
		}
	}
}

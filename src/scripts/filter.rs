use std::{cell::Cell, collections::HashMap, sync::LazyLock};

use crate::data_dir::DataDirError;

use super::{list::ScriptList, PocketPyScript};

pub static DEFAULT_FILTER_SCRIPT: LazyLock<PocketPyScript> = LazyLock::new(|| PocketPyScript {
	name: "new_filter".to_string(),
	code: "return True".to_string(),
});

static FILTERS_COLOR_HASH: LazyLock<colorhash::ColorHash> =
	LazyLock::new(|| colorhash::ColorHash::new());

pub struct FilterBadge<'list, 'pressed> {
	pressed: &'pressed mut bool,
	filter_script_name: &'list str,
}

impl<'list, 'pressed> FilterBadge<'list, 'pressed> {
	pub fn new(filter_script_name: &'list str, pressed: &'pressed mut bool) -> Self {
		Self {
			filter_script_name,
			pressed,
		}
	}
}

impl egui::Widget for FilterBadge<'_, '_> {
	fn ui(self, ui: &mut egui::Ui) -> egui::Response {
		ui.add_visible_ui(true, |ui| {
			if *self.pressed {
				let col = FILTERS_COLOR_HASH.rgb(self.filter_script_name);

				let mut new_stroke = ui.visuals_mut().widgets.hovered.bg_stroke;
				new_stroke.color =
					egui::Color32::from_rgb(col.red() as u8, col.green() as u8, col.blue() as u8);
				new_stroke.width *= 2.0;
				ui.visuals_mut().widgets.hovered.bg_stroke = new_stroke;
				ui.visuals_mut().widgets.inactive.bg_stroke = new_stroke;
			}

			let response = ui.button(egui::RichText::new(self.filter_script_name).size(16.0));

			if response.clicked() {
				*self.pressed = !*self.pressed;
			}

			response
		})
		.inner
	}
}

pub struct FilterList {
	list: HashMap<String, Cell<bool>>,
	changed: Cell<bool>,
}

impl FilterList {
	pub fn new() -> Result<Self, &'static DataDirError> {
		Ok(Self {
			list: ScriptList::new(crate::data_dir()?.filter_scripts().to_owned())
				.0
				.scripts_mut()
				.map(|script| (script.script.name.clone(), Cell::new(false)))
				.collect(),
			changed: Cell::new(true),
		})
	}

	pub fn iter(&self) -> impl Iterator<Item = (&str, bool)> {
		self.list.iter().map(|(k, v)| (k.as_str(), v.get()))
	}

	pub fn iter_mut(&mut self) -> impl Iterator<Item = (&str, FilterListEnabledRef)> {
		self.list.iter().map(|(k, v)| {
			(
				k.as_str(),
				FilterListEnabledRef {
					enabled: v,
					changed_ref: &self.changed,
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

pub struct FilterListEnabledRef<'list> {
	enabled: &'list Cell<bool>,
	changed_ref: &'list Cell<bool>,
}

impl FilterListEnabledRef<'_> {
	pub fn get(&self) -> bool {
		self.enabled.get()
	}

	pub fn set(&mut self, value: bool) {
		if value != self.enabled.get() {
			self.enabled.set(value);
			self.changed_ref.set(true);
		}
	}
}

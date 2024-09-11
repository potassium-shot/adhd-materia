use std::sync::LazyLock;

use crate::{
	data_dir::{DataDir, DataDirError},
	session::Session,
	toast_error,
};

use super::list::{ScriptEditor, ScriptList};

static BADGES_COLOR_HASH: LazyLock<colorhash::ColorHash> =
	LazyLock::new(|| colorhash::ColorHash::new());

pub struct Badge<'list, 'pressed, T> {
	pressed: &'pressed mut bool,
	script_name: &'list str,
	order: i32,
	_t: std::marker::PhantomData<T>,
}

impl<'list, 'pressed, T: BadgeType> Badge<'list, 'pressed, T> {
	pub fn new(script_name: &'list str, pressed: &'pressed mut bool, order: i32) -> Self {
		Self {
			script_name,
			pressed,
			order,
			_t: std::marker::PhantomData,
		}
	}
}

impl<T: BadgeType> egui::Widget for Badge<'_, '_, T> {
	fn ui(self, ui: &mut egui::Ui) -> egui::Response {
		ui.add_visible_ui(true, |ui| {
			let mut layout_job = egui::text::LayoutJob::default();

			if *self.pressed {
				let col = BADGES_COLOR_HASH.rgb(self.script_name);
				let col =
					egui::Color32::from_rgb(col.red() as u8, col.green() as u8, col.blue() as u8);

				let mut new_stroke = ui.visuals_mut().widgets.hovered.bg_stroke;
				new_stroke.color = col;
				new_stroke.width *= 2.0;
				ui.visuals_mut().widgets.hovered.bg_stroke = new_stroke;
				ui.visuals_mut().widgets.inactive.bg_stroke = new_stroke;

				if T::display_order() {
					layout_job.append(
						format!("{}", self.order).as_str(),
						0.0,
						egui::TextFormat::simple(egui::FontId::proportional(16.0), col),
					);
					layout_job.append(
						"|",
						4.0,
						egui::TextFormat::simple(
							egui::FontId::proportional(16.0),
							ui.visuals().weak_text_color(),
						),
					);
					layout_job.append("", 4.0, egui::TextFormat::default());
				}
			}

			layout_job.append(
				self.script_name,
				0.0,
				egui::TextFormat::simple(
					egui::FontId::proportional(16.0),
					ui.visuals().text_color(),
				),
			);
			let response = ui.button(layout_job);

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

	fn display_order() -> bool;

	fn get_session_badge_list(session: &Session) -> &Vec<String>;
	fn get_session_badge_list_mut(session: &mut Session) -> &mut Vec<String>;
}

pub struct BadgeList<T> {
	unset: Vec<String>,
	set: Vec<String>,
	changed: bool,
	_t: std::marker::PhantomData<T>,
}

impl<T: BadgeType> BadgeList<T> {
	pub fn new() -> Result<Self, &'static DataDirError> {
		let set: Vec<String> = T::get_session_badge_list(&Session::current())
				.iter()
				.filter(|badge| {
					if let Ok(path) = T::get_path() {
						path.join(badge).with_extension("py").exists()
					} else {
						false
					}
				})
				.cloned()
				.collect();

		Ok(Self {
			unset: ScriptList::new()?
				.0
				.scripts_mut()
				.map(|script: &mut ScriptEditor<T>| script.script.name.clone())
				.filter(|name| !set.contains(name))
				.collect(),
			set,
			changed: true,
			_t: std::marker::PhantomData,
		})
	}

	pub fn iter_set(&self) -> impl Iterator<Item = &str> {
		self.set.iter().map(|s| s.as_str())
	}

	pub fn iter_all(&self) -> impl Iterator<Item = (&str, bool)> {
		self.set
			.iter()
			.map(|s| (s.as_str(), true))
			.chain(self.unset.iter().map(|s| (s.as_str(), false)))
	}

	pub fn check_changed(&mut self) -> bool {
		if self.changed {
			self.changed = false;
			true
		} else {
			false
		}
	}

	pub fn swap(&mut self, idx: usize) {
		let idx = if idx >= self.set.len() {
			BadgeListVecIndex::Unset(idx - self.set.len())
		} else {
			BadgeListVecIndex::Set(idx)
		};

		let to_swap = match idx {
			BadgeListVecIndex::Set(idx) => self.set.remove(idx),
			BadgeListVecIndex::Unset(idx) => self.unset.remove(idx),
		};

		match idx {
			BadgeListVecIndex::Set(_) => self.unset.insert(0, to_swap),
			BadgeListVecIndex::Unset(_) => self.set.push(to_swap),
		}

		self.changed = true;

		if let Err(e) = Session::mutate(|session| {
			self.set.clone_into(T::get_session_badge_list_mut(session));
		}) {
			toast_error!("Could not save session: {}", e);
		}
	}
}

enum BadgeListVecIndex {
	Set(usize),
	Unset(usize),
}

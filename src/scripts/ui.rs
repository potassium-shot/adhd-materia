use crate::{settings::Settings, toast_error};

use super::{badge::BadgeType, list::ScriptEditor};

pub struct ScriptWidget<'script, T> {
	pub(super) script: &'script mut ScriptEditor<T>,
}

impl<'script, T: BadgeType> ScriptWidget<'script, T> {
	pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
		let mut switch_to_display = false;
		let mut mark_for_delete = false;

		let response = ui
			.group(|ui| {
				ui.vertical(|ui| {
					let theme = egui_extras::syntax_highlighting::CodeTheme::from_style(ui.style());

					let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
						let mut layout_job = egui_extras::syntax_highlighting::highlight(
							ui.ctx(),
							&theme,
							string,
							"py",
						);
						layout_job.wrap.max_width = wrap_width;
						ui.fonts(|f| f.layout_job(layout_job))
					};

					match &mut self.script.state {
						super::list::ScriptEditorState::EditMode(edited_script) => {
							ui.horizontal_top(|ui| {
								ui.text_edit_singleline(&mut edited_script.name);

								if ui
									.button(
										egui::RichText::from("💾")
											.color(Settings::get().theme.get_catppuccin().lavender),
									)
									.on_hover_ui(|ui| {
										ui.label("Save script");
									})
									.clicked()
								{
									switch_to_display = true;
								}

								if ui
									.button(
										egui::RichText::from("🗑")
											.color(ui.style().visuals.error_fg_color),
									)
									.clicked()
								{
									mark_for_delete = true;
								}
							});

							ui.with_layout(
								egui::Layout::left_to_right(egui::Align::LEFT)
									.with_main_justify(true),
								|ui| {
									ui.add(
										egui::TextEdit::multiline(&mut edited_script.code)
											.code_editor()
											.layouter(&mut layouter),
									);
								},
							);
						}
						super::list::ScriptEditorState::DisplayMode => {
							ui.horizontal_top(|ui| {
								ui.heading(&self.script.script.name);

								if ui
									.button(
										egui::RichText::from("✏")
											.color(ui.style().visuals.warn_fg_color),
									)
									.on_hover_ui(|ui| {
										ui.label("Edit task");
									})
									.clicked()
								{
									self.script.edit();
								}
							});
							ui.separator();
							ui.add_space(8.0);

							ui.with_layout(
								egui::Layout::left_to_right(egui::Align::LEFT)
									.with_main_justify(true),
								|ui| {
									ui.add_enabled(
										false,
										egui::TextEdit::multiline(&mut self.script.script.code)
											.code_editor()
											.layouter(&mut layouter),
									);
								},
							);
						}
					}
				});
			})
			.response;

		if mark_for_delete {
			self.script.delete();
		} else if switch_to_display {
			if let Err(e) = self.script.display() {
				toast_error!("Could not save script: {}", e);
			}
		}

		response
	}
}

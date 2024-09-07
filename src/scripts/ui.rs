use crate::toast_error;

use super::list::ScriptEditor;

pub struct ScriptWidget<'script> {
	pub(super) script: &'script mut ScriptEditor,
}

impl<'script> ScriptWidget<'script> {
	pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
		let mut switch_to_display = false;
		let mut mark_for_delete = false;

		let response = ui
			.group(|ui| {
				ui.vertical(|ui| match &mut self.script.state {
					super::list::ScriptEditorState::EditMode(edited_script) => {
						ui.horizontal_top(|ui| {
							ui.text_edit_singleline(&mut edited_script.name);

							if ui.button("S").clicked() {
								switch_to_display = true;
							}

							if ui.button("D").clicked() {
								mark_for_delete = true;
							}
						});

						ui.text_edit_multiline(&mut edited_script.code);
					}
					super::list::ScriptEditorState::DisplayMode => {
						ui.horizontal_top(|ui| {
							ui.heading(&self.script.script.name);

							if ui.button("E").clicked() {
								self.script.edit();
							}
						});
						ui.separator();
						ui.add_space(8.0);
						ui.add_enabled(
							false,
							egui::TextEdit::multiline(&mut self.script.script.code),
						);
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

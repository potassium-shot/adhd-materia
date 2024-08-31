use super::{Task, TaskState};

pub struct TaskWidget<'task> {
	task: &'task mut Task,
}

impl egui::Widget for TaskWidget<'_> {
	fn ui(self, ui: &mut egui::Ui) -> egui::Response {
		let mut set_pending_delete = false;

		let response = ui
			.group(|ui| match &self.task.state {
				TaskState::Display => {
					ui.vertical(|ui| {
						ui.horizontal(|ui| {
							ui.label(egui::RichText::from(self.task.name.as_str()).heading());

							if ui
								.button(
									egui::RichText::from("✏")
										.color(egui::Color32::from_rgb(0xFF, 0xD0, 0x70)),
								)
								.on_hover_ui(|ui| {
									ui.label("Edit task");
								})
								.clicked()
							{
								self.task.edit();
							}
						});

						if !self.task.description.is_empty() {
							ui.separator();
							ui.label(self.task.description.as_str());
						}
					});
				}
				TaskState::Edit { .. } => {
					ui.vertical(|ui| {
						ui.horizontal(|ui| {
							ui.text_edit_singleline(&mut self.task.name);

							if ui
								.button(
									egui::RichText::from("💾")
										.color(egui::Color32::from_rgb(0xAF, 0xAF, 0xFF)),
								)
								.on_hover_ui(|ui| {
									ui.label("Save task");
								})
								.clicked()
							{
								self.task.display();
							}

							if ui
								.button(
									egui::RichText::from("🗑")
										.color(ui.style().visuals.error_fg_color),
								)
								.on_hover_ui(|ui| {
									ui.label("Delete task");
								})
								.clicked()
							{
								set_pending_delete = true;
							}
						});

						ui.text_edit_multiline(&mut self.task.description);
					});
				}
			})
			.response;

		if let TaskState::Edit { pending_delete } = &mut self.task.state {
			*pending_delete |= set_pending_delete;
		}

		response
	}
}

impl<'task> TaskWidget<'task> {
	pub fn new(task: &'task mut Task) -> Self {
		Self { task }
	}
}

use crate::tag::Tag;

use super::{
	scheduled::{RepeatMode, ScheduledTask},
	NormalTaskData, Task, TaskPath, TaskState, TaskTypeData,
};

pub struct TaskWidget<'task, T> {
	task: &'task mut Task<T>,
}

impl egui::Widget for TaskWidget<'_, NormalTaskData> {
	fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
		self.draw_task_widget(ui, TaskPath::Tasks)
	}
}

impl<'task, T: TaskTypeData> TaskWidget<'task, T> {
	pub fn new(task: &'task mut Task<T>) -> Self {
		Self { task }
	}
}

impl egui::Widget for TaskWidget<'_, ScheduledTask> {
	fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
		let uuid = self.task.get_uuid().clone();
		ui.push_id(uuid, |ui| {
			ui.group(|ui| {
				ui.vertical(|ui| {
					match self.task.state {
						TaskState::Display => {
							ui.strong(format!(
								"{}, {}",
								self.task.type_data.date, self.task.type_data.repeat_mode
							));
						}
						TaskState::Edit { .. } => {
							ui.horizontal(|ui| {
								ui.add(egui_extras::DatePickerButton::new(
									&mut self.task.type_data.date,
								));

								ui.separator();
								ui.label("Repat mode");

								egui::ComboBox::new("repeat_mode", "")
									.selected_text(format!("{}", self.task.type_data.repeat_mode))
									.show_ui(ui, |ui| {
										ui.selectable_value(
											&mut self.task.type_data.repeat_mode,
											RepeatMode::Never,
											"Never",
										);
										ui.selectable_value(
											&mut self.task.type_data.repeat_mode,
											RepeatMode::Daily,
											"Daily",
										);
										ui.selectable_value(
											&mut self.task.type_data.repeat_mode,
											RepeatMode::Weekly,
											"Weekly",
										);
										ui.selectable_value(
											&mut self.task.type_data.repeat_mode,
											RepeatMode::Monthly,
											"Monthly",
										);
										ui.selectable_value(
											&mut self.task.type_data.repeat_mode,
											RepeatMode::Yearly,
											"Yealry",
										);
									});
							});
						}
					}
					self.draw_task_widget(ui, TaskPath::Scheduled);
				});
			})
			.response
		})
		.inner
	}
}

impl<T: TaskTypeData> TaskWidget<'_, T> {
	fn draw_task_widget(&mut self, ui: &mut egui::Ui, path: TaskPath) -> egui::Response {
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

						if !self.task.tags.is_empty() {
							ui.separator();

							ui.horizontal_wrapped(|ui| {
								for tag in self.task.tags.iter_mut() {
									ui.add(tag.widget(false));
									ui.add_space(8.0);
								}
							});
						}
					});
				}
				TaskState::Edit { no_buttons, .. } => {
					let no_buttons = *no_buttons;

					ui.vertical(|ui| {
						ui.horizontal(|ui| {
							ui.text_edit_singleline(&mut self.task.name);

							if !no_buttons {
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
									self.task.display(path);
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
							}
						});

						ui.text_edit_multiline(&mut self.task.description);

						ui.separator();

						ui.horizontal_wrapped(|ui| {
							let mut tags_to_remove = Vec::new();

							for (i, tag) in self.task.tags.iter_mut().enumerate() {
								ui.add(tag.widget(true));

								if ui
									.button(
										egui::RichText::from("❌")
											.color(ui.style().visuals.error_fg_color),
									)
									.clicked()
								{
									tags_to_remove.push(i);
								}

								ui.add_space(8.0);
							}

							for i in tags_to_remove.into_iter().rev() {
								self.task.tags.remove(i);
							}

							if ui.button("New Tag").clicked() {
								self.task.tags.push(Tag::default());
							}
						});
					});
				}
			})
			.response;

		if let TaskState::Edit { pending_delete, .. } = &mut self.task.state {
			*pending_delete |= set_pending_delete;
		}

		response
	}
}

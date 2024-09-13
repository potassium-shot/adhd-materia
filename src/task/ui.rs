use std::collections::HashMap;

use uuid::Uuid;

use crate::{
	settings::{Settings, DEFAULT_DATE_FORMAT},
	tag::Tag,
	toast_error,
	utils::ChronoDelayFormatExt,
};

use super::{
	scheduled::{RepeatMode, ScheduledTask},
	NormalTaskData, Task, TaskPath, TaskState, TaskTypeData,
};

pub struct TaskWidgetResponse {
	pub changed: bool,
	pub selected: bool,
	pub rect: egui::Rect,
}

pub struct TaskWidget<'task, T> {
	task: &'task mut Task<T>,
}

impl TaskWidget<'_, NormalTaskData> {
	pub fn show(
		mut self,
		ui: &mut egui::Ui,
		task_names: &HashMap<Uuid, String>,
		selected: bool,
		scroll_to: &mut Option<Uuid>,
	) -> TaskWidgetResponse {
		self.draw_task_widget(ui, TaskPath::Tasks, true, task_names, selected, scroll_to)
	}
}

impl<'task, T: TaskTypeData> TaskWidget<'task, T> {
	pub fn new(task: &'task mut Task<T>) -> Self {
		Self { task }
	}
}

impl TaskWidget<'_, ScheduledTask> {
	pub fn show(
		mut self,
		ui: &mut egui::Ui,
		task_names: &HashMap<Uuid, String>,
		scroll_to: &mut Option<Uuid>,
	) -> TaskWidgetResponse {
		let uuid = self.task.get_uuid().clone();

		ui.push_id(uuid, |ui| {
			if !self.task.type_data.active {
				ui.set_opacity(0.5);
			}

			ui.group(|ui| {
				ui.vertical(|ui| {
					match self.task.state {
						TaskState::Display => {
							ui.horizontal_top(|ui| {
								ui.add(egui::Checkbox::without_text(
									&mut self.task.type_data.active,
								));

								ui.strong(format!(
									"{}, {}",
									self.task
										.type_data
										.date
										.format_or_err(Settings::get().date_format.as_str())
										.unwrap_or(
											self.task
												.type_data
												.date
												.format(DEFAULT_DATE_FORMAT)
												.to_string()
										),
									self.task.type_data.repeat_mode
								));
							});
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
					self.draw_task_widget(
						ui,
						TaskPath::Scheduled,
						false,
						task_names,
						false,
						scroll_to,
					)
				})
				.inner
			})
			.inner
		})
		.inner
	}
}

impl<T: TaskTypeData> TaskWidget<'_, T> {
	fn draw_task_widget(
		&mut self,
		ui: &mut egui::Ui,
		path: TaskPath,
		can_be_done: bool,
		task_names: &HashMap<Uuid, String>,
		selected: bool,
		scroll_to: &mut Option<Uuid>,
	) -> TaskWidgetResponse {
		let mut response = TaskWidgetResponse {
			changed: false,
			selected: false,
			rect: egui::Rect::ZERO,
		};

		ui.push_id(*self.task.get_uuid(), |ui| {
			let mut set_pending_delete = false;

			if self.task.is_done() {
				ui.set_opacity(0.5);
			}

			response.rect = egui::Frame::group(ui.style())
				.stroke(egui::Stroke::new(
					ui.visuals().window_stroke.width,
					if selected {
						ui.visuals().hyperlink_color
					} else {
						ui.visuals().window_stroke.color
					},
				))
				.show(ui, |ui| match &self.task.state {
					TaskState::Display => {
						ui.vertical(|ui| {
							ui.horizontal(|ui| {
								if can_be_done {
									let done_tag = Settings::get_done_tag();

									ui.add_enabled_ui(!self.task.tags.contains(&done_tag), |ui| {
										let bg_color = ui.visuals().hyperlink_color;
										let fg_color = ui.visuals().window_fill;

										ui.visuals_mut().override_text_color = Some(fg_color);
										ui.visuals_mut().widgets.inactive.weak_bg_fill = bg_color;
										ui.visuals_mut().widgets.active.weak_bg_fill = bg_color;
										ui.visuals_mut().widgets.hovered.weak_bg_fill = bg_color;

										if ui.button(egui::RichText::new("✅").size(20.0)).clicked()
										{
											self.task.tags.insert(0, done_tag.clone());

											if let Err(e) = self.task.save(path) {
												toast_error!("Could not save task: {}", e);
											}

											response.changed = true;
										}
									});

									ui.separator();
								}

								response.selected = ui
									.button(
										egui::RichText::from(self.task.name.as_str())
											.heading()
											.color(if selected {
												ui.visuals().hyperlink_color
											} else {
												ui.visuals().text_color()
											}),
									)
									.clicked();

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
										tag.widget(false).show(ui, task_names, scroll_to);
										ui.add_space(8.0);
									}
								});
							}

							ui.add_space(4.0);

							ui.with_layout(
								egui::Layout::right_to_left(egui::Align::BOTTOM),
								|ui| {
									if ui.small_button("📋").clicked() {
										ui.output_mut(|o| {
											o.copied_text = self.task.get_uuid().to_string();
										});
									}

									ui.small(
										egui::RichText::new(self.task.get_uuid().to_string())
											.weak()
											.monospace(),
									);
								},
							);
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
											egui::RichText::from("💾").color(
												Settings::get().theme.get_catppuccin().lavender,
											),
										)
										.on_hover_ui(|ui| {
											ui.label("Save task");
										})
										.clicked()
									{
										self.task.display(path);
										response.changed = true;
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
								let mut tags_to_swap = None;

								for (i, tag) in self.task.tags.iter_mut().enumerate() {
									if let Some(swap_dir) =
										tag.widget(true).show(ui, task_names, scroll_to)
									{
										tags_to_swap = Some((
											i as isize,
											match swap_dir {
												crate::tag::TagSwapRequest::Forward => {
													(i as isize) + 1
												}
												crate::tag::TagSwapRequest::Backward => {
													(i as isize) - 1
												}
											},
										));
									}

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

								if let Some((a, b)) = tags_to_swap {
									let a = a.clamp(0, self.task.tags.len() as isize - 1) as usize;
									let b = b.clamp(0, self.task.tags.len() as isize - 1) as usize;
									self.task.tags.swap(a, b);
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
				.response
				.rect;

			if let TaskState::Edit { pending_delete, .. } = &mut self.task.state {
				*pending_delete |= set_pending_delete;
			}
		});

		response
	}
}

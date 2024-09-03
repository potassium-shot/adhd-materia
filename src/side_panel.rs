use std::time::Duration;

use crate::{
	ok_cancel_dialog::{OkCancelDialog, OkCancelResult},
	task::{
		list::{TaskList, TaskListError},
		scheduled::ScheduledTask,
		TaskPath,
	},
};

#[derive(Default, kinded::Kinded)]
pub enum SidePanel {
	#[default]
	Hidden,
	ScheduledTasks {
		scheduled_task_list: Result<TaskList<ScheduledTask>, TaskListError>,
		interactable: bool,
	},
	Scripts {},
}

impl SidePanel {
	pub fn show(&mut self, ui: &mut egui::Ui) {
		match self {
			Self::Hidden => {}
			Self::ScheduledTasks {
				scheduled_task_list,
				interactable,
			} => {
				ui.heading("Scheduled Tasks");
				ui.separator();
				ui.add_space(16.0);

				ui.add_enabled_ui(*interactable, |ui| {
					if !*interactable {
						ui.multiply_opacity(0.25);
					}

					*interactable = true;

					match scheduled_task_list {
						Ok(task_list) => {
							ui.with_layout(
								egui::Layout::top_down_justified(egui::Align::Min),
								|ui| {
									egui::Grid::new("task_grid")
										.num_columns(1)
										.spacing((40.0, 4.0))
										.striped(true)
										.show(ui, |ui| {
											for task in task_list.tasks_mut() {
												ui.add(task.widget());
												ui.end_row();

												if task.is_pending_delete() {
													*interactable = false;

													if let Some(result) = OkCancelDialog::default()
														.with_title(format!(
															"Delete scheduled task {}?",
															task.name
														))
														.with_subtext(
															"You cannot undo this action.",
														)
														.with_ok_text("Delete")
														.with_ok_color(
															ui.style().visuals.error_fg_color,
														)
														.show(ui.ctx())
													{
														match result {
															OkCancelResult::Ok => {
																task.mark_for_delete()
															}
															OkCancelResult::Cancel => task.edit(),
														}
													}
												}
											}
										});

									ui.add_space(16.0);

									if ui.button("New Task").clicked() {
										if let Err(e) = task_list.new_task() {
											crate::toasts()
												.error(format!(
													"Could not create scheduled task: {}",
													e
												))
												.set_closable(true)
												.set_duration(Some(Duration::from_millis(10_000)));
										}
									}
								},
							);

							for e in task_list.cleanup_marked_for_delete() {
								crate::toasts()
									.error(format!("Could not delete scheduled task: {}", e))
									.set_closable(true)
									.set_duration(Some(Duration::from_millis(10_000)));
							}
						}
						Err(error) => {
							ui.add_sized(
								ui.available_size(),
								egui::Label::new(
									egui::RichText::from(format!(
										"Couldn't load scheduled task list: {}",
										error
									))
									.color(ui.style().visuals.error_fg_color)
									.heading(),
								),
							);
						}
					}
				});
			}
			Self::Scripts {} => {
				ui.heading("Scripts");
				ui.separator();
				ui.add_space(16.0);

				ui.label("Dinosaur haha");
			}
		}
	}

	pub fn toggle(&mut self, kind: SidePanelKind) {
		if self.kind() == kind {
			self.hide();
		} else {
			self.open(kind);
		}
	}

	pub fn open(&mut self, kind: SidePanelKind) {
		if kind != self.kind() {
			self.close();
		}

		*self = match kind {
			SidePanelKind::ScheduledTasks => {
				let task_list = TaskList::<ScheduledTask>::new(TaskPath::Scheduled);

				if let Ok((_, errors)) = &task_list {
					errors.iter().for_each(|error| {
						crate::toasts()
							.error(format!("Couldn't load task: {}", error))
							.set_closable(true)
							.set_duration(Some(Duration::from_millis(10_000)));
					});
				}

				Self::ScheduledTasks {
					scheduled_task_list: task_list.map(|(list, _)| list),
					interactable: true,
				}
			}
			SidePanelKind::Scripts => Self::Scripts {},
			SidePanelKind::Hidden => Self::Hidden,
		}
	}

	fn close(&mut self) {
		match self {
			SidePanel::Hidden => {}
			SidePanel::ScheduledTasks {
				scheduled_task_list,
				..
			} => {
				if let Ok(list) = scheduled_task_list {
					list.save_all();
				}
			}
			SidePanel::Scripts {} => {}
		}
	}

	pub fn hide(&mut self) {
		self.open(SidePanelKind::Hidden);
	}

	pub fn is_shown(&self) -> bool {
		match self {
			Self::Hidden => false,
			_ => true,
		}
	}
}

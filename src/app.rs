use std::time::Duration;

use eframe::{App, CreationContext};

use crate::{
	ok_cancel_dialog::{OkCancelDialog, OkCancelResult},
	settings::Settings,
	side_panel::{SidePanel, SidePanelKind},
	startup_script::StartupScript,
	task::{
		list::{TaskList, TaskListError},
		TaskPath,
	},
};

pub struct AdhdMateriaApp {
	task_list: Result<TaskList, TaskListError>,
	side_panel: SidePanel,

	interactable: bool,
}

impl AdhdMateriaApp {
	pub fn new(cc: &CreationContext) -> Self {
		// Enable catppuccin theme
		Settings::get().theme.apply(&cc.egui_ctx);

		// Setup Nunito font
		const NUNITO_REGULAR: &str = "nunito_regular";

		let mut fonts = egui::FontDefinitions::default();

		fonts.font_data.insert(
			String::from(NUNITO_REGULAR),
			egui::FontData::from_static(include_bytes!("../assets/fonts/Nunito-Regular.ttf")),
		);

		fonts
			.families
			.entry(egui::FontFamily::Proportional)
			.or_default()
			.insert(0, String::from(NUNITO_REGULAR));

		cc.egui_ctx.set_fonts(fonts);

		match StartupScript::run() {
			Ok(errors) => {
				for error in errors {
					crate::toasts()
						.error(format!("Scheduled tasks error: {}", error))
						.set_closable(true)
						.set_duration(Some(Duration::from_millis(10_000)));
				}
			}
			Err(e) => {
				crate::toasts()
					.error(format!("{}", e))
					.set_closable(true)
					.set_duration(None);
			}
		}

		// Load tasks and report errors
		let task_list = TaskList::new(TaskPath::Tasks);

		if let Ok((_, errors)) = &task_list {
			errors.iter().for_each(|error| {
				crate::toasts()
					.error(format!("Couldn't load task: {}", error))
					.set_closable(true)
					.set_duration(Some(Duration::from_millis(10_000)));
			});
		}

		Self {
			task_list: task_list.map(|(list, _)| list),
			side_panel: SidePanel::default(),

			interactable: true,
		}
	}
}

impl App for AdhdMateriaApp {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		egui::SidePanel::left("left_panel_buttons")
			.exact_width(64.0)
			.resizable(false)
			.show(ctx, |ui| {
				ui.vertical_centered_justified(|ui| {
					let mut side_panel_button =
						|ui: &mut egui::Ui, kind: SidePanelKind, c: char| {
							if ui
								.add(
									egui::Button::new(
										egui::RichText::from(c.to_string()).size(32.0).color(
											if self.side_panel.kind() == kind {
												if Settings::get().theme.is_dark() {
													egui::Color32::WHITE
												} else {
													egui::Color32::BLACK
												}
											} else {
												egui::Color32::PLACEHOLDER
											},
										),
									)
									.frame(false),
								)
								.clicked()
							{
								self.side_panel.toggle(kind);
							}
						};

					side_panel_button(ui, SidePanelKind::ScheduledTasks, '🕗');
					ui.separator();
					side_panel_button(ui, SidePanelKind::Scripts, '📃');
					ui.separator();
					side_panel_button(ui, SidePanelKind::Settings, '⛭');
				});
			});

		egui::SidePanel::left("left_panel")
			.min_width(192.0)
			.default_width(384.0)
			.show_animated(ctx, self.side_panel.is_shown(), |ui| {
				self.side_panel.show(ui);
			});

		egui::CentralPanel::default().show(ctx, |ui| {
			ui.heading("Task List");
			ui.horizontal_wrapped(|ui| {
				ui.label("This is a list of today's tasks. Tasks can be made manually, or then can be scheduled using the");
				if ui.link(egui::RichText::from("🕗 Task Scheduler").color(ui.style().visuals.hyperlink_color)).clicked() {
					self.side_panel.open(SidePanelKind::ScheduledTasks);
				}
				ui.label("tab.");
			});
			ui.separator();
			ui.add_space(8.0);

			let clear_done = ui.horizontal_wrapped(|ui| {
				ui.button("Clear Done Tasks").clicked()
			}).inner;

			ui.add_space(8.0);
			ui.separator();
			ui.add_space(16.0);

			ui.add_enabled_ui(self.interactable, |ui| {
				if !self.interactable {
					ui.multiply_opacity(0.25);
				}

				self.interactable = true;

				match &mut self.task_list {
					Ok(task_list) => {
						ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
							egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
								egui::Grid::new("task_grid")
									.num_columns(1)
									.spacing((40.0, 12.0))
									.striped(true)
									.show(ui, |ui| {
										for task in task_list.tasks_mut() {
											ui.add(task.widget());
											ui.end_row();

											if task.is_pending_delete() {
												self.interactable = false;

												if let Some(result) = OkCancelDialog::default()
													.with_title(format!("Delete task {}?", task.name))
													.with_subtext("You cannot undo this action.")
													.with_ok_text("Delete")
													.with_ok_color(ui.style().visuals.error_fg_color)
													.show(ctx)
												{
													match result {
														OkCancelResult::Ok => task.mark_for_delete(),
														OkCancelResult::Cancel => task.edit(),
													}
												}
											}

											if clear_done && task.is_done() {
												task.mark_for_delete();
											}
										}
									});

								ui.add_space(16.0);

								if ui.button("New Task").clicked() {
									let mut new_task = Settings::get().default_task.clone();
									new_task.edit();

									if let Err(e) = task_list.add_task(new_task) {
										crate::toasts()
											.error(format!("Could not create task: {}", e))
											.set_closable(true)
											.set_duration(Some(Duration::from_millis(10_000)));
									}
								}
							});
						});

						for e in task_list.cleanup_marked_for_delete() {
							crate::toasts()
								.error(format!("Could not delete task: {}", e))
								.set_closable(true)
								.set_duration(Some(Duration::from_millis(10_000)));
						}
					}
					Err(error) => {
						ui.add_sized(
							ui.available_size(),
							egui::Label::new(
								egui::RichText::from(format!("Couldn't load task list: {}", error))
									.color(ui.style().visuals.error_fg_color)
									.heading(),
							),
						);
					}
				}
			});
		});

		crate::toasts().show(ctx);
	}
}

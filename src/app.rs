use std::{
	collections::{HashMap, VecDeque},
	sync::Mutex,
	time::Duration,
};

use eframe::{App, CreationContext};
use uuid::Uuid;

use crate::{
	data_dir::DataDirError,
	handle_toast_error,
	ok_cancel_dialog::{OkCancelDialog, OkCancelResult},
	scripts::{
		badge::{BadgeList, BadgeType},
		filter::FilterList,
		sorting::SortingList,
		PocketPyLock,
	},
	settings::Settings,
	side_panel::{SidePanel, SidePanelKind},
	startup_script::StartupScript,
	task::{
		display_list::TaskDisplayList,
		list::{TaskList, TaskListError},
		TaskPath,
	},
	toast_info, toast_success,
};

static mut SCRIPT_LOCK: Option<crate::scripts::PocketPyLock> = None;

pub fn script_lock() -> crate::scripts::PocketPyLockGuard<'static> {
	unsafe { SCRIPT_LOCK.as_ref().unwrap() }.lock()
}

static SCRIPTS_WAITLIST: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());

pub fn push_script_to_waitlist(script_name: String) {
	SCRIPTS_WAITLIST.lock().unwrap().push_back(script_name);
}

pub struct AdhdMateriaApp {
	task_list: Result<TaskList, TaskListError>,
	task_display_list: Option<TaskDisplayList>,
	task_name_cache: HashMap<Uuid, String>,
	side_panel: SidePanel,

	selected_task: Option<Uuid>,

	interactable: bool,

	filter_list: Result<FilterList, &'static DataDirError>,
	sorting_list: Result<SortingList, &'static DataDirError>,
}

impl AdhdMateriaApp {
	pub fn new(cc: &CreationContext) -> Self {
		unsafe {
			SCRIPT_LOCK = Some(PocketPyLock::new());
		}

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

		let filter_list = FilterList::new();
		let sorting_list = SortingList::new();

		Self {
			task_display_list: task_list
				.as_ref()
				.map(|(list, _)| list)
				.ok()
				.and_then(|task_list| Some((task_list, filter_list.as_ref().ok()?)))
				.and_then(|(task_list, filter_list)| {
					Some((task_list, filter_list, sorting_list.as_ref().ok()?))
				})
				.map(|(task_list, filter_list, sorting_list)| {
					TaskDisplayList::new(task_list, filter_list, sorting_list)
				}),
			task_list: task_list.map(|(list, _)| list),
			task_name_cache: HashMap::new(),
			side_panel: SidePanel::default(),

			selected_task: None,

			interactable: true,

			filter_list,
			sorting_list,
		}
	}
}

impl App for AdhdMateriaApp {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		let left_panel_was_shown = self.side_panel.is_shown();

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
					side_panel_button(ui, SidePanelKind::FilterScripts, '🔻');
					ui.separator();
					side_panel_button(ui, SidePanelKind::SortingScripts, '🔤');
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
				self.side_panel.show(ui, &self.task_name_cache);
			});

		if left_panel_was_shown && !self.side_panel.is_shown() {
			self.filter_list = FilterList::new();
			self.sorting_list = SortingList::new();
		}

		let mut update_required = self
			.filter_list
			.as_mut()
			.is_ok_and(|filter_list| filter_list.check_changed())
			|| self
				.sorting_list
				.as_mut()
				.is_ok_and(|sorting_list| sorting_list.check_changed());

		if let Ok(task_list) = self.task_list.as_mut() {
			egui::SidePanel::right("right_panel")
				.min_width(384.0)
				.default_width(576.0)
				.show_animated(ctx, self.selected_task.is_some(), |ui| {
					let selected_task_id = self.selected_task.as_ref().expect("is some");
					let selected_task = task_list
						.get_mut(selected_task_id)
						.expect("selected id should be valid");

					ui.horizontal(|ui| {
						ui.heading(selected_task.name.as_str());
						if ui.button("⮪").clicked() {
							self.selected_task = None;
						}
					});
					ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
						if ui.small_button("📋").clicked() {
							ui.output_mut(|o| {
								o.copied_text = selected_task.get_uuid().to_string();
							});
						}

						ui.small(
							egui::RichText::new(selected_task.get_uuid().to_string())
								.weak()
								.monospace(),
						);
					});
					ui.separator();
					ui.add_space(8.0);

					ui.label(selected_task.description.as_str());

					ui.add_space(8.0);
					ui.separator();
					ui.add_space(8.0);

					ui.horizontal_wrapped(|ui| {
						for tag in selected_task.tags.iter_mut() {
							tag.widget(false).show(ui, &self.task_name_cache);
							ui.add_space(8.0);
						}
					});
				});
		}

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

			let mut done_cleared = 0;

			ui.add_space(8.0);

			show_badge_list(ui, &mut self.filter_list, "Filter");
			show_badge_list(ui, &mut self.sorting_list, "Sorting");

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
										for task_id in self.task_display_list.as_ref().expect("display list should be Some when task list is ok").tasks() {
											let task = task_list.get_mut(task_id).expect("task display list should only have valid uuids");
											let task_widget_response = task.widget().show(ui, &self.task_name_cache);
											update_required |= task_widget_response.changed;

											if task_widget_response.selected {
												if self.selected_task == Some(*task_id) {
													self.selected_task = None;
												} else {
													self.selected_task = Some(task_id.clone());
												}
											}

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
												done_cleared += 1;
											}
										}
									});

								ui.add_space(16.0);

								if ui.button("New Task").clicked() {
									let mut new_task = Settings::get().default_task.clone();
									new_task.new_uuid();
									new_task.edit();

									if let Err(e) = task_list.add_task(new_task) {
										crate::toasts()
											.error(format!("Could not create task: {}", e))
											.set_closable(true)
											.set_duration(Some(Duration::from_millis(10_000)));
									}

									update_required = true;
								}
							});
						});

						let (anything_deleted, cleanup_errors) = task_list.cleanup_marked_for_delete();

						update_required |= anything_deleted;

						for e in cleanup_errors {
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

			if clear_done {
				if done_cleared > 0 {
					toast_success!("{} tasks cleared", done_cleared);
				} else {
					toast_info!("No tasks to clear");
				}
			}
		});

		if update_required {
			if self.task_display_list.take().is_some() {
				self.task_display_list = Some(TaskDisplayList::new(
					self.task_list.as_ref().expect("task display list is some"),
					self.filter_list
						.as_ref()
						.expect("task display list is some"),
					self.sorting_list.as_ref().expect("task display is some"),
				));
			}

			if let Ok(task_list) = self.task_list.as_ref() {
				self.task_name_cache.clear();

				for task in task_list.tasks() {
					self.task_name_cache
						.insert(task.get_uuid().clone(), task.name.clone());
				}
			}
		}

		let mut script_waitlist = SCRIPTS_WAITLIST.lock().unwrap();

		while let Some(script) = script_waitlist.pop_front() {
			crate::scripts::run_standalone_script(script.as_str())
		}

		while let Some(new_task) = crate::scripts::new_tasks_waitlist_next() {
			if let Ok(task_list) = self.task_list.as_mut() {
				handle_toast_error!("Script couldn't add task: {}", task_list.add_task(new_task));
			}
		}

		crate::toasts().show(ctx);
	}
}

impl Drop for AdhdMateriaApp {
	fn drop(&mut self) {
		unsafe {
			SCRIPT_LOCK = None;
		}
	}
}

fn show_badge_list<T: BadgeType>(
	ui: &mut egui::Ui,
	badge_list: &mut Result<BadgeList<T>, &'static DataDirError>,
	script_name: &'static str,
) {
	match badge_list {
		Ok(ref mut badge_list) => {
			ui.horizontal_wrapped(|ui| {
				ui.label(egui::RichText::new(format!("{}: ", script_name)).size(16.0));

				let mut to_swap = None;

				for (idx, (badge, mut enabled)) in badge_list.iter_all().enumerate() {
					let was_enabled = enabled;

					ui.add(crate::scripts::badge::Badge::<T>::new(
						badge,
						&mut enabled,
						idx as i32 + 1,
					));

					if was_enabled != enabled {
						to_swap = Some(idx);
					}
				}

				if let Some(idx) = to_swap {
					badge_list.swap(idx);
				}
			});
		}
		Err(e) => {
			ui.label(format!("Couldn't load {} scripts: {}", script_name, e));
		}
	}
}

use std::{collections::HashMap, time::Duration};

use uuid::Uuid;

use crate::{
	data_dir::DataDirError,
	ok_cancel_dialog::{OkCancelDialog, OkCancelResult},
	scripts::{
		badge::BadgeType,
		filter::{FilterBadgeType, DEFAULT_FILTER_SCRIPT},
		list::{ScriptEditorDeletionState, ScriptList},
		sorting::{SortingBadgeType, DEFAULT_SORTING_SCRIPT},
		standalone_script::{StandaloneScriptBadgeType, DEFAULT_STANDALONE_SCRIPT},
		PocketPyScript,
	},
	settings::{self, AdhdMateriaTheme, Settings, DEFAULT_SCHEDULED_TASK_TAG},
	task::{
		list::{TaskList, TaskListError},
		scheduled::ScheduledTask,
		TaskPath,
	},
	toast_error,
};

macro_rules! open_scripts {
	($badge_type: ty, $side_panel_type: ident) => {
		SidePanel::$side_panel_type {
			script_list: match ScriptList::new() {
				Ok((script_list, errors)) => {
					errors.into_iter().for_each(|e| {
						toast_error!("Couldn't load script: {}", e);
					});

					Ok(script_list)
				}
				Err(e) => Err(e),
			},
			interactable: true,
		}
	};
}

#[derive(Default, kinded::Kinded)]
pub enum SidePanel {
	#[default]
	Hidden,
	ScheduledTasks {
		scheduled_task_list: Result<TaskList<ScheduledTask>, TaskListError>,
		interactable: bool,
	},
	FilterScripts {
		script_list: Result<ScriptList<FilterBadgeType>, &'static DataDirError>,
		interactable: bool,
	},
	SortingScripts {
		script_list: Result<ScriptList<SortingBadgeType>, &'static DataDirError>,
		interactable: bool,
	},
	Scripts {
		script_list: Result<ScriptList<StandaloneScriptBadgeType>, &'static DataDirError>,
		interactable: bool,
	},
	Settings,
}

impl SidePanel {
	pub fn show(&mut self, ui: &mut egui::Ui, task_names: &HashMap<Uuid, String>) {
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
												task.widget().show(ui, task_names);
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
										let mut new_task = Settings::get()
											.default_task
											.clone()
											.convert(ScheduledTask::default());
										new_task.new_uuid();
										new_task.edit();

										if let Err(e) = task_list.add_task(new_task) {
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

							for e in task_list.cleanup_marked_for_delete().1 {
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
			Self::FilterScripts {
				script_list,
				interactable,
			} => {
				show_scripts(
					ui,
					script_list,
					"Filter",
					&DEFAULT_FILTER_SCRIPT,
					interactable,
				);
			}
			Self::SortingScripts {
				script_list,
				interactable,
			} => {
				show_scripts(
					ui,
					script_list,
					"Sorting",
					&DEFAULT_SORTING_SCRIPT,
					interactable,
				);
			}
			Self::Scripts {
				script_list,
				interactable,
			} => {
				show_scripts(
					ui,
					script_list,
					"Standalone",
					&DEFAULT_STANDALONE_SCRIPT,
					interactable,
				);
			}
			Self::Settings => {
				ui.heading("Settings");
				ui.separator();
				ui.add_space(16.0);

				egui::Grid::new("settings_list")
					.num_columns(2)
					.spacing((40.0, 8.0))
					.show(ui, |ui| {
						let mut settings = Settings::get();

						ui.label("Theme");

						if egui::ComboBox::from_id_source("theme_selector")
							.selected_text(format!("{}", settings.theme))
							.show_ui(ui, |ui| {
								let mut changed = false;

								changed |= ui
									.selectable_value(
										&mut settings.theme,
										AdhdMateriaTheme::CatppuccinLatte,
										format!("{}", AdhdMateriaTheme::CatppuccinLatte),
									)
									.clicked();
								changed |= ui
									.selectable_value(
										&mut settings.theme,
										AdhdMateriaTheme::CatppuccinFrappe,
										format!("{}", AdhdMateriaTheme::CatppuccinFrappe),
									)
									.clicked();
								changed |= ui
									.selectable_value(
										&mut settings.theme,
										AdhdMateriaTheme::CatppuccinMacchiato,
										format!("{}", AdhdMateriaTheme::CatppuccinMacchiato),
									)
									.clicked();
								changed |= ui
									.selectable_value(
										&mut settings.theme,
										AdhdMateriaTheme::CatppuccinMocha,
										format!("{}", AdhdMateriaTheme::CatppuccinMocha),
									)
									.clicked();

								changed
							})
							.inner
							.is_some_and(|b| b)
						{
							settings.theme.apply(ui.ctx());
						}

						ui.end_row();

						settings.default_task.edit_no_buttons();

						ui.label("Default task");
						settings.default_task.widget().show(ui, task_names);
						ui.end_row();

						ui.label("Repeating task rewind");
						egui::ComboBox::from_id_source("repeatable_rewind")
							.selected_text(format!("{:?}", settings.repeatable_rewind))
							.show_ui(ui, |ui| {
								ui.selectable_value(
									&mut settings.repeatable_rewind,
									settings::RepeatableRewind::One,
									"One",
								);
								ui.selectable_value(
									&mut settings.repeatable_rewind,
									settings::RepeatableRewind::All,
									"All",
								);
							})
							.response
							.on_hover_ui(|ui| {
								ui.label(
									"When a repeating task should have triggered more than once, \
									One will make it trigger once, and All will make it trigger all times",
								);
							});
						ui.end_row();

						ui.label("Scheduled task tag");

						ui.horizontal(|ui| {
							let mut scheduled_task_tag_enabled =
								settings.scheduled_task_tag.is_some();
							let mut scheduled_task_tag =
								settings.scheduled_task_tag.clone().unwrap_or_default();

							ui.add(egui::Checkbox::without_text(
								&mut scheduled_task_tag_enabled,
							));
							ui.add_enabled(
								scheduled_task_tag_enabled,
								egui::TextEdit::singleline(&mut scheduled_task_tag),
							)
							.on_hover_ui(|ui| {
								ui.label("Tag to apply to scheduled tasks, $DATE is replaced by the scheduled date");
							});

							if let Some(tag) = &mut settings.scheduled_task_tag {
								*tag = scheduled_task_tag;
							}

							if scheduled_task_tag_enabled {
								if settings.scheduled_task_tag.is_none() {
									settings.scheduled_task_tag =
										Some(String::from(DEFAULT_SCHEDULED_TASK_TAG));
								}
							} else {
								settings.scheduled_task_tag = None;
							}
						});

						ui.end_row();

						ui.label("Delete used scheduled tasks");
						ui.add(egui::Checkbox::without_text(
							&mut settings.delete_used_scheduled_tasks,
						))
						.on_hover_ui(|ui| {
							ui.label("Delete scheduled tasks that have been used and are never to be triggered again.");
						});
						ui.end_row();

						ui.horizontal(|ui| {
							ui.label("Date format");
							if ui.link(egui::RichText::new("reference").color(ui.visuals().hyperlink_color)).clicked() {
								ui.output_mut(|o| {
									o.open_url = Some(egui::OpenUrl::new_tab("https://docs.rs/chrono/latest/chrono/format/strftime/index.html"));
								});
							}
						});
						ui.text_edit_singleline(&mut settings.date_format);
						ui.end_row();

						ui.label("Done tag name");
						ui.text_edit_singleline(settings.get_done_tag_string_mut().as_mut());
						ui.end_row();
					});
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
			SidePanelKind::FilterScripts => open_scripts!(FilterBadgeType, FilterScripts),
			SidePanelKind::SortingScripts => open_scripts!(SortingBadgeType, SortingScripts),
			SidePanelKind::Scripts => open_scripts!(StandaloneScriptBadgeType, Scripts),
			SidePanelKind::Settings => Self::Settings,
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
			SidePanel::FilterScripts { script_list, .. } => {
				close_scripts(script_list, "Filter");
			}
			SidePanel::SortingScripts { script_list, .. } => {
				close_scripts(script_list, "Sorting");
			}
			SidePanel::Scripts { script_list, .. } => {
				close_scripts(script_list, "Standalone");
			}
			SidePanel::Settings => {
				let mut settings = Settings::get();
				settings.default_task.apply_tags();

				if let Err(e) = settings.save() {
					crate::toasts()
						.error(format!("Could not save settings: {}", e))
						.set_closable(true)
						.set_duration(Some(Duration::from_millis(10_000)));
				}
			}
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

impl Drop for SidePanel {
	fn drop(&mut self) {
		self.close();
	}
}

fn show_scripts<T: BadgeType>(
	ui: &mut egui::Ui,
	script_list: &mut Result<ScriptList<T>, &DataDirError>,
	script_name: &'static str,
	default_script: &std::sync::LazyLock<PocketPyScript>,
	interactable: &mut bool,
) {
	ui.add_enabled_ui(*interactable, |ui| {
		ui.heading(format!("{} Scripts", script_name));
		ui.separator();
		ui.add_space(16.0);

		match script_list {
			Ok(script_list) => {
				egui::ScrollArea::vertical()
					.auto_shrink(false)
					.show(ui, |ui| {
						ui.vertical_centered_justified(|ui| {
							for script in script_list.scripts_mut() {
								script.widget().show(ui);

								if script.deletion_state == ScriptEditorDeletionState::Pending {
									*interactable = false;

									if let Some(result) = OkCancelDialog::default()
										.with_title(format!(
											"Delete {} Script {}?",
											script_name, script.script.name
										))
										.with_subtext("You cannot undo this action.")
										.with_ok_text("Delete")
										.with_ok_color(ui.style().visuals.error_fg_color)
										.show(ui.ctx())
									{
										match result {
											OkCancelResult::Ok => {
												script.deletion_state =
													ScriptEditorDeletionState::Marked;
												*interactable = true;
											}
											OkCancelResult::Cancel => {
												script.deletion_state =
													ScriptEditorDeletionState::None;
												*interactable = true;
											}
										}
									}
								}
							}

							script_list.cleanup();

							if ui.button(format!("New {} Script", script_name)).clicked() {
								if let Err(e) = script_list.add((*default_script).clone()) {
									toast_error!("Couldn't create {} script: {}", script_name, e);
								}
							}
						});
					});
			}
			Err(e) => {
				ui.label(
					egui::RichText::new(format!("Couldn't load {} scripts: {}", script_name, e))
						.color(ui.style().visuals.error_fg_color)
						.heading(),
				);
			}
		}
	});
}

fn close_scripts<T: BadgeType>(
	script_list: &mut Result<ScriptList<T>, &DataDirError>,
	script_name: &'static str,
) {
	if let Ok(list) = script_list {
		list.save_all().into_iter().for_each(|e| {
			toast_error!("Couldn't save {} script: {}", script_name, e);
		});
	}
}

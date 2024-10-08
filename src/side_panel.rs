use std::{collections::HashMap, time::Duration};

use uuid::Uuid;

use crate::{
	data_dir::DataDirError, help_string, ok_cancel_dialog::{OkCancelDialog, OkCancelResult}, scripts::{
		badge::BadgeType,
		filter::{FilterBadgeType, DEFAULT_FILTER_SCRIPT},
		list::{ScriptEditorDeletionState, ScriptList},
		sorting::{SortingBadgeType, DEFAULT_SORTING_SCRIPT},
		standalone_script::{StandaloneScriptBadgeType, DEFAULT_STANDALONE_SCRIPT},
		PocketPyScript,
	}, session::Session, settings::{
		self, AdhdMateriaTheme, Settings, SprintFrequency, SprintFrequencyKind,
		DEFAULT_SCHEDULED_TASK_TAG,
	}, task::{
		list::{TaskList, TaskListError},
		scheduled::ScheduledTask,
		TaskPath,
	}, toast_error
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
	CompletedTasks {
		total_completed_tasks: i32,
	},
	Settings {
		color_associations_cache: Vec<(String, egui::epaint::Hsva)>,
	},
}

impl SidePanelKind {
	pub fn name(&self) -> &'static str {
		match self {
			SidePanelKind::Hidden => "",
			SidePanelKind::ScheduledTasks => "Scheduled Tasks",
			SidePanelKind::FilterScripts => "Filter Scripts",
			SidePanelKind::SortingScripts => "Sorting Scripts",
			SidePanelKind::Scripts => "Scripts",
			SidePanelKind::CompletedTasks => "Completed Tasks",
			SidePanelKind::Settings => "Settings",
		}
	}
}

impl SidePanel {
	pub fn show(
		&mut self,
		ui: &mut egui::Ui,
		task_names: &HashMap<Uuid, String>,
		scroll_to: &mut Option<Uuid>,
		selected_task: &mut Option<Uuid>,
	) {
		ui.add_space(8.0);

		match self {
			Self::Hidden => {}
			Self::ScheduledTasks {
				scheduled_task_list,
				interactable,
			} => {
				ui.heading("Scheduled Tasks");
				ui.separator();
				ui.add_space(8.0);

				help_string!(ui, "scheduled_task_list");
				help_string!(ui, "scheduled_task_list_2");

				ui.add_space(8.0);

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
												task.widget().show(ui, task_names, scroll_to, selected_task);
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
				ui.heading("Filter Scripts");
				ui.separator();
				ui.add_space(8.0);

				help_string!(ui, "filter_scripts");
				help_string!(ui, "filter_scripts_2");

				match help_string!(ui, "other_scripts") {
					Some(0) => {
						self.open(SidePanelKind::Scripts);
						return;
					}
					_ => {}
				}

				ui.add_space(8.0);

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
				ui.heading("Sorting Scripts");
				ui.separator();
				ui.add_space(8.0);

				help_string!(ui, "sorting_scripts");
				help_string!(ui, "sorting_scripts_2");

				match help_string!(ui, "other_scripts") {
					Some(0) => {
						self.open(SidePanelKind::Scripts);
						return;
					}
					_ => {}
				}

				ui.add_space(8.0);

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
				ui.heading("Standalone Scripts");
				ui.separator();
				ui.add_space(8.0);

				match help_string!(ui, "scripts") {
					Some(0) => {
						self.open(SidePanelKind::FilterScripts);
						return;
					}
					Some(1) => {
						self.open(SidePanelKind::SortingScripts);
						return;
					}
					_ => {}
				}

				match help_string!(ui, "scripts_2") {
					Some(0) => {
						ui.output_mut(|o| {
							o.open_url = Some(egui::OpenUrl::new_tab("https://github.com/potassium-shot/adhd-materia"));
						});
					}
					_ => {}
				}

				ui.add_space(8.0);

				show_scripts(
					ui,
					script_list,
					"Standalone",
					&DEFAULT_STANDALONE_SCRIPT,
					interactable,
				);
			}
			Self::CompletedTasks { total_completed_tasks } => {
				ui.horizontal(|ui| {
					ui.heading("Total Completed Tasks:");
					ui.heading(egui::RichText::new(total_completed_tasks.to_string()).color(Settings::get().theme.get_catppuccin().green))
				});
				ui.separator();
				ui.add_space(8.0);

				match help_string!(ui, "sprint_list") {
					Some(0) => {
						self.open(SidePanelKind::Settings);
						return;
					},
					_ => {}
				}

				ui.add_space(8.0);
				
				let sprint_list = &Session::current().past_done_counters;

				egui::ScrollArea::vertical().show_rows(ui, 16.0, sprint_list.len(), |ui, range| {
					egui::Grid::new("sprint_list")
						.num_columns(1)
						.striped(true)
						.spacing((40.0, 4.0))
						.show(ui, |ui| {
							let range_start = range.start;

							for (i, sprint) in sprint_list[range].iter().copied().enumerate() {
								ui.vertical_centered_justified(|ui| {
									ui.label(format!("Sprint {} - {} tasks completed", sprint_list.len() - (range_start + i), sprint));
								});
								ui.end_row();
							}
						});
				});
			}
			Self::Settings { color_associations_cache } => {
				ui.heading("Settings");
				ui.separator();
				ui.add_space(8.0);

				help_string!(ui, "settings");

				ui.add_space(8.0);

				egui::Grid::new("settings_list")
					.num_columns(2)
					.spacing((40.0, 8.0))
					.show(ui, |ui| {
						let mut settings = Settings::get();

						ui.label("Help messages").on_hover_ui(|ui| {
							ui.horizontal(|ui| {
								ui.label("Shows");
								ui.colored_label(settings.theme.get_catppuccin().blue, "ℹ help messages");
							});
						});
						ui.add(egui::Checkbox::without_text(&mut settings.help_messages));
						ui.end_row();

						ui.label("Theme").on_hover_text(
							"Selects the theme. Currently only Catppuccin themes are hardcoded, but custom themes should be supported in the future."
						);

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

						ui.label("Default task").on_hover_text("Newly created tasks will look like this");
						settings.default_task.widget().show(ui, task_names, false, scroll_to, selected_task);
						ui.end_row();

						ui.label("Repeating task rewind").on_hover_text(
							"When a repeating task should have triggered more than once, \
							One will make it trigger once, and All will make it trigger all times",
						);
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
							});
						ui.end_row();

						ui.label("Scheduled task tag").on_hover_text("Tag to apply to scheduled tasks, $DATE is replaced by the scheduled date");

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
							);

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

						ui.label("Delete used scheduled tasks").on_hover_text(
							"Delete scheduled tasks that have been used and are never to be triggered again."
						);
						ui.add(egui::Checkbox::without_text(
							&mut settings.delete_used_scheduled_tasks,
						));
						ui.end_row();

						ui.horizontal(|ui| {
							ui.label("Date format,");
							if ui.link(egui::RichText::new("see reference").color(ui.visuals().hyperlink_color)).clicked() {
								ui.output_mut(|o| {
									o.open_url = Some(egui::OpenUrl::new_tab("https://docs.rs/chrono/latest/chrono/format/strftime/index.html"));
								});
							}
						});
						ui.text_edit_singleline(&mut settings.date_format);
						ui.end_row();

						ui.label("Sprints Start").on_hover_text("Sprint start/reference date.");
						ui.add(egui_extras::DatePickerButton::new(&mut settings.sprint_end_reference));
						ui.end_row();

						ui.label("Sprint Frequency").on_hover_text("Frequency of sprints. Repeats start from the Sprint Start date.");

						let mut freq = settings.sprint_end.kind();

						egui::ComboBox::new("sprint_end_freq", "")
							.selected_text(format!("{}", freq))
							.show_ui(ui, |ui| {
								ui.selectable_value(
									&mut freq,
									SprintFrequencyKind::Weekly,
									"Weekly",
								);
								ui.selectable_value(
									&mut freq,
									SprintFrequencyKind::TwoWeekly,
									"Two Weekly",
								);
								ui.selectable_value(
									&mut freq,
									SprintFrequencyKind::Monthly,
									"Monthly",
								);
								ui.selectable_value(
									&mut freq,
									SprintFrequencyKind::Custom,
									"Custom",
								);
							});
						
						settings.sprint_end = match freq {
								SprintFrequencyKind::Weekly => SprintFrequency::Weekly,
								SprintFrequencyKind::TwoWeekly => SprintFrequency::TwoWeekly,
								SprintFrequencyKind::Monthly => SprintFrequency::Monthly,
								SprintFrequencyKind::Custom => {
									let mut days = if let SprintFrequency::Custom { days } = settings.sprint_end {
										days
									} else {
										7
									};

									ui.end_row();
									ui.label("Sprint Frequency Days");
									ui.add(egui::DragValue::new(&mut days));

									SprintFrequency::Custom {
										days,
									}
								}
							};

						ui.end_row();

						ui.label("Name colors").on_hover_text(
							"Associates a color to tag/filter/sorting names. Names that aren't in here will have a random color."
						);
						ui.collapsing("Associations", |ui| {
							egui::Grid::new("color_assoc")
								.striped(true)
								.num_columns(2)
								.show(ui, |ui| {
									let mut to_remove = Vec::new();

									for (i, (name, color)) in color_associations_cache.iter_mut().enumerate() {
										ui.text_edit_singleline(name);
										ui.color_edit_button_hsva(color);
										
										if ui.button("X").clicked() {
											to_remove.push(i);
										}

										ui.end_row();
									}

									for i in to_remove.into_iter().rev() {
										color_associations_cache.remove(i);
									}
								});

							ui.vertical_centered_justified(|ui| {
								if ui.button("+").clicked() {
									color_associations_cache.push((String::from("name"), egui::epaint::Hsva::new(0.0, 0.8, 0.8, 1.0)));
								}
							});
						});
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
			SidePanelKind::CompletedTasks => {
				let session = Session::current();
				Self::CompletedTasks {
					total_completed_tasks: session.past_done_counters.iter().copied().sum::<i32>() + session.current_done_counter,
				}
			},
			SidePanelKind::Settings => Self::Settings {
				color_associations_cache: Settings::get().color_associations.iter().map(|(name, color)| (name.clone(), egui::epaint::Hsva::from_srgba_premultiplied(color.to_array()))).collect(),
			},
			SidePanelKind::Hidden => Self::Hidden,
		}
	}

	fn close(&mut self) {
		match self {
			Self::Hidden => {}
			Self::ScheduledTasks {
				scheduled_task_list,
				..
			} => {
				if let Ok(list) = scheduled_task_list {
					list.save_all();
				}
			}
			Self::FilterScripts { script_list, .. } => {
				close_scripts(script_list, "Filter");
			}
			Self::SortingScripts { script_list, .. } => {
				close_scripts(script_list, "Sorting");
			}
			Self::Scripts { script_list, .. } => {
				close_scripts(script_list, "Standalone");
			}
			Self::CompletedTasks { .. } => {}
			Self::Settings { color_associations_cache } => {
				let mut settings = Settings::get();
				settings.default_task.apply_tags();

				settings.color_associations = color_associations_cache.iter_mut().map(|(name, color)| {
					let rgb = color.to_srgb();
					(name.clone(), egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]))
				}).collect();

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

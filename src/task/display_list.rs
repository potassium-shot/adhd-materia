use uuid::Uuid;

use crate::scripts::{filter::FilterList, value::AnyIntoPocketPyValue, PocketPyScript};

use super::{list::TaskList, Task};

pub struct TaskDisplayList {
	tasks: Vec<Uuid>,
}

impl TaskDisplayList {
	pub fn new(task_list: &TaskList, filter_list: &FilterList) -> Self {
		let task_list: Vec<&Task> = task_list.tasks().collect();
		let mut task_passes = vec![true; task_list.len()];

		match crate::data_dir() {
			Ok(data_dir) => {
				for passes in filter_list
					.iter()
					.filter_map(
						|(filter_script, enabled)| {
							if enabled {
								Some(filter_script)
							} else {
								None
							}
						},
					)
					.filter_map(|filter_script_name| {
						match PocketPyScript::load(
							data_dir
								.filter_scripts()
								.join(filter_script_name)
								.with_extension("py"),
						) {
							Ok(script) => match script.execute_function_for::<bool>(
								crate::app::script_lock(),
								script.name.as_str(),
								[(
									"task",
									task_list
										.iter()
										.map(|task| Box::new((*task).clone()) as AnyIntoPocketPyValue)
										.collect(),
								)],
							) {
								Ok(passes) => Some(passes),
								Err(e) => {
									eprintln!("Error in filter script:\n{}", e);
									None
								}
							},
							Err(_) => None,
						}
					}) {
					task_passes
						.iter_mut()
						.zip(passes)
						.for_each(|(a, b)| *a &= b);
				}
			}
			Err(_) => {}
		}

		Self {
			tasks: task_list
				.into_iter()
				.zip(task_passes)
				.filter_map(|(task, pass)| {
					if pass {
						Some(task.get_uuid().clone())
					} else {
						None
					}
				})
				.collect(),
		}
	}

	pub fn tasks(&self) -> impl Iterator<Item = &Uuid> {
		self.tasks.iter()
	}
}

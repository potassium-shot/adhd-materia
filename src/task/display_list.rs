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
							Ok(script) => script
								.execute_function_for::<bool>(
									crate::app::script_lock(),
									script.name.as_str(),
									[(
										"test",
										(0..task_list.len())
											.map(|_| Box::new(0i64) as AnyIntoPocketPyValue)
											.collect(),
									)],
								)
								.ok(),
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

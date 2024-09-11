use uuid::Uuid;

use crate::scripts::{
	filter::FilterList, sorting::SortingList, value::AnyIntoPocketPyValue, PocketPyScript,
};

use super::{list::TaskList, Task};

pub struct TaskDisplayList {
	tasks: Vec<Uuid>,
}

impl TaskDisplayList {
	pub fn new(task_list: &TaskList, filter_list: &FilterList, sorting_list: &SortingList) -> Self {
		let task_list: Vec<&Task> = task_list.tasks().collect();
		let mut task_passes = vec![true; task_list.len()];
		let mut task_orderings: Option<Vec<i64>> = None;

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
										.map(|task| {
											Box::new((*task).clone()) as AnyIntoPocketPyValue
										})
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

				if let Some((sorting, _)) = sorting_list.iter().find(|s| s.1) {
					if let Some(orderings) = match PocketPyScript::load(
						data_dir
							.sorting_scripts()
							.join(sorting)
							.with_extension("py"),
					) {
						Ok(script) => match script.execute_function_for::<i64>(
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
							Ok(orderings) => Some(orderings),
							Err(e) => {
								eprintln!("Error in sorting script:\n{}", e);
								None
							}
						},
						Err(_) => None,
					} {
						task_orderings = Some(orderings);
					}
				}
			}
			Err(_) => {}
		}

		let task_orderings = if let Some(orderings) = task_orderings {
			orderings
		} else {
			vec![0; task_list.len()]
		};

		let mut tasks: Vec<(Uuid, i64)> = task_list
			.into_iter()
			.zip(task_passes)
			.zip(task_orderings)
			.filter_map(|((task, pass), ord)| {
				if pass {
					Some((task.get_uuid().clone(), ord))
				} else {
					None
				}
			})
			.collect();

		tasks.sort_by_key(|(_, ord)| *ord);

		Self { tasks: tasks.into_iter().map(|(task, _)| task).collect() }
	}

	pub fn tasks(&self) -> impl Iterator<Item = &Uuid> {
		self.tasks.iter()
	}
}

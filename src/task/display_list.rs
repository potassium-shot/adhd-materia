use std::cmp::Ordering;

use uuid::Uuid;

use crate::{scripts::{
	filter::FilterList, sorting::SortingList, value::AnyIntoPocketPyValue, PocketPyScript,
}, toast_error};

use super::{list::TaskList, Task};

pub struct TaskDisplayList {
	tasks: Vec<Uuid>,
}

impl TaskDisplayList {
	pub fn new(
		task_list: &TaskList,
		filter_list: &FilterList,
		sorting_list: &SortingList,
		parent_task: Option<Uuid>,
	) -> Self {
		let task_list: Vec<&Task> = task_list.tasks().collect();
		let mut task_passes = vec![true; task_list.len()];
		let mut task_orderings: Vec<Vec<i64>> = Vec::new();

		match crate::data_dir() {
			Ok(data_dir) => {
				for passes in filter_list.iter_set().filter_map(|filter_script_name| {
					match PocketPyScript::load(
						data_dir
							.filter_scripts()
							.join(filter_script_name)
							.with_extension("py"),
					) {
						Ok(script) => match script.execute_function_for::<bool>(
							crate::app::script_lock(),
							script.name.as_str(),
							[
								(
									"task",
									task_list
										.iter()
										.map(|task| {
											Box::new((*task).clone()) as AnyIntoPocketPyValue
										})
										.collect(),
								),
								(
									"parent",
									match parent_task {
										Some(parent_task) => std::iter::once(parent_task)
											.cycle()
											.take(task_list.len())
											.map(|u| Box::new(u) as AnyIntoPocketPyValue)
											.collect(),
										_ => std::iter::once(())
											.cycle()
											.take(task_list.len())
											.map(|none| Box::new(none) as AnyIntoPocketPyValue)
											.collect(),
									},
								),
							],
						) {
							Ok(passes) => Some(passes),
							Err(e) => {
								toast_error!("Error in filter script:\n{}", e);
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

				for ordering in sorting_list.iter_set().filter_map(|sorting_script_name| {
					match PocketPyScript::load(
						data_dir
							.sorting_scripts()
							.join(sorting_script_name)
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
								toast_error!("Error in sorting script:\n{}", e);
								None
							}
						},
						Err(_) => None,
					}
				}) {
					task_orderings.push(ordering);
				}
			}
			Err(_) => {}
		}

		let mut tasks: Vec<(Uuid, Vec<i64>)> = task_list
			.into_iter()
			.zip(task_passes)
			.enumerate()
			.filter_map(|(idx, (task, pass))| {
				if pass {
					Some((
						task.get_uuid().clone(),
						task_orderings.iter().map(|o| o[idx]).collect::<Vec<i64>>(),
					))
				} else {
					None
				}
			})
			.collect();

		tasks.sort_unstable_by(|(_, orderings_a), (_, orderings_b)| {
			let mut final_ordering = Ordering::Equal;

			for (a, b) in orderings_a.iter().zip(orderings_b.iter()) {
				match a.cmp(b) {
					Ordering::Equal => {}
					Ordering::Less => {
						final_ordering = Ordering::Less;
						break;
					}
					Ordering::Greater => {
						final_ordering = Ordering::Greater;
						break;
					}
				}
			}

			final_ordering
		});

		Self {
			tasks: tasks.into_iter().map(|(task, _)| task).collect(),
		}
	}

	pub fn tasks(&self) -> impl Iterator<Item = &Uuid> {
		self.tasks.iter()
	}
}

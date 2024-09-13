use std::{collections::HashMap, time::Duration};

use uuid::Uuid;

use crate::{data_dir::DataDirError, tag::TagValue};

use super::{NormalTaskData, Task, TaskError, TaskPath, TaskTypeData};

pub struct TaskList<T = NormalTaskData> {
	tasks: HashMap<Uuid, Task<T>>,
	path: TaskPath,
}

impl<T: TaskTypeData> TaskList<T> {
	pub fn new(path: TaskPath) -> Result<(Self, Vec<TaskError>), TaskListError> {
		let (tasks, errors) = std::fs::read_dir(path.get_path()?)?.fold(
			(HashMap::<Uuid, Task<T>>::new(), Vec::<TaskError>::new()),
			|(mut tasks, mut errors), entry| {
				let result: Result<(), TaskError> = (|| {
					let entry = entry?;

					if entry.metadata()?.is_file() {
						let task = Task::<T>::load_from_name(
							entry.file_name().to_string_lossy().to_string(),
							path,
						)?;
						tasks.insert(task.uuid.clone(), task);
					}

					Ok(())
				})();

				if let Err(e) = result {
					errors.push(e);
				}

				(tasks, errors)
			},
		);

		Ok((Self { tasks, path }, errors))
	}

	pub fn tasks(&self) -> impl Iterator<Item = &Task<T>> {
		self.tasks.values()
	}

	pub fn tasks_mut(&mut self) -> impl Iterator<Item = &mut Task<T>> {
		self.tasks.values_mut()
	}

	pub fn get_mut(&mut self, id: &Uuid) -> Option<&mut Task<T>> {
		self.tasks.get_mut(id)
	}

	pub fn add_task(&mut self, task: Task<T>) -> Result<(), TaskError> {
		task.save(self.path)?;
		self.tasks.insert(task.uuid, task);
		Ok(())
	}

	pub fn delete_task(&mut self, uuid: &Uuid) -> Result<(), TaskError> {
		if let Some(task) = self.tasks.remove(uuid) {
			task.delete(self.path)?;
		}

		let mut to_delete = Vec::new();

		for (other_uuid, task) in self.tasks.iter_mut() {
			let mut subtask_tag_count = 0;
			let mut delete_i = None;

			task.tags.iter().enumerate().for_each(|(i, tag)| {
				if tag.name.as_str() == "subtask_of" {
					subtask_tag_count += 1;

					if tag.value == Some(TagValue::TaskReference(*uuid)) {
						delete_i = Some(i);
					}
				}
			});

			if let Some(i) = delete_i {
				task.tags.remove(i);

				if subtask_tag_count == 1 {
					to_delete.push(other_uuid.clone());
				}
			}
		}

		for uuid in to_delete {
			self.delete_task(&uuid)?;
		}

		Ok(())
	}

	pub fn cleanup_marked_for_delete(&mut self) -> (bool, Vec<TaskError>) {
		let to_delete: Vec<Uuid> = self
			.tasks
			.iter()
			.filter_map(|(uuid, task)| {
				if task.marked_for_delete {
					Some(uuid.clone())
				} else {
					None
				}
			})
			.collect();

		let anything = !to_delete.is_empty();

		let mut error_list = Vec::new();

		for uuid in to_delete {
			if let Err(e) = self.delete_task(&uuid) {
				error_list.push(e);
			}
		}

		(anything, error_list)
	}

	pub fn save_all(&self) {
		for task in self.tasks.values() {
			if let Err(e) = task.save(self.path) {
				crate::toasts()
					.error(format!("Could not save task: {}", e))
					.set_closable(true)
					.set_duration(Some(Duration::from_millis(10_000)));
			}
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum TaskListError {
	#[error("Could not access data directory: {0}")]
	DataDirError(
		#[from]
		#[source]
		&'static DataDirError,
	),

	#[error("IO error: {0}")]
	IOError(
		#[from]
		#[source]
		std::io::Error,
	),
}

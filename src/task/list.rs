use std::collections::{hash_map, HashMap};

use uuid::Uuid;

use crate::data_dir::DataDirError;

use super::{Task, TaskError};

pub struct TaskList {
	tasks: HashMap<Uuid, Task>,
}

impl TaskList {
	pub fn new() -> Result<(Self, Vec<TaskError>), TaskListError> {
		let (tasks, errors) = std::fs::read_dir(crate::data_dir()?.tasks())?.fold(
			(HashMap::<Uuid, Task>::new(), Vec::<TaskError>::new()),
			|(mut tasks, mut errors), entry| {
				let result: Result<(), TaskError> = (|| {
					let entry = entry?;

					if entry.metadata()?.is_file() {
						let task =
							Task::load_from_name(entry.file_name().to_string_lossy().to_string())?;
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

		Ok((Self { tasks }, errors))
	}

	pub fn tasks(&self) -> hash_map::Values<Uuid, Task> {
		self.tasks.values()
	}

	pub fn tasks_mut(&mut self) -> hash_map::ValuesMut<Uuid, Task> {
		self.tasks.values_mut()
	}

	pub fn new_task(&mut self) -> Result<(), TaskError> {
		let task = Task::default();
		task.save()?;
		self.tasks.insert(task.uuid, task);
		Ok(())
	}

	pub fn delete_task(&mut self, uuid: &Uuid) -> Result<(), TaskError> {
		if let Some(task) = self.tasks.remove(uuid) {
			task.delete()?;
		}

		Ok(())
	}

	pub fn cleanup_marked_for_delete(&mut self) -> Vec<TaskError> {
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

		let mut error_list = Vec::new();

		for uuid in to_delete {
			if let Err(e) = self.delete_task(&uuid) {
				error_list.push(e);
			}
		}

		error_list
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

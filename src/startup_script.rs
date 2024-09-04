use crate::{
	session::{Session, SessionError}, settings::Settings, task::{
		list::{TaskList, TaskListError},
		scheduled::ScheduledTask,
		NormalTaskData, TaskError, TaskPath,
	}
};

pub struct StartupScript;

impl StartupScript {
	pub fn run() -> Result<Vec<TaskError>, StartupError> {
		let (scheduled_task_list, mut errors) =
			TaskList::<ScheduledTask>::new(TaskPath::Scheduled)?;
		let (mut task_list, _) = TaskList::<NormalTaskData>::new(TaskPath::Tasks)?;

		for task in scheduled_task_list.get_tasks() {
			let spawn_count = match Settings::get().repeatable_rewind {
				crate::settings::RepeatableRewind::One => i32::min(task.type_data.spawn_count(), 0),
				crate::settings::RepeatableRewind::All => task.type_data.spawn_count(),
			};

			for _ in 0..spawn_count {
				if let Err(e) = task_list.add_task(task.clone().convert(NormalTaskData::default()))
				{
					errors.push(e);
				}
			}
		}

		Session::mutate(|session| session.last_session = chrono::Local::now().date_naive())?;
		Ok(errors)
	}
}

#[derive(Debug, thiserror::Error)]
pub enum StartupError {
	#[error("Could not load task list: {0}")]
	TaskListError(
		#[from]
		#[source]
		TaskListError,
	),

	#[error("Could not write session, this means scheduled tasks will trigger every launch! {0}")]
	SessionError(
		#[from]
		#[source]
		SessionError,
	),
}

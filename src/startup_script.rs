use crate::{
	session::{Session, SessionError},
	task::{
		list::{TaskList, TaskListError},
		scheduled::ScheduledTask,
		NormalTaskData, TaskError, TaskPath,
	},
};

pub struct StartupScript;

impl StartupScript {
	pub fn run() -> Result<Vec<TaskError>, StartupError> {
		let (scheduled_task_list, mut errors) =
			TaskList::<ScheduledTask>::new(TaskPath::Scheduled)?;
		let (mut task_list, _) = TaskList::<NormalTaskData>::new(TaskPath::Tasks)?;

		for task in scheduled_task_list.get_tasks() {
			for _ in 0..task.type_data.spawn_count() {
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

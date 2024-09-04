use std::str::FromStr;

use crate::{
	session::{Session, SessionError},
	settings::Settings,
	tag::Tag,
	task::{
		list::{TaskList, TaskListError},
		scheduled::{RepeatMode, ScheduledTask},
		NormalTaskData, TaskError, TaskPath,
	},
};

pub struct StartupScript;

impl StartupScript {
	pub fn run() -> Result<Vec<TaskError>, StartupError> {
		let (mut scheduled_task_list, mut errors) =
			TaskList::<ScheduledTask>::new(TaskPath::Scheduled)?;
		let (mut task_list, _) = TaskList::<NormalTaskData>::new(TaskPath::Tasks)?;

		for task in scheduled_task_list.get_tasks_mut() {
			let spawn_count = match Settings::get().repeatable_rewind {
				crate::settings::RepeatableRewind::One => i32::min(task.type_data.spawn_count(), 1),
				crate::settings::RepeatableRewind::All => task.type_data.spawn_count(),
			};

			let today = chrono::Local::now().date_naive();

			for _ in 0..spawn_count {
				let mut new_task = task.clone().convert(NormalTaskData::default());
				new_task.new_uuid();

				if let Ok(tag) = Tag::from_str(
					&Settings::get()
						.scheduled_task_tag
						.replace("$DATE", today.to_string().as_str()),
				) {
					new_task.tags.push(tag);
				}

				if let Err(e) = task_list.add_task(new_task) {
					errors.push(e);
				}
			}

			if spawn_count > 0 && task.type_data.repeat_mode == RepeatMode::Never {
				task.mark_for_delete();
			}
		}

		scheduled_task_list.cleanup_marked_for_delete();

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

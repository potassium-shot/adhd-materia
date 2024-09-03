use std::{path::Path, str::FromStr, time::Duration};

use ui::TaskWidget;
use uuid::Uuid;

use crate::{data_dir::DataDirError, tag::Tag};

pub mod list;
pub mod scheduled;
mod ui;

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Task<T = NormalTaskData> {
	#[serde(skip)]
	uuid: Uuid,

	pub name: String,
	pub description: String,
	pub tags: Vec<Tag>,

	pub type_data: T,

	#[serde(skip)]
	state: TaskState,
	#[serde(skip)]
	marked_for_delete: bool,
}

impl<T: TaskTypeData> Default for Task<T> {
	fn default() -> Self {
		Self {
			uuid: Uuid::new_v4(),

			name: String::from("Unnamed"),
			description: String::new(),
			tags: Vec::new(),

			type_data: T::default(),

			state: TaskState::Display,
			marked_for_delete: false,
		}
	}
}

impl<T: TaskTypeData> FromStr for Task<T> {
	type Err = TaskErrorKind;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(ron::from_str(s)?)
	}
}

impl<T: TaskTypeData> Task<T> {
	pub fn get_uuid(&self) -> &Uuid {
		&self.uuid
	}

	pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, TaskErrorKind> {
		let path = path.as_ref();
		let mut result = Self::from_str(std::fs::read_to_string(path)?.as_str())?;
		result.uuid = Uuid::from_str(
			path.file_name()
				.expect("if no filename, should have failed earlier")
				.to_string_lossy()
				.as_ref(),
		)?;
		Ok(result)
	}

	pub fn load_from_name(name: impl AsRef<str>, path: TaskPath) -> Result<Self, TaskError> {
		let name = name.as_ref();
		let result = (|| (Self::load_from_path(path.get_path()?.join(name))))();
		result.map_err(|e| TaskError {
			task_name: name.to_owned(),
			error_kind: e,
		})
	}

	pub fn load(uuid: Uuid, path: TaskPath) -> Result<Self, TaskError> {
		Self::load_from_name(uuid.to_string(), path)
	}

	pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<(), TaskError> {
		Ok(std::fs::write(
			path,
			ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::new())
				.expect("Task ron serialization should not fail"),
		)
		.map_err(|e| TaskError {
			task_name: self.uuid.to_string(),
			error_kind: e.into(),
		})?)
	}

	pub fn save(&self, path: TaskPath) -> Result<(), TaskError> {
		let name = self.uuid.to_string();

		self.save_to_path(Self::get_data_dir(&name, path)?.join(name))
	}

	fn delete(&self, path: TaskPath) -> Result<(), TaskError> {
		let name = self.uuid.to_string();

		Ok(std::fs::remove_file(
			Self::get_data_dir(&name, path)?.join(name),
		)?)
	}

	pub fn mark_for_delete(&mut self) {
		self.marked_for_delete = true;
	}

	fn get_data_dir(name: impl ToString, path: TaskPath) -> Result<&'static Path, TaskError> {
		path.get_path().map_err(|e| TaskError {
			task_name: name.to_string(),
			error_kind: e.into(),
		})
	}

	pub fn display(&mut self, path: TaskPath) {
		for tag in &mut self.tags {
			if let Err(e) = tag.apply_text() {
				crate::toasts()
					.error(format!("Tag parsing error: {}", e))
					.set_closable(true)
					.set_duration(Some(Duration::from_millis(10_000)));
			}
		}

		if let Err(e) = self.save(path) {
			crate::toasts()
				.error(format!("Could not save task: {}", e))
				.set_closable(true)
				.set_duration(Some(Duration::from_millis(10_000)));
		}

		self.state = TaskState::Display;
	}

	pub fn edit(&mut self) {
		self.state = TaskState::Edit {
			pending_delete: false,
		};
	}

	pub fn is_pending_delete(&self) -> bool {
		match self.state {
			TaskState::Edit { pending_delete } => pending_delete,
			_ => false,
		}
	}

	pub fn widget(&mut self) -> TaskWidget<T> {
		TaskWidget::new(self)
	}

	pub fn convert<NewT: TaskTypeData>(self, type_data: NewT) -> Task<NewT> {
		Task {
			type_data,
			uuid: self.uuid,
			name: self.name,
			description: self.description,
			tags: self.tags,
			state: self.state,
			marked_for_delete: self.marked_for_delete,
		}
	}
}

#[derive(Debug, Eq, PartialEq, Clone, Default)]
enum TaskState {
	#[default]
	Display,
	Edit {
		pending_delete: bool,
	},
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskPath {
	Tasks,
	Scheduled,
}

impl TaskPath {
	fn get_path(&self) -> Result<&'static Path, &'static DataDirError> {
		Ok(match self {
			Self::Tasks => crate::data_dir()?.tasks(),
			Self::Scheduled => crate::data_dir()?.scheduled(),
		})
	}
}

pub trait TaskTypeData:
	std::fmt::Debug + PartialEq + Default + serde::Serialize + serde::de::DeserializeOwned
{
}

#[derive(Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct NormalTaskData;

impl TaskTypeData for NormalTaskData {}

#[derive(Debug, thiserror::Error)]
#[error("Task {task_name}: {error_kind}")]
pub struct TaskError {
	pub task_name: String,
	pub error_kind: TaskErrorKind,
}

#[derive(Debug, thiserror::Error)]
pub enum TaskErrorKind {
	#[error("Parse error: {0}")]
	ParseError(
		#[from]
		#[source]
		ron::error::SpannedError,
	),

	#[error("IO Error: {0}")]
	IOError(
		#[from]
		#[source]
		std::io::Error,
	),

	#[error("Invalid data directory: {0}")]
	InvalidDataDir(
		#[from]
		#[source]
		&'static DataDirError,
	),

	#[error("Invalid uuid filename: {0}")]
	InvalidUuid(
		#[from]
		#[source]
		uuid::Error,
	),
}

impl<T: Into<TaskErrorKind>> From<T> for TaskError {
	fn from(value: T) -> Self {
		Self {
			task_name: String::from("<unknown>"),
			error_kind: value.into(),
		}
	}
}

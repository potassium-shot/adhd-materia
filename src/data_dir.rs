use std::path::{Path, PathBuf};

pub struct DataDir {
	tasks_dir: PathBuf,
	scheduled_dir: PathBuf,
}

impl DataDir {
	pub fn new() -> Result<Self, DataDirError> {
		let dir = directories::ProjectDirs::from("", "", "adhd-materia")
			.ok_or(DataDirError::NoHomeDirectory)?
			.data_dir()
			.to_path_buf();

		let tasks_dir = dir.join("tasks");
		let scheduled_dir = dir.join("scheduled");

		std::fs::create_dir_all(&tasks_dir)?;
		std::fs::create_dir_all(&scheduled_dir)?;
		Ok(Self {
			tasks_dir,
			scheduled_dir,
		})
	}

	pub fn tasks(&self) -> &Path {
		self.tasks_dir.as_path()
	}

	pub fn scheduled(&self) -> &Path {
		self.scheduled_dir.as_path()
	}
}

#[derive(Debug, thiserror::Error)]
pub enum DataDirError {
	#[error("No home directory was found.")]
	NoHomeDirectory,

	#[error("IO Error: {0}")]
	IOError(
		#[from]
		#[source]
		std::io::Error,
	),
}

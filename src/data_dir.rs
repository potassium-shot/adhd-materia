use std::path::{Path, PathBuf};

pub struct DataDir {
	tasks_dir: PathBuf,
	scheduled_dir: PathBuf,
	session_file: PathBuf,
	settings_file: PathBuf,
}

impl DataDir {
	pub fn new() -> Result<Self, DataDirError> {
		let dir = directories::ProjectDirs::from("", "", "adhd-materia")
			.ok_or(DataDirError::NoHomeDirectory)?
			.data_dir()
			.to_path_buf();

		let tasks_dir = dir.join("tasks");
		let scheduled_dir = dir.join("scheduled");
		let session_file = dir.join("session.ron");
		let settings_file = dir.join("settings.ron");

		std::fs::create_dir_all(&tasks_dir)?;
		std::fs::create_dir_all(&scheduled_dir)?;
		Ok(Self {
			tasks_dir,
			scheduled_dir,
			session_file,
			settings_file,
		})
	}

	pub fn tasks(&self) -> &Path {
		self.tasks_dir.as_path()
	}

	pub fn scheduled(&self) -> &Path {
		self.scheduled_dir.as_path()
	}

	pub fn session(&self) -> &Path {
		self.session_file.as_path()
	}

	pub fn settings(&self) -> &Path {
		self.settings_file.as_path()
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

use std::path::{Path, PathBuf};

pub struct DataDir {
	tasks_dir: PathBuf,
}

impl DataDir {
	pub fn new() -> Result<Self, DataDirError> {
		let tasks_dir = directories::ProjectDirs::from("", "", "adhd-materia")
			.ok_or(DataDirError::NoHomeDirectory)?
			.data_dir()
			.join("tasks");
		std::fs::create_dir_all(&tasks_dir)?;
		Ok(Self { tasks_dir })
	}

	pub fn tasks(&self) -> &Path {
		self.tasks_dir.as_path()
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

use std::{
	str::FromStr,
	sync::{LazyLock, Mutex, MutexGuard},
};

use crate::data_dir::DataDirError;

static SETTINGS: LazyLock<Mutex<Settings>> =
	LazyLock::new(|| Mutex::new(Settings::load().unwrap_or_default()));

#[derive(Debug, PartialEq, Eq, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Settings {
	repeatable_rewind: RepeatableRewind,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub enum RepeatableRewind {
	#[default]
	One,
	All,
}

impl Settings {
	pub fn load() -> Result<Self, SettingsError> {
		Ok(std::fs::read_to_string(crate::data_dir()?.settings())?.parse()?)
	}

	pub fn save(&self) -> Result<(), SettingsError> {
		std::fs::write(crate::data_dir()?.settings(), self.to_string())?;
		Ok(())
	}

	pub fn get() -> MutexGuard<'static, Self> {
		SETTINGS.lock().expect("Settings should be lockable")
	}
}

impl FromStr for Settings {
	type Err = ron::error::SpannedError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		ron::from_str(s)
	}
}

impl ToString for Settings {
	fn to_string(&self) -> String {
		ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
			.expect("settings ron serialization should never fail")
	}
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
	#[error("IO Error: {0}")]
	IoError(
		#[from]
		#[source]
		std::io::Error,
	),

	#[error("Could not access data directory: {0}")]
	DataDirError(
		#[from]
		#[source]
		&'static DataDirError,
	),

	#[error("Could not parse the settings file: {0}")]
	ParseError(
		#[from]
		#[source]
		ron::error::SpannedError,
	),
}

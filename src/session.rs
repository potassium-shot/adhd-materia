use std::{
	str::FromStr,
	sync::{LazyLock, Mutex},
};

use crate::data_dir::DataDirError;

static SESSION: LazyLock<Mutex<Session>> =
	LazyLock::new(|| Mutex::new(Session::load().unwrap_or_default()));

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Session {
	pub last_session: chrono::NaiveDate,
}

impl Session {
	pub fn load() -> Result<Self, SessionError> {
		Ok(std::fs::read_to_string(crate::data_dir()?.session())?.parse()?)
	}

	pub fn save(&self) -> Result<(), SessionError> {
		std::fs::write(crate::data_dir()?.session(), self.to_string())?;

		Ok(())
	}

	pub fn mutate(f: impl FnOnce(&mut Self)) -> Result<(), SessionError> {
		let mut session = SESSION.lock().expect("session should be lockable");
		f(&mut session);
		session.save()?;

		Ok(())
	}
}

impl FromStr for Session {
	type Err = ron::error::SpannedError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		ron::from_str(s)
	}
}

impl ToString for Session {
	fn to_string(&self) -> String {
		ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
			.expect("ron serialization of Session should not fail")
	}
}

impl Default for Session {
	fn default() -> Self {
		Self {
			last_session: chrono::Local::now().date_naive(),
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
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

	#[error("Parse error: {0}")]
	ParseError(
		#[from]
		#[source]
		ron::error::SpannedError,
	),
}

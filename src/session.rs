use std::{
	str::FromStr,
	sync::{LazyLock, Mutex, MutexGuard},
};

use crate::data_dir::DataDirError;

static SESSION: LazyLock<Mutex<Session>> =
	LazyLock::new(|| Mutex::new(Session::load().unwrap_or_default()));

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Session {
	pub last_session: chrono::NaiveDate,
	pub set_filters: Vec<String>,
	pub set_sortings: Vec<String>,
	pub current_done_counter: i32,
	pub past_done_counters: Vec<i32>,
}

impl Default for Session {
	fn default() -> Self {
		Self {
			last_session: chrono::Local::now().date_naive(),
			set_filters: Vec::new(),
			set_sortings: Vec::new(),
			current_done_counter: 0,
			past_done_counters: Vec::new(),
		}
	}
}

impl Session {
	pub fn load() -> Result<Self, SessionError> {
		let data_dir = crate::data_dir()?;
		let session_path = data_dir.session();

		if !session_path.exists() {
			let mut session = Self::default();

			// Write default filters/sortings
			std::fs::write(
				data_dir.filter_scripts().join("default_filter.py"),
				include_str!("../assets/filter_default.py"),
			)?;
			std::fs::write(
				data_dir.filter_scripts().join("only_undone.py"),
				include_str!("../assets/filter_only_undone.py"),
			)?;
			std::fs::write(
				data_dir.sorting_scripts().join("by_priority.py"),
				include_str!("../assets/sorting_by_priority.py"),
			)?;

			// Set some of these
			session.set_filters.push(String::from("default_filter"));
			session.set_sortings.push(String::from("by_priority"));

			return Ok(session);
		}

		Ok(std::fs::read_to_string(session_path)?.parse()?)
	}

	pub fn save(&self) -> Result<(), SessionError> {
		std::fs::write(crate::data_dir()?.session(), self.to_string())?;

		Ok(())
	}

	pub fn current() -> MutexGuard<'static, Self> {
		SESSION.lock().expect("session should be lockable")
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

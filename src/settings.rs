use std::{
	str::FromStr,
	sync::{LazyLock, Mutex, MutexGuard, RwLock, RwLockReadGuard},
};

use convert_case::Casing;

use crate::{data_dir::DataDirError, tag::Tag, task::Task};

pub const DEFAULT_SCHEDULED_TASK_TAG: &str = "scheduled_on($DATE)";
pub const DEFAULT_DATE_FORMAT: &str = "%a. %-d %b. %Y";
pub const DEFAULT_DONE_TAG: &str = "done";

static SETTINGS: LazyLock<Mutex<Settings>> =
	LazyLock::new(|| Mutex::new(Settings::load().unwrap_or_default()));

static DONE_TAG: LazyLock<RwLock<Tag>> = LazyLock::new(|| {
	RwLock::new(
		Tag::from_str(&Settings::get().done_tag)
			.or(Tag::from_str(DEFAULT_DONE_TAG))
			.expect("Default done tag should be valid"),
	)
});

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Settings {
	pub theme: AdhdMateriaTheme,
	pub default_task: Task,
	pub repeatable_rewind: RepeatableRewind,
	pub scheduled_task_tag: Option<String>,
	pub delete_used_scheduled_tasks: bool,
	pub date_format: String,
	done_tag: String,
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			theme: AdhdMateriaTheme::default(),
			default_task: Task::default(),
			repeatable_rewind: RepeatableRewind::default(),
			scheduled_task_tag: Some(String::from(DEFAULT_SCHEDULED_TASK_TAG)),
			delete_used_scheduled_tasks: true,
			date_format: String::from(DEFAULT_DATE_FORMAT),
			done_tag: String::from(DEFAULT_DONE_TAG),
		}
	}
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub enum RepeatableRewind {
	#[default]
	One,
	All,
}

#[derive(
	Debug, PartialEq, Eq, Hash, Clone, Copy, Default, serde::Serialize, serde::Deserialize,
)]
pub enum AdhdMateriaTheme {
	CatppuccinLatte,
	CatppuccinFrappe,
	#[default]
	CatppuccinMacchiato,
	CatppuccinMocha,
}

static THEME_NAMES: LazyLock<std::collections::HashMap<AdhdMateriaTheme, String>> =
	LazyLock::new(|| {
		use AdhdMateriaTheme::*;

		let mut map = std::collections::HashMap::new();

		for theme in [
			CatppuccinLatte,
			CatppuccinFrappe,
			CatppuccinMacchiato,
			CatppuccinMocha,
		] {
			map.insert(
				theme,
				format!("{:?}", theme).to_case(convert_case::Case::Title),
			);
		}

		map
	});

impl std::fmt::Display for AdhdMateriaTheme {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", THEME_NAMES[self])
	}
}

impl AdhdMateriaTheme {
	pub fn apply(&self, ctx: &egui::Context) {
		match self {
			AdhdMateriaTheme::CatppuccinLatte => {
				catppuccin_egui::set_theme(ctx, catppuccin_egui::LATTE)
			}
			AdhdMateriaTheme::CatppuccinFrappe => {
				catppuccin_egui::set_theme(ctx, catppuccin_egui::FRAPPE)
			}
			AdhdMateriaTheme::CatppuccinMacchiato => {
				catppuccin_egui::set_theme(ctx, catppuccin_egui::MACCHIATO)
			}
			AdhdMateriaTheme::CatppuccinMocha => {
				catppuccin_egui::set_theme(ctx, catppuccin_egui::MOCHA)
			}
		}
	}

	pub fn is_dark(&self) -> bool {
		match self {
			AdhdMateriaTheme::CatppuccinLatte => false,
			AdhdMateriaTheme::CatppuccinFrappe => true,
			AdhdMateriaTheme::CatppuccinMacchiato => true,
			AdhdMateriaTheme::CatppuccinMocha => true,
		}
	}
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

	pub fn get_done_tag() -> RwLockReadGuard<'static, Tag> {
		DONE_TAG.read().expect("done tag should be readable")
	}

	pub fn get_done_tag_string_mut(&mut self) -> DoneTagStringGuard {
		DoneTagStringGuard(self)
	}
}

pub struct DoneTagStringGuard<'settings>(&'settings mut Settings);

impl AsMut<String> for DoneTagStringGuard<'_> {
	fn as_mut(&mut self) -> &mut String {
		&mut self.0.done_tag
	}
}

impl Drop for DoneTagStringGuard<'_> {
	fn drop(&mut self) {
		if let Ok(tag) = Tag::from_str(&self.0.done_tag) {
			*DONE_TAG.write().expect("done tag should be writable") = tag;
		}
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

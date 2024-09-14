use std::{
	str::FromStr,
	sync::{LazyLock, Mutex, MutexGuard, RwLock, RwLockReadGuard},
};

use chrono::Datelike;
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
	pub sprint_end_reference: chrono::NaiveDate,
	pub sprint_end: SprintFrequency,
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
			sprint_end_reference: chrono::Local::now().date_naive(),
			sprint_end: SprintFrequency::default(),
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
	Debug, PartialEq, Eq, Clone, Copy, Default, serde::Serialize, serde::Deserialize, kinded::Kinded,
)]
pub enum SprintFrequency {
	#[default]
	Weekly,
	TwoWeekly,
	Monthly,
	Custom {
		days: u32,
	},
}

impl std::fmt::Display for SprintFrequency {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Weekly => write!(f, "Weekly"),
			Self::TwoWeekly => write!(f, "Two Weekly"),
			Self::Monthly => write!(f, "Monthly"),
			Self::Custom { days } => write!(f, "Every {} days", days),
		}
	}
}

impl SprintFrequency {
	pub fn must_reset(&self, reference: chrono::NaiveDate, last_check: chrono::NaiveDate) -> bool {
		match self {
			Self::Weekly => self.must_reset_custom(reference, last_check, 7),
			Self::TwoWeekly => self.must_reset_custom(reference, last_check, 14),
			Self::Monthly => {
				let target_month_day = reference.day0();
				let today = chrono::Local::now().date_naive();

				if today == last_check {
					return false;
				}

				let with_target_month = if today.day0() > target_month_day {
					last_check
						.checked_add_months(chrono::Months::new(1))
						.unwrap()
				} else {
					last_check
				};

				let with_target_day = match with_target_month.with_day0(target_month_day) {
					Some(date) => date,
					None => with_target_month
						.checked_add_months(chrono::Months::new(1))
						.unwrap()
						.with_day0(0)
						.unwrap()
						.pred_opt()
						.unwrap(),
				};

				today >= with_target_day
			}
			Self::Custom { days } => self.must_reset_custom(reference, last_check, *days),
		}
	}

	fn must_reset_custom(
		&self,
		reference: chrono::NaiveDate,
		last_check: chrono::NaiveDate,
		days: u32,
	) -> bool {
		let days = days as i32;

		let reference_dayce = reference.num_days_from_ce();
		let last_check_dayce = last_check.num_days_from_ce();
		let today_dayce = chrono::Local::now().num_days_from_ce();

		if today_dayce == last_check_dayce {
			return false;
		}

		let delta = last_check_dayce - reference_dayce;
		let mut quo = delta / days;
		let rest = delta % days;

		if rest > 0 {
			quo += 1;
		}

		let next_check = quo * days + reference_dayce;

		today_dayce >= next_check
	}
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
		catppuccin_egui::set_theme(ctx, self.get_catppuccin());
	}

	pub fn get_catppuccin(&self) -> catppuccin_egui::Theme {
		match self {
			AdhdMateriaTheme::CatppuccinLatte => catppuccin_egui::LATTE,
			AdhdMateriaTheme::CatppuccinFrappe => catppuccin_egui::FRAPPE,
			AdhdMateriaTheme::CatppuccinMacchiato => catppuccin_egui::MACCHIATO,
			AdhdMateriaTheme::CatppuccinMocha => catppuccin_egui::MOCHA,
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

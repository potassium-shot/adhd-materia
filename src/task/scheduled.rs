use chrono::{Datelike, NaiveDate};

use crate::session::Session;

use super::TaskTypeData;

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ScheduledTask {
	pub active: bool,
	pub date: chrono::NaiveDate,
	pub repeat_mode: RepeatMode,
}

impl Default for ScheduledTask {
	fn default() -> Self {
		Self {
			active: true,
			date: chrono::Local::now().date_naive(),
			repeat_mode: RepeatMode::Never,
		}
	}
}

impl TaskTypeData for ScheduledTask {}

impl ScheduledTask {
	pub fn spawn_count(&self) -> i32 {
		let session = if let Ok(session) = Session::load() {
			session
		} else {
			return 0;
		};

		self.spawn_count_at(chrono::Local::now().date_naive(), session.last_session)
	}

	pub fn spawn_count_at(&self, today: NaiveDate, last_seen: NaiveDate) -> i32 {
		if !self.active || today < self.date {
			return 0;
		}

		match self.repeat_mode {
			RepeatMode::Never => {
				if last_seen < self.date {
					1
				} else {
					0
				}
			}
			RepeatMode::Daily => {
				today
					.signed_duration_since(NaiveDate::max(last_seen.succ_opt().unwrap(), self.date))
					.num_days() as i32 + 1
			}
			RepeatMode::Weekly => {
				let mut count = 0;
				let mut cursor = last_seen.clone();

				if cursor.weekday() == self.date.weekday() {
					count = -1;
				}

				let mut target_day = today
					.week(chrono::Weekday::Mon)
					.first_day()
					.checked_add_days(chrono::Days::new(
						self.date.weekday().num_days_from_monday().into(),
					))
					.unwrap();

				if today.weekday().num_days_from_monday()
					< self.date.weekday().num_days_from_monday()
				{
					target_day = target_day.checked_sub_days(chrono::Days::new(7)).unwrap();
				}

				while cursor <= target_day {
					count += 1;
					cursor = cursor.checked_add_days(chrono::Days::new(7)).unwrap();
				}

				count
			}
			RepeatMode::Monthly => {
				let mut count = 0;
				let mut cursor = last_seen.clone();

				if cursor.day0() == self.date.day0() {
					count = -1;
				}

				let mut target_day = today.with_day0(self.date.day0()).unwrap_or_else(|| {
					today
						.with_day0(0)
						.unwrap()
						.checked_add_months(chrono::Months::new(1))
						.unwrap()
						.checked_sub_days(chrono::Days::new(1))
						.unwrap()
				});

				if today.day0() < self.date.day0() {
					target_day = target_day
						.checked_sub_months(chrono::Months::new(1))
						.unwrap();
				}

				while cursor <= target_day {
					count += 1;
					cursor = cursor.checked_add_months(chrono::Months::new(1)).unwrap();
				}

				count
			}
			RepeatMode::Yearly => {
				let mut count = 0;
				let mut cursor = last_seen.clone();

				if cursor.ordinal0() == self.date.ordinal0() {
					count = -1;
				}

				let mut target_day =
					today
						.with_ordinal0(self.date.ordinal0())
						.unwrap_or_else(|| {
							today
								.with_ordinal0(0)
								.unwrap()
								.checked_add_months(chrono::Months::new(12))
								.unwrap()
								.checked_sub_days(chrono::Days::new(1))
								.unwrap()
						});

				if today.ordinal0() < self.date.ordinal0() {
					target_day = target_day
						.checked_sub_months(chrono::Months::new(12))
						.unwrap();
				}

				while cursor <= target_day {
					count += 1;
					cursor = cursor.checked_add_months(chrono::Months::new(12)).unwrap();
				}

				count
			}
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum RepeatMode {
	#[default]
	Never,
	Daily,
	Weekly,
	Monthly,
	Yearly,
}

impl std::fmt::Display for RepeatMode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				RepeatMode::Never => "not repeated",
				RepeatMode::Daily => "repeated daily",
				RepeatMode::Weekly => "repeated weekly",
				RepeatMode::Monthly => "repeated monthly",
				RepeatMode::Yearly => "repeated yearly",
			}
		)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	macro_rules! date {
		($y: expr, $m: expr, $d: expr) => {
			NaiveDate::from_ymd_opt($y, $m, $d).unwrap()
		};
	}

	#[test]
	fn test_a() {
		let d = ScheduledTask {
			date: date!(2024, 1, 5),
			repeat_mode: RepeatMode::Never,
			active: true,
		};

		assert_eq!(d.spawn_count_at(date!(2024, 1, 8), date!(2024, 1, 1)), 1);
	}

	#[test]
	fn test_b() {
		let d = ScheduledTask {
			date: date!(2024, 1, 5),
			repeat_mode: RepeatMode::Daily,
			active: true,
		};

		assert_eq!(d.spawn_count_at(date!(2024, 1, 8), date!(2024, 1, 1)), 4);
	}

	#[test]
	fn test_c() {
		let d = ScheduledTask {
			date: date!(2024, 1, 5),
			repeat_mode: RepeatMode::Weekly,
			active: true,
		};

		assert_eq!(d.spawn_count_at(date!(2024, 1, 23), date!(2024, 1, 1)), 3);
	}

	#[test]
	fn test_d() {
		let d = ScheduledTask {
			date: date!(2024, 1, 5),
			repeat_mode: RepeatMode::Monthly,
			active: true,
		};

		assert_eq!(d.spawn_count_at(date!(2024, 4, 23), date!(2024, 1, 1)), 4);
	}

	#[test]
	fn test_e() {
		let d = ScheduledTask {
			date: date!(2024, 1, 16),
			repeat_mode: RepeatMode::Monthly,
			active: true,
		};

		assert_eq!(d.spawn_count_at(date!(2024, 7, 4), date!(2024, 1, 16)), 5);
	}

	#[test]
	fn test_f() {
		let d = ScheduledTask {
			date: date!(2024, 4, 16),
			repeat_mode: RepeatMode::Yearly,
			active: true,
		};

		assert_eq!(d.spawn_count_at(date!(2027, 10, 6), date!(2024, 1, 12)), 4);
	}
}

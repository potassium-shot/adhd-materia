use super::TaskTypeData;

#[derive(Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct ScheduledTask {
	pub date: chrono::NaiveDate,
}

impl TaskTypeData for ScheduledTask { }

impl ScheduledTask {
	pub fn new(date: chrono::NaiveDate) -> Self {
		Self { date }
	}
}

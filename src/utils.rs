use std::fmt::Write;

use chrono::NaiveDate;

pub trait ChronoDelayFormatExt {
	fn write_or_err(&self, fmt: &str, buf: impl Write) -> Result<(), std::fmt::Error>;

	fn format_or_err(&self, fmt: &str) -> Result<String, std::fmt::Error> {
		let mut buf = String::new();
		self.write_or_err(fmt, &mut buf)?;
		Ok(buf)
	}
}

impl ChronoDelayFormatExt for NaiveDate {
	fn write_or_err(&self, fmt: &str, mut buf: impl Write) -> Result<(), std::fmt::Error> {
		write!(buf, "{}", self.format(fmt))?;
		Ok(())
	}
}

#[macro_export]
macro_rules! toast_error {
	($fmt: expr, $e: expr) => {
		crate::toasts()
			.error(format!($fmt, $e))
			.set_closable(true)
			.set_duration(Some(std::time::Duration::from_millis(10_000)))
	};
}

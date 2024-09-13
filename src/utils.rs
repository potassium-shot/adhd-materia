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
macro_rules! toast {
	($type: ident, $time: expr, $fmt: expr, $($e: expr),*) => {
		crate::toasts()
			.$type(format!($fmt, $($e),*))
			.set_closable(true)
			.set_duration(Some(std::time::Duration::from_millis($time)))
	};
	($type: ident, $fmt: expr) => {
		crate::toast!($type, $fmt,)
	}
}

#[macro_export]
macro_rules! toast_error {
	($fmt: expr, $($e: expr),*) => {
		{
			log::error!($fmt, $($e),*);
			crate::toast!(error, 10_000, $fmt, $($e),*)
		}
	};
	($fmt: expr) => {
		crate::toast_error!($fmt,)
	}
}

#[macro_export]
macro_rules! toast_info {
	($fmt: expr, $($e: expr),*) => {
		{
			log::info!($fmt, $($e),*);
			crate::toast!(info, 3000, $fmt, $($e),*)
		}
	};
	($fmt: expr) => {
		crate::toast_info!($fmt,)
	}
}

#[macro_export]
macro_rules! toast_success {
	($fmt: expr, $($e: expr),*) => {
		{
			log::info!($fmt, $($e),*);
			crate::toast!(success, 3000, $fmt, $($e),*)
		}
	};
	($fmt: expr) => {
		crate::toast_success!($fmt,)
	}
}

#[macro_export]
macro_rules! handle_toast_error {
	($fmt: expr, $e: expr) => {{
		if let Err(e) = $e {
			crate::toast_error!($fmt, e);
		}
	}};
}

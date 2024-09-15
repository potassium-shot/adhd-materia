use std::{collections::HashMap, fmt::Write, iter::Peekable};

use chrono::NaiveDate;

use crate::settings::Settings;

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

static HELP_STRING_CACHE: std::sync::LazyLock<
	std::sync::RwLock<HashMap<&'static str, Vec<HelpStringPart>>>,
> = std::sync::LazyLock::new(|| std::sync::RwLock::new(HashMap::new()));

struct HelpStringPart {
	text: String,
	ty: HelpStringPartType,
}

enum HelpStringPartType {
	Text,
	Code,
	Link(i32),
}

#[macro_export]
macro_rules! help_string {
	($ui: expr, $fp: expr) => {
		crate::utils::format_help_string(
			$ui,
			include_str!(concat!("../assets/help_strings/", $fp, ".txt")),
		)
	};
}

pub fn format_help_string(ui: &mut egui::Ui, help_string: &'static str) -> Option<i32> {
	if !Settings::get().help_messages {
		return None;
	}

	if !HELP_STRING_CACHE
		.try_read()
		.unwrap()
		.contains_key(help_string)
	{
		let mut parts = Vec::new();
		let mut current_part = HelpStringPart {
			text: String::new(),
			ty: HelpStringPartType::Text,
		};
		let mut chars = help_string.chars().peekable();

		while let Some(c) = chars.peek() {
			match c {
				'[' => {
					parts.push(std::mem::replace(
						&mut current_part,
						HelpStringPart {
							text: String::new(),
							ty: HelpStringPartType::Text,
						},
					));
					parts.push(parse_link_part(&mut chars, help_string));
				}
				']' => panic!("Found unmatched `]` in help string `{help_string}`"),
				'`' => {
					parts.push(std::mem::replace(
						&mut current_part,
						HelpStringPart {
							text: String::new(),
							ty: HelpStringPartType::Text,
						},
					));
					parts.push(parse_code_part(&mut chars, help_string));
				}
				_ => current_part
					.text
					.push(chars.next().expect("peek returned some")),
			}
		}

		if !current_part.text.is_empty() {
			parts.push(current_part);
		}

		HELP_STRING_CACHE
			.try_write()
			.unwrap()
			.insert(help_string, parts);
	}

	let help_string_cache_borrow = HELP_STRING_CACHE.try_read().unwrap();
	let parts = help_string_cache_borrow
		.get(help_string)
		.expect("just added it");
	let mut pressed_code = None;

	let rect = ui
		.horizontal_wrapped(|ui| {
			ui.visuals_mut().override_text_color =
				Some(crate::settings::Settings::get().theme.get_catppuccin().blue);
			ui.add(egui::Label::new("ℹ").selectable(false));
			ui.spacing_mut().item_spacing.x = 0.0;

			for part in parts {
				match part.ty {
					HelpStringPartType::Text => {
						ui.add(egui::Label::new(part.text.as_str()).selectable(false));
					}
					HelpStringPartType::Code => {
						ui.add(
							egui::Label::new(
								egui::RichText::new(part.text.as_str())
									.code()
									.background_color(ui.visuals().code_bg_color)
									.color(ui.visuals().strong_text_color()),
							)
							.selectable(false),
						);
					}
					HelpStringPartType::Link(link_code) => {
						if ui
							.link(
								egui::RichText::new(part.text.as_str())
									.color(ui.visuals().hyperlink_color),
							)
							.clicked()
						{
							pressed_code = Some(link_code);
						}
					}
				}
			}
		})
		.response
		.rect;

	ui.interact(rect, egui::Id::new(help_string), egui::Sense::hover())
		.on_hover_text_at_pointer("Help messages can be disabled in the settings.");

	pressed_code
}

fn parse_link_part(
	chars: &mut Peekable<impl Iterator<Item = char>>,
	help_string: &'static str,
) -> HelpStringPart {
	assert!(chars.next() == Some('['));

	let mut part = HelpStringPart {
		text: String::new(),
		ty: HelpStringPartType::Text,
	};

	while let Some(c) = chars.peek() {
		match c {
			']' => {
				chars.next().expect("peek returned some");

				let mut number_str = String::new();

				while let Some(c_number) = chars.peek() {
					match c_number {
						'0'..='9' => {
							number_str.push(chars.next().expect("peek returned some"));
						}
						_ => break,
					}
				}

				part.ty = HelpStringPartType::Link(number_str.parse().unwrap());
				return part;
			}
			_ => part.text.push(chars.next().expect("peek returned some")),
		}
	}

	panic!("Found unmatched `[` in help string `{help_string}`");
}

fn parse_code_part(
	chars: &mut Peekable<impl Iterator<Item = char>>,
	help_string: &'static str,
) -> HelpStringPart {
	assert!(chars.next() == Some('`'));

	let mut part = HelpStringPart {
		text: String::new(),
		ty: HelpStringPartType::Code,
	};

	while let Some(c) = chars.peek() {
		match c {
			'`' => {
				chars.next().expect("peek returned some");
				return part;
			}
			_ => part.text.push(chars.next().expect("peek returned some")),
		}
	}

	panic!("Found unmatched ``` in help string `{help_string}`");
}

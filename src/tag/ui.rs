use std::sync::LazyLock;

use convert_case::Casing;

use crate::{
	settings::{Settings, DEFAULT_DATE_FORMAT},
	utils::ChronoDelayFormatExt,
};

use super::{Tag, TagValue};

static COLORHASH: LazyLock<colorhash::ColorHash> = LazyLock::new(|| colorhash::ColorHash::new());

pub struct TagWidget<'tag> {
	tag: &'tag mut Tag,
	edit_mode: bool,
}

impl<'tag> TagWidget<'tag> {
	pub fn new(tag: &'tag mut Tag, edit_mode: bool) -> Self {
		Self { tag, edit_mode }
	}
}

impl egui::Widget for TagWidget<'_> {
	fn ui(self, ui: &mut egui::Ui) -> egui::Response {
		if self.edit_mode {
			ui.add(egui::TextEdit::singleline(self.tag.get_editing_text()).code_editor())
		} else {
			Self::draw_tag(ui, &self.tag)
		}
	}
}

impl TagWidget<'_> {
	fn draw_tag(ui: &mut egui::Ui, tag: &Tag) -> egui::Response {
		let col_hash = COLORHASH.rgb(&tag.name);
		let col = egui::Color32::from_rgb(
			col_hash.red() as u8,
			col_hash.green() as u8,
			col_hash.blue() as u8,
		);

		egui::Frame::group(ui.style())
			.stroke(egui::Stroke::new(
				ui.style().visuals.window_stroke().width,
				col,
			))
			.fill(col.lerp_to_gamma(ui.style().visuals.window_fill(), 0.8))
			.show(ui, |ui| {
				ui.label(tag.name.to_case(convert_case::Case::Title));

				if let Some(value) = &tag.value {
					Self::draw_tag_value(ui, value, col);
				}
			})
			.response
	}

	fn draw_tag_value(ui: &mut egui::Ui, tag_value: &TagValue, color: egui::Color32) {
		let color = color.lerp_to_gamma(egui::Color32::WHITE, 0.3);

		match tag_value {
			TagValue::Tag(tag) => {
				Self::draw_tag(ui, tag);
			}
			TagValue::List(list) => {
				for value in list {
					Self::draw_tag_value(ui, value, color);
				}
			}
			TagValue::Dictionary(dict) => {
				for (key, value) in dict {
					ui.weak(key);
					Self::draw_tag_value(ui, value, color);
				}
			}
			other => {
				egui::Frame::group(ui.style())
					.stroke(egui::Stroke::new(
						ui.style().visuals.window_stroke().width,
						color,
					))
					.fill(color.lerp_to_gamma(ui.style().visuals.window_fill(), 0.8))
					.show(ui, |ui| {
						ui.label(match other {
							TagValue::Int(i) => i.to_string(),
							TagValue::Float(f) => f.to_string(),
							TagValue::Date(d) => d
								.format_or_err(Settings::get().date_format.as_str())
								.unwrap_or(d.format(DEFAULT_DATE_FORMAT).to_string()),
							TagValue::Text(t) => t.clone(),
							TagValue::List(_) | TagValue::Dictionary(_) | TagValue::Tag(_) => {
								unreachable!("already handled")
							}
						});
					});
			}
		}
	}
}

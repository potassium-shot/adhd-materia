use std::{collections::HashMap, sync::LazyLock};

use convert_case::Casing;
use uuid::Uuid;

use crate::{
	settings::{Settings, DEFAULT_DATE_FORMAT},
	utils::ChronoDelayFormatExt,
};

use super::{Tag, TagValue};

static COLORHASH: LazyLock<colorhash::ColorHash> = LazyLock::new(|| colorhash::ColorHash::new());

fn get_tag_color(tag: &Tag) -> egui::Color32 {
	if tag.name.as_str() == "done" {
		egui::Color32::from_rgb(0x20, 0xF0, 0x20)
	} else {
		let col_hash = COLORHASH.rgb(&tag.name);
		egui::Color32::from_rgb(
			col_hash.red() as u8,
			col_hash.green() as u8,
			col_hash.blue() as u8,
		)
	}
}

pub struct TagWidget<'tag> {
	tag: &'tag mut Tag,
	edit_mode: bool,
}

impl<'tag> TagWidget<'tag> {
	pub fn new(tag: &'tag mut Tag, edit_mode: bool) -> Self {
		Self { tag, edit_mode }
	}
}

pub enum TagSwapRequest {
	Forward,
	Backward,
}

impl TagWidget<'_> {
	pub fn show(
		self,
		ui: &mut egui::Ui,
		task_names: &HashMap<Uuid, String>,
		scroll_to: &mut Option<Uuid>,
		selected_task: &mut Option<Uuid>,
	) -> Option<TagSwapRequest> {
		if self.edit_mode {
			let mut swap_req = None;

			if ui.add(egui::Button::new("◀").frame(false)).clicked() {
				swap_req = Some(TagSwapRequest::Backward);
			}

			ui.add(egui::TextEdit::singleline(self.tag.get_editing_text()).code_editor());

			if ui.add(egui::Button::new("▶").frame(false)).clicked() {
				swap_req = Some(TagSwapRequest::Forward);
			}

			swap_req
		} else {
			let (rect, _) = ui.allocate_at_least(
				egui::Vec2::new(1.0, (19 + 12 * self.tag.nested_block_count()) as f32),
				egui::Sense::hover(),
			);

			ui.allocate_ui_at_rect(rect.with_max_x(ui.available_width()), |ui| {
				ui.horizontal_centered(|ui| {
					Self::draw_tag(ui, &self.tag, task_names, scroll_to, selected_task);
				});
			});

			None
		}
	}
}

impl TagWidget<'_> {
	fn draw_tag(
		ui: &mut egui::Ui,
		tag: &Tag,
		task_names: &HashMap<Uuid, String>,
		scroll_to: &mut Option<Uuid>,
		selected_task: &mut Option<Uuid>,
	) -> egui::Response {
		let col = get_tag_color(tag);

		egui::Frame::group(ui.style())
			.stroke(egui::Stroke::new(
				ui.style().visuals.window_stroke().width,
				col,
			))
			.fill(col.lerp_to_gamma(ui.style().visuals.window_fill(), 0.8))
			.show(ui, |ui| {
				ui.label(tag.name.to_case(convert_case::Case::Title));

				if let Some(value) = &tag.value {
					Self::draw_tag_value(ui, value, col, task_names, scroll_to, selected_task);
				}
			})
			.response
	}

	fn draw_tag_value(
		ui: &mut egui::Ui,
		tag_value: &TagValue,
		color: egui::Color32,
		task_names: &HashMap<Uuid, String>,
		scroll_to: &mut Option<Uuid>,
		selected_task: &mut Option<Uuid>,
	) {
		let color = color.lerp_to_gamma(egui::Color32::WHITE, 0.3);

		match tag_value {
			TagValue::Tag(tag) => {
				Self::draw_tag(ui, tag, task_names, scroll_to, selected_task);
			}
			TagValue::List(list) => {
				for value in list {
					Self::draw_tag_value(ui, value, color, task_names, scroll_to, selected_task);
				}
			}
			TagValue::Dictionary(dict) => {
				for (key, value) in dict {
					ui.weak(key);
					Self::draw_tag_value(ui, value, color, task_names, scroll_to, selected_task);
				}
			}
			TagValue::TaskReference(uuid) => {
				if ui
					.button(
						egui::RichText::new(
							task_names.get(uuid).map(|s| s.as_str()).unwrap_or("???"),
						)
						.color(Settings::get().theme.get_catppuccin().yellow),
					)
					.clicked()
				{
					*scroll_to = Some(*uuid);
					*selected_task = Some(*uuid);
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
							TagValue::Bool(b) => egui::RichText::new(b.to_string()).color(if *b {
								Settings::get().theme.get_catppuccin().teal
							} else {
								Settings::get().theme.get_catppuccin().flamingo
							}),
							TagValue::Int(i) => egui::RichText::new(i.to_string()),
							TagValue::Float(f) => egui::RichText::new(f.to_string()),
							TagValue::Date(d) => egui::RichText::new(
								d.format_or_err(Settings::get().date_format.as_str())
									.unwrap_or(d.format(DEFAULT_DATE_FORMAT).to_string()),
							),
							TagValue::Text(t) => egui::RichText::new(t.clone()),
							TagValue::List(_)
							| TagValue::Dictionary(_)
							| TagValue::Tag(_)
							| TagValue::TaskReference(_) => {
								unreachable!("already handled")
							}
						});
					});
			}
		}
	}
}

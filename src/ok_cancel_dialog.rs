pub struct OkCancelDialog {
	title: String,
	subtext: egui::WidgetText,
	ok_text: &'static str,
	ok_color: egui::Color32,
	cancel_text: &'static str,
	cancel_color: egui::Color32,
}

impl Default for OkCancelDialog {
	fn default() -> Self {
		Self {
			title: String::new(),
			subtext: egui::WidgetText::default(),
			ok_text: "Ok",
			ok_color: egui::Color32::PLACEHOLDER,
			cancel_text: "Cancel",
			cancel_color: egui::Color32::PLACEHOLDER,
		}
	}
}

impl OkCancelDialog {
	pub fn with_title(mut self, title: impl ToString) -> Self {
		self.title = title.to_string();
		self
	}

	pub fn with_subtext(mut self, subtext: impl Into<egui::WidgetText>) -> Self {
		self.subtext = subtext.into();
		self
	}

	pub fn with_ok_text(mut self, ok_text: &'static str) -> Self {
		self.ok_text = ok_text;
		self
	}

	pub fn with_ok_color(mut self, ok_color: egui::Color32) -> Self {
		self.ok_color = ok_color;
		self
	}

	pub fn show(self, ctx: &egui::Context) -> Option<OkCancelResult> {
		let mut result = None;

		egui::Window::new(self.title.as_str())
			.anchor(egui::Align2::CENTER_CENTER, (0.0, 0.0))
			.collapsible(false)
			.min_size((1.0, 1.0))
			.auto_sized()
			.show(ctx, |ui| {
				ui.label(self.subtext.clone());

				ui.horizontal(|ui| {
					if ui
						.button(egui::RichText::from(self.ok_text).color(self.ok_color.clone()))
						.clicked()
					{
						result = Some(OkCancelResult::Ok);
					}

					if ui
						.button(
							egui::RichText::from(self.cancel_text).color(self.cancel_color.clone()),
						)
						.clicked()
					{
						result = Some(OkCancelResult::Cancel);
					}
				});
			});

		result
	}
}

pub enum OkCancelResult {
	Ok,
	Cancel,
}

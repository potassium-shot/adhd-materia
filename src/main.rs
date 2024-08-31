fn main() -> eframe::Result {
	env_logger::init();

	let native_options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default()
			.with_inner_size((640.0, 480.0))
			.with_icon(
				eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon256.png")[..])
					.expect("icon256.png image data should be valid"),
			),
		..Default::default()
	};
	eframe::run_native(
		"ADHD Materia",
		native_options,
		Box::new(|cc| Ok(Box::new(adhd_materia::AdhdMateriaApp::new(cc)))),
	)
}

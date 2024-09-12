use std::sync::LazyLock;

use crate::toast_error;

use super::{badge::BadgeType, list::ScriptEditorStateKind, PocketPyScript};

pub static DEFAULT_STANDALONE_SCRIPT: LazyLock<PocketPyScript> = LazyLock::new(|| PocketPyScript {
	name: "new_script".to_string(),
	code: String::new(),
});

pub struct StandaloneScriptBadgeType;

impl BadgeType for StandaloneScriptBadgeType {
	fn get_path_from_data_dir(
		data_dir: &'static crate::data_dir::DataDir,
	) -> &'static std::path::Path {
		data_dir.standalone_scripts()
	}

	fn display_order() -> bool {
		unimplemented!("Standalone scripts don't have badges")
	}

	fn get_session_badge_list(_session: &crate::session::Session) -> &Vec<String> {
		unimplemented!("Standalone scripts don't have badges")
	}

	fn get_session_badge_list_mut(_session: &mut crate::session::Session) -> &mut Vec<String> {
		unimplemented!("Standalone scripts don't have badges")
	}

	fn draw_ui_titlebar(
		ui: &mut egui::Ui,
		state: ScriptEditorStateKind,
		script: &mut PocketPyScript,
	) {
		if let ScriptEditorStateKind::DisplayMode = state {
			if ui.button("Run").clicked() {
				if let Err(e) = script.execute_function::<()>(crate::app::script_lock(), "run", [])
				{
					toast_error!("Error running script: {}", e);
				}
			}
		}
	}
}

use std::sync::LazyLock;

use super::{
	badge::{BadgeList, BadgeType},
	PocketPyScript,
};

pub static DEFAULT_SORTING_SCRIPT: LazyLock<PocketPyScript> = LazyLock::new(|| PocketPyScript {
	name: "new_sorting".to_string(),
	code: "return 0".to_string(),
});

pub struct SortingBadgeType;

impl BadgeType for SortingBadgeType {
	fn get_path_from_data_dir(
		data_dir: &'static crate::data_dir::DataDir,
	) -> &'static std::path::Path {
		data_dir.sorting_scripts()
	}

	fn display_order() -> bool {
		true
	}

	fn get_session_badge_list(session: &crate::session::Session) -> &Vec<String> {
		&session.set_sortings
	}

	fn get_session_badge_list_mut(session: &mut crate::session::Session) -> &mut Vec<String> {
		&mut session.set_sortings
	}
}

pub type SortingList = BadgeList<SortingBadgeType>;

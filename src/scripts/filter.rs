use std::sync::LazyLock;

use super::{
	badge::{BadgeList, BadgeType},
	PocketPyScript,
};

pub static DEFAULT_FILTER_SCRIPT: LazyLock<PocketPyScript> = LazyLock::new(|| PocketPyScript {
	name: "new_filter".to_string(),
	code: "return True".to_string(),
});

pub struct FilterBadgeType;

impl BadgeType for FilterBadgeType {
	fn get_path_from_data_dir(
		data_dir: &'static crate::data_dir::DataDir,
	) -> &'static std::path::Path {
		data_dir.filter_scripts()
	}

	fn display_order() -> bool {
		false
	}
	
	fn get_session_badge_list(session: &crate::session::Session) -> &Vec<String> {
		&session.set_filters
	}

	fn get_session_badge_list_mut(session: &mut crate::session::Session) -> &mut Vec<String> {
		&mut session.set_filters
	}
}

pub type FilterList = BadgeList<FilterBadgeType>;

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
}

pub type FilterList = BadgeList<FilterBadgeType>;

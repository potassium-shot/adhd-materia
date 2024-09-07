use std::sync::LazyLock;

use super::PocketPyScript;

pub static DEFAULT_FILTER_SCRIPT: LazyLock<PocketPyScript> = LazyLock::new(|| PocketPyScript {
	name: "new_filter".to_string(),
	code: "return True".to_string(),
});

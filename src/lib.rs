mod app;
mod data_dir;
mod ok_cancel_dialog;
mod side_panel;
mod tag;
mod task;

use std::sync::{LazyLock, Mutex, MutexGuard};

pub use app::AdhdMateriaApp;
use data_dir::{DataDir, DataDirError};

static TOASTS: LazyLock<Mutex<egui_notify::Toasts>> =
	LazyLock::new(|| Mutex::new(egui_notify::Toasts::default()));

pub fn toasts() -> MutexGuard<'static, egui_notify::Toasts> {
	TOASTS.lock().expect("TOASTS should be lockable")
}

static DATA_DIR: LazyLock<Result<DataDir, DataDirError>> = LazyLock::new(DataDir::new);

pub fn data_dir() -> Result<&'static DataDir, &'static DataDirError> {
	DATA_DIR.as_ref()
}

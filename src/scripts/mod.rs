use std::{
	ffi::{CStr, CString},
	ptr::null_mut,
	sync::{Mutex, MutexGuard},
};

use badge::BadgeType;
use pocketpy_sys::*;

use standalone_script::StandaloneScriptBadgeType;
use value::{AnyIntoPocketPyValue, IntoPocketPyValue};

use crate::data_dir::DataDirError;

pub mod badge;
pub mod filter;
pub mod list;
mod py_bindings;
pub mod sorting;
pub mod standalone_script;
pub mod ui;
pub mod value;

static POCKETPY_LOCK: Mutex<Mutex<()>> = Mutex::new(Mutex::new(()));

#[allow(unused_macros)]
macro_rules! pyprintln {
	($fmt: expr, $($arg: expr),*) => {
		unsafe {
			println!($fmt, $(
				{
					py_repr($arg);
					std::ffi::CStr::from_ptr(py_tostr(py_retval())).to_string_lossy().to_string()
				}
			),*);
		}
	};
}

pub(crate) struct PocketPyLock<'lock>(MutexGuard<'lock, Mutex<()>>);

impl PocketPyLock<'_> {
	pub fn new() -> Self {
		let lock = POCKETPY_LOCK.lock().unwrap();

		unsafe {
			py_initialize();
			py_bindings::initialize_bindings();
		}

		println!("PocketPy initialized");

		Self(lock)
	}
}

pub(crate) struct PocketPyLockGuard<'lock>(#[allow(dead_code)] MutexGuard<'lock, ()>);

impl<'lock> PocketPyLock<'lock> {
	pub fn lock(&'lock self) -> PocketPyLockGuard<'lock> {
		PocketPyLockGuard(self.0.lock().unwrap())
	}
}

impl Drop for PocketPyLock<'_> {
	fn drop(&mut self) {
		unsafe {
			py_finalize();
			println!("PocketPy finalized");
		}
	}
}

#[derive(Debug, Clone)]
pub struct PocketPyScript {
	pub name: String,
	pub code: String,
}

impl PocketPyScript {
	pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self, PocketPyScriptError> {
		let path = path.as_ref();
		let code = std::fs::read_to_string(path)?;

		Ok(Self {
			name: path
				.file_stem()
				.expect("if no filename, should have failed earlier")
				.to_string_lossy()
				.to_string(),
			code,
		})
	}

	pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<(), PocketPyScriptError> {
		let path = path.as_ref();
		std::fs::write(
			path.join(self.name.as_str()).with_extension("py"),
			&self.code,
		)?;
		Ok(())
	}

	pub fn delete(&self, path: impl AsRef<std::path::Path>) -> Result<(), PocketPyScriptError> {
		let path = path.as_ref();
		std::fs::remove_file(path.join(self.name.as_str()).with_extension("py"))?;
		Ok(())
	}

	pub fn execute_function_for<ReturnType: IntoPocketPyValue + 'static>(
		&self,
		_lock: PocketPyLockGuard<'_>,
		name: &str,
		args: impl IntoIterator<Item = (&'static str, Vec<AnyIntoPocketPyValue>)>,
	) -> Result<Vec<ReturnType>, PocketPyScriptError> {
		let name_c = CString::new(self.name.as_str())
			.expect("Name from filename should not contain 0 bytes");
		let func_name_c = CString::new(name).expect("Name should not contain 0 bytes");

		let (arg_names, args): (Vec<&'static str>, Vec<_>) =
			args.into_iter().map(|a| (a.0, a.1)).unzip();

		let arg_names = arg_names.join(",");

		let code = CString::new(format!(
			"def {}({}):\n\t{}",
			name,
			arg_names,
			self.code.replace('\n', "\n\t").as_str()
		))
		.expect("Code should not contain 0 bytes");

		let call_count = args.get(0).map(|el| el.len()).unwrap_or(1);
		let mut ret_vals = Vec::with_capacity(call_count);

		unsafe {
			if !py_exec(
				code.as_ptr(),
				name_c.as_ptr(),
				py_CompileMode_EXEC_MODE,
				null_mut(),
			) {
				return Err(PocketPyScriptError::PocketPyError(
					CStr::from_ptr(py_formatexc()).to_string_lossy().to_string(),
				));
			}

			let func_ref = py_getglobal(py_name(func_name_c.as_ptr()));
			let argc = args.len();

			for call_i in 0..call_count {
				py_push(func_ref);
				py_pushnil();

				for arg_i in 0..argc {
					args[arg_i][call_i].push_pocketpy_value();
				}

				if !py_vectorcall(argc as u16, 0) {
					return Err(PocketPyScriptError::PocketPyError(
						CStr::from_ptr(py_formatexc()).to_string_lossy().to_string(),
					));
				}

				ret_vals.push(ReturnType::from_pocketpy_value_ptr(py_retval())?);
			}

			Ok(ret_vals)
		}
	}

	pub fn execute_function<ReturnType: IntoPocketPyValue + 'static>(
		&self,
		lock: PocketPyLockGuard<'_>,
		name: &str,
		args: impl IntoIterator<Item = (&'static str, AnyIntoPocketPyValue)>,
	) -> Result<ReturnType, PocketPyScriptError> {
		Ok(self
			.execute_function_for::<ReturnType>(
				lock,
				name,
				args.into_iter()
					.map(|(arg_name, arg_val)| (arg_name, vec![arg_val])),
			)?
			.pop()
			.expect("Function should return at least one value"))
	}
}

pub fn run_standalone_script(name: &str) {
	if let Ok(path) = StandaloneScriptBadgeType::get_path() {
		if let Ok(script) = PocketPyScript::load(path.join(name).with_extension("py")) {
			if let Err(e) = script.execute_function::<()>(crate::app::script_lock(), "run", []) {
				eprintln!("Error running script: {}", e);
			}
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum PocketPyScriptError {
	#[error("PocketPy compilation error: {0}")]
	PocketPyError(String),

	#[error("Script returned value of the wrong type")]
	WrongType,

	#[error("IO Error: {0}")]
	IOError(
		#[from]
		#[source]
		std::io::Error,
	),

	#[error("Could not access data directory: {0}")]
	DataDirError(
		#[from]
		#[source]
		&'static DataDirError,
	),
}

#[cfg(test)]
mod tests {
	use super::*;

	macro_rules! py_arg {
		($name: expr, $value: expr) => {
			($name, Box::new($value) as Box<dyn IntoPocketPyValue>)
		};
	}

	macro_rules! py_args {
		($name: expr, $($value: expr),*) => {
			($name, vec![$(Box::new($value) as Box<dyn IntoPocketPyValue>),*])
		};
	}

	#[test]
	fn test_a() {
		let lock = PocketPyLock::new();

		assert_eq!(
			PocketPyScript {
				name: "test_a".to_string(),
				code: "new_x = x + 4\nreturn new_x + y * 2".to_string(),
			}
			.execute_function::<i64>(
				lock.lock(),
				"one_plus_one",
				[py_arg!("x", 5i64), py_arg!("y", 2i64)]
			)
			.unwrap(),
			13,
		);

		drop(lock);
	}

	#[test]
	fn test_b() {
		let lock = PocketPyLock::new();

		assert_eq!(
			PocketPyScript {
				name: "test_b".to_string(),
				code: "new_x = x + 4\nreturn new_x + y * 2".to_string(),
			}
			.execute_function_for::<i64>(
				lock.lock(),
				"one_plus_one",
				[py_args!("x", 5i64, 10i64), py_args!("y", 2i64, 3i64)]
			)
			.unwrap(),
			vec![13i64, 20i64],
		);

		drop(lock);
	}
}

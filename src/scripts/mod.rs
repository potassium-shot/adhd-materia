use std::{
	ffi::{CStr, CString},
	ptr::null_mut,
	sync::{Mutex, MutexGuard},
};

use chrono::Datelike;
use pocketpy_sys::*;

use value::{AnyIntoPocketPyValue, IntoPocketPyValue};

use crate::data_dir::DataDirError;

pub mod badge;
pub mod filter;
pub mod list;
pub mod sorting;
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

pub fn new_py_date_py_values(
	out: *mut py_TValue,
	year: *mut py_TValue,
	month: *mut py_TValue,
	day: *mut py_TValue,
) -> bool {
	unsafe {
		py_newobject(
			out,
			py_totype(py_getglobal(py_name(c"Date".as_ptr()))),
			-1,
			0,
		);

		py_setdict(out, py_name(c"year".as_ptr()), year);
		py_setdict(out, py_name(c"month".as_ptr()), month);
		py_setdict(out, py_name(c"day".as_ptr()), day);
		true
	}
}

pub fn new_py_date(out: *mut py_TValue, date: &chrono::NaiveDate) -> bool {
	unsafe {
		for (i, element) in [
			date.year() as i64,
			date.month0() as i64 + 1,
			date.day0() as i64 + 1,
		]
		.into_iter()
		.enumerate()
		{
			py_newint(py_getreg(i as i32 + 5), element);
		}

		new_py_date_py_values(out, py_getreg(5), py_getreg(6), py_getreg(7))
	}
}

pub(crate) struct PocketPyLock<'lock>(MutexGuard<'lock, Mutex<()>>);

impl PocketPyLock<'_> {
	pub fn new() -> Self {
		let lock = POCKETPY_LOCK.lock().unwrap();

		unsafe {
			py_initialize();

			// Create the Task python type
			let task_type = py_newtype(
				c"Task".as_ptr(),
				py_totype(py_getbuiltin(py_name(c"object".as_ptr()))),
				null_mut(),
				None,
			);

			py_setglobal(py_name(c"Task".as_ptr()), py_tpobject(task_type));

			#[allow(non_snake_case)]
			unsafe extern "C" fn task____new__(
				_argc: std::os::raw::c_int,
				_argv: *mut py_TValue,
			) -> bool {
				py_newobject(
					py_retval(),
					py_totype(py_getglobal(py_name(c"Task".as_ptr()))),
					-1,
					0,
				);
				true
			}

			py_bindmethod(task_type, c"__new__".as_ptr(), Some(task____new__));

			#[allow(non_snake_case)]
			unsafe extern "C" fn task__get_tag_with_name(
				argc: std::os::raw::c_int,
				argv: *mut py_TValue,
			) -> bool {
				if argc != 2 {
					py_newnone(py_retval());
					return py_exception(
						py_totype(py_getbuiltin(py_name(c"Exception".as_ptr()))),
						c"Expected 2 arguments".as_ptr(),
					);
				}

				let self_ = argv;
				let tag_name = ((argv as usize) + size_of::<usize>() * 2) as *mut py_TValue;
				let tag_list = py_getdict(self_, py_name(c"tags".as_ptr()));

				for i in 0..(py_list_len(tag_list)) {
					let tag = py_list_getitem(tag_list, i);

					if py_equal(py_getdict(tag, py_name(c"name".as_ptr())), tag_name) == 1 {
						py_assign(py_retval(), tag);
						return true;
					}
				}

				py_newnone(py_retval());
				true
			}

			py_bindmethod(
				task_type,
				c"get_tag_with_name".as_ptr(),
				Some(task__get_tag_with_name),
			);

			#[allow(non_snake_case)]
			unsafe extern "C" fn task__has_tag_with_name(
				argc: std::os::raw::c_int,
				argv: *mut py_TValue,
			) -> bool {
				if task__get_tag_with_name(argc, argv) {
					py_newbool(py_retval(), !py_isidentical(py_retval(), py_None));
					true
				} else {
					py_newnone(py_retval());
					false
				}
			}

			py_bindmethod(
				task_type,
				c"has_tag_with_name".as_ptr(),
				Some(task__has_tag_with_name),
			);

			// Create the Tag python type
			let tag_type = py_newtype(
				c"Tag".as_ptr(),
				py_totype(py_getbuiltin(py_name(c"object".as_ptr()))),
				null_mut(),
				None,
			);

			py_setglobal(py_name(c"Tag".as_ptr()), py_tpobject(tag_type));

			#[allow(non_snake_case)]
			unsafe extern "C" fn tag____new__(
				_argc: std::os::raw::c_int,
				_argv: *mut py_TValue,
			) -> bool {
				py_newobject(
					py_retval(),
					py_totype(py_getglobal(py_name(c"Tag".as_ptr()))),
					-1,
					0,
				);
				true
			}

			py_bindmethod(tag_type, c"__new__".as_ptr(), Some(tag____new__));

			// Create the Date python type
			let date_type = py_newtype(
				c"Date".as_ptr(),
				py_totype(py_getbuiltin(py_name(c"object".as_ptr()))),
				null_mut(),
				None,
			);

			py_setglobal(py_name(c"Date".as_ptr()), py_tpobject(date_type));

			#[allow(non_snake_case)]
			unsafe extern "C" fn date____new__(
				argc: std::os::raw::c_int,
				argv: *mut py_TValue,
			) -> bool {
				if argc != 4 {
					py_newnone(py_retval());
					return py_exception(
						py_totype(py_getbuiltin(py_name(c"Exception".as_ptr()))),
						c"Expected 3 arguments".as_ptr(),
					);
				}

				let year = ((argv as usize) + size_of::<usize>() * 2) as *mut py_TValue;
				let month = ((argv as usize) + size_of::<usize>() * 4) as *mut py_TValue;
				let day = ((argv as usize) + size_of::<usize>() * 6) as *mut py_TValue;

				for arg in [year, month, day] {
					if !py_istype(arg, py_totype(py_getbuiltin(py_name(c"int".as_ptr())))) {
						py_newnone(py_retval());
						return py_exception(
							py_totype(py_getbuiltin(py_name(c"Exception".as_ptr()))),
							c"Date expects integer arguments".as_ptr(),
						);
					}
				}

				new_py_date_py_values(py_retval(), year, month, day)
			}

			py_bindmethod(date_type, c"__new__".as_ptr(), Some(date____new__));

			#[allow(non_snake_case)]
			unsafe extern "C" fn date____str_____repr__(
				argc: std::os::raw::c_int,
				argv: *mut py_TValue,
			) -> bool {
				if argc != 1 {
					py_newnone(py_retval());
					return py_exception(
						py_totype(py_getbuiltin(py_name(c"Exception".as_ptr()))),
						c"Expected 0 argument".as_ptr(),
					);
				}

				let self_ = argv;

				let mut element_iter = [c"year", c"month", c"day"]
					.into_iter()
					.map(|el| py_toint(py_getdict(self_, py_name(el.as_ptr()))));

				let text = CString::new(format!(
					"{}-{}-{}",
					element_iter.next().unwrap(),
					element_iter.next().unwrap(),
					element_iter.next().unwrap()
				))
				.unwrap();

				py_newstr(py_retval(), text.as_ptr());
				true
			}

			py_bindmethod(date_type, c"__str__".as_ptr(), Some(date____str_____repr__));
			py_bindmethod(
				date_type,
				c"__repr__".as_ptr(),
				Some(date____str_____repr__),
			);
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

	#[allow(dead_code)]
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

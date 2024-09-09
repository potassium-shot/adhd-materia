use pocketpy_sys::*;

use super::PocketPyScriptError;

pub trait IntoPocketPyValue {
	fn push_pocketpy_value(&self);

	fn from_pocketpy_value_ptr(value: *mut py_TValue) -> Result<Self, PocketPyScriptError>
	where
		Self: Sized;

	fn from_pocketpy_value(value: &py_TValue) -> Result<Self, PocketPyScriptError>
	where
		Self: Sized,
	{
		Self::from_pocketpy_value_ptr((value as *const py_TValue).cast_mut())
	}
}

pub type AnyIntoPocketPyValue = Box<dyn IntoPocketPyValue>;

impl IntoPocketPyValue for i64 {
	fn push_pocketpy_value(&self) {
		unsafe {
			let r0 = py_getreg(0);
			py_newint(r0, *self);
			py_push(r0);
		}
	}

	fn from_pocketpy_value_ptr(value: *mut py_TValue) -> Result<Self, PocketPyScriptError>
	where
		Self: Sized,
	{
		unsafe {
			if py_istype(value, py_totype(py_getbuiltin(py_name(c"int".as_ptr())))) {
				Ok(py_toint(value))
			} else {
				Err(PocketPyScriptError::WrongType)
			}
		}
	}
}

impl IntoPocketPyValue for f64 {
	fn push_pocketpy_value(&self) {
		unsafe {
			let r0 = py_getreg(0);
			py_newfloat(r0, *self);
			py_push(r0);
		}
	}

	fn from_pocketpy_value_ptr(value: *mut py_TValue) -> Result<Self, PocketPyScriptError>
	where
		Self: Sized,
	{
		unsafe {
			if py_istype(value, py_totype(py_getbuiltin(py_name(c"float".as_ptr())))) {
				Ok(py_tofloat(value))
			} else {
				Err(PocketPyScriptError::WrongType)
			}
		}
	}
}

impl IntoPocketPyValue for bool {
	fn push_pocketpy_value(&self) {
		unsafe {
			let r0 = py_getreg(0);
			py_newbool(r0, *self);
			py_push(r0);
		}
	}

	fn from_pocketpy_value_ptr(value: *mut py_TValue) -> Result<Self, PocketPyScriptError>
	where
		Self: Sized,
	{
		unsafe {
			if py_istype(value, py_totype(py_getbuiltin(py_name(c"bool".as_ptr())))) {
				Ok(py_tobool(value))
			} else {
				Err(PocketPyScriptError::WrongType)
			}
		}
	}
}

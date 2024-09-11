use std::ffi::CString;

use pocketpy_sys::*;

use crate::{
	tag::{Tag, TagValue},
	task::Task,
};

use super::{py_bindings::new_py_date, PocketPyScriptError};

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

impl IntoPocketPyValue for Task {
	fn push_pocketpy_value(&self) {
		unsafe {
			let r0 = py_getreg(0);
			let r1 = py_getreg(1);

			py_newobject(
				r0,
				py_totype(py_getglobal(py_name(c"Task".as_ptr()))),
				-1,
				0,
			);

			py_newstr(
				r1,
				CString::new(self.name.as_str())
					.unwrap_or_default()
					.as_ptr(),
			);
			py_setdict(r0, py_name(c"name".as_ptr()), r1);

			py_newstr(
				r1,
				CString::new(self.description.as_str())
					.unwrap_or_default()
					.as_ptr(),
			);
			py_setdict(r0, py_name(c"description".as_ptr()), r1);

			py_newlistn(r1, self.tags.len() as i32);

			for (i, tag) in self.tags.iter().enumerate() {
				tag.push_pocketpy_value();
				py_list_setitem(r1, i as i32, py_peek(-1));
				py_pop();
			}

			py_setdict(r0, py_name(c"tags".as_ptr()), r1);

			py_push(r0);
		}
	}

	fn from_pocketpy_value_ptr(value: *mut py_TValue) -> Result<Self, PocketPyScriptError>
	where
		Self: Sized,
	{
		unsafe {
			if py_istype(value, py_totype(py_getglobal(py_name(c"Task".as_ptr())))) {
				unimplemented!("why the fuck you wanna do that right now idiot");
			} else {
				Err(PocketPyScriptError::WrongType)
			}
		}
	}
}

impl IntoPocketPyValue for Tag {
	fn push_pocketpy_value(&self) {
		unsafe {
			let r0 = py_getreg(2);
			let r1 = py_getreg(3);

			py_newobject(r0, py_totype(py_getglobal(py_name(c"Tag".as_ptr()))), -1, 0);

			py_newstr(
				r1,
				CString::new(self.name.as_str())
					.unwrap_or_default()
					.as_ptr(),
			);
			py_setdict(r0, py_name(c"name".as_ptr()), r1);

			match self.value.as_ref() {
				Some(value) => value.push_pocketpy_value(),
				None => py_pushnone(),
			}

			py_setdict(r0, py_name(c"value".as_ptr()), py_peek(-1));
			py_pop();

			py_push(r0);
		}
	}

	fn from_pocketpy_value_ptr(value: *mut py_TValue) -> Result<Self, PocketPyScriptError>
	where
		Self: Sized,
	{
		unsafe {
			if py_istype(value, py_totype(py_getbuiltin(py_name(c"bool".as_ptr())))) {
				unimplemented!("fuck you bitch");
			} else {
				Err(PocketPyScriptError::WrongType)
			}
		}
	}
}

impl IntoPocketPyValue for TagValue {
	fn push_pocketpy_value(&self) {
		unsafe {
			let r0 = py_getreg(4);

			match self {
				TagValue::Bool(b) => py_newbool(r0, *b),
				TagValue::Int(i) => py_newint(r0, *i),
				TagValue::Float(f) => py_newfloat(r0, *f),
				TagValue::Date(d) => {
					new_py_date(r0, d);
				}
				TagValue::Text(t) => {
					py_newstr(r0, CString::new(t.as_str()).unwrap_or_default().as_ptr())
				}
				TagValue::List(l) => {
					py_newlistn(r0, l.len() as i32);

					for (i, v) in l.iter().enumerate() {
						v.push_pocketpy_value();
						py_list_setitem(r0, i as i32, py_peek(-1));
						py_pop();
					}
				}
				TagValue::Dictionary(d) => {
					py_newdict(r0);

					for (k, v) in d.iter() {
						v.push_pocketpy_value();
						py_dict_setitem_by_str(
							r0,
							CString::new(k.as_str()).unwrap_or_default().as_ptr(),
							py_peek(-1),
						);
						py_pop();
					}
				}
				TagValue::Tag(t) => {
					t.push_pocketpy_value();
					return;
				}
			}

			py_push(r0);
		}
	}

	fn from_pocketpy_value_ptr(value: *mut py_TValue) -> Result<Self, PocketPyScriptError>
	where
		Self: Sized,
	{
		unsafe {
			if py_istype(value, py_totype(py_getbuiltin(py_name(c"bool".as_ptr())))) {
				unimplemented!("fuck you bitch");
			} else {
				Err(PocketPyScriptError::WrongType)
			}
		}
	}
}

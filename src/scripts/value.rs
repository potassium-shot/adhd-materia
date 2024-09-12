use std::{
	collections::HashMap,
	ffi::{CStr, CString},
};

use pocketpy_sys::*;

use crate::{
	spytvalue,
	tag::{Tag, TagValue},
	task::Task,
};

use super::{
	py_bindings::{naive_date_from_py_date, new_py_date},
	PocketPyScriptError,
};

pub trait IntoPocketPyValue {
	fn into_pocketpy_value(&self, out: *mut py_TValue);

	fn from_pocketpy_value_ptr(value: *mut py_TValue) -> Result<Self, PocketPyScriptError>
	where
		Self: Sized;
}

pub type AnyIntoPocketPyValue = Box<dyn IntoPocketPyValue>;

impl IntoPocketPyValue for () {
	fn into_pocketpy_value(&self, out: *mut py_TValue) {
		unsafe {
			py_newnone(out);
		}
	}

	fn from_pocketpy_value_ptr(value: *mut py_TValue) -> Result<Self, PocketPyScriptError>
	where
		Self: Sized,
	{
		unsafe {
			if py_isidentical(value, py_None) {
				Ok(())
			} else {
				Err(PocketPyScriptError::WrongType)
			}
		}
	}
}

impl IntoPocketPyValue for i64 {
	fn into_pocketpy_value(&self, out: *mut py_TValue) {
		unsafe {
			py_newint(out, *self);
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
	fn into_pocketpy_value(&self, out: *mut py_TValue) {
		unsafe {
			py_newfloat(out, *self);
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
	fn into_pocketpy_value(&self, out: *mut py_TValue) {
		unsafe {
			py_newbool(out, *self);
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
	fn into_pocketpy_value(&self, out: *mut py_TValue) {
		unsafe {
			spytvalue!(r0);

			py_newobject(
				out,
				py_totype(py_getglobal(py_name(c"Task".as_ptr()))),
				-1,
				0,
			);

			py_newstr(
				r0,
				CString::new(self.name.as_str())
					.unwrap_or_default()
					.as_ptr(),
			);
			py_setdict(out, py_name(c"name".as_ptr()), r0);

			py_newstr(
				r0,
				CString::new(self.description.as_str())
					.unwrap_or_default()
					.as_ptr(),
			);
			py_setdict(out, py_name(c"description".as_ptr()), r0);

			py_newlistn(r0, self.tags.len() as i32);

			for (i, tag) in self.tags.iter().enumerate() {
				spytvalue!(tag_val);
				tag.into_pocketpy_value(tag_val);
				py_list_setitem(r0, i as i32, tag_val);
			}

			py_setdict(out, py_name(c"tags".as_ptr()), r0);
		}
	}

	fn from_pocketpy_value_ptr(value: *mut py_TValue) -> Result<Self, PocketPyScriptError>
	where
		Self: Sized,
	{
		unsafe {
			if py_istype(value, py_totype(py_getglobal(py_name(c"Task".as_ptr())))) {
				let mut task = Task::default();
				task.name = CStr::from_ptr(py_tostr(py_getdict(value, py_name(c"name".as_ptr()))))
					.to_string_lossy()
					.to_string();

				task.description = CStr::from_ptr(py_tostr(py_getdict(
					value,
					py_name(c"description".as_ptr()),
				)))
				.to_string_lossy()
				.to_string();

				let tag_list = py_getdict(value, py_name(c"tags".as_ptr()));
				py_len(tag_list);
				let len = py_toint(py_retval()) as i32;
				task.tags = Vec::with_capacity(len as usize);

				for i in 0..len {
					task.tags
						.push(Tag::from_pocketpy_value_ptr(py_list_getitem(tag_list, i))?);
				}

				Ok(task)
			} else {
				Err(PocketPyScriptError::WrongType)
			}
		}
	}
}

impl IntoPocketPyValue for Tag {
	fn into_pocketpy_value(&self, out: *mut py_TValue) {
		unsafe {
			spytvalue!(r0);

			py_newobject(
				out,
				py_totype(py_getglobal(py_name(c"Tag".as_ptr()))),
				-1,
				0,
			);

			py_newstr(
				r0,
				CString::new(self.name.as_str())
					.unwrap_or_default()
					.as_ptr(),
			);
			py_setdict(out, py_name(c"name".as_ptr()), r0);

			match self.value.as_ref() {
				Some(value) => value.into_pocketpy_value(r0),
				None => py_assign(r0, py_None),
			}

			py_setdict(out, py_name(c"value".as_ptr()), r0);
		}
	}

	fn from_pocketpy_value_ptr(value: *mut py_TValue) -> Result<Self, PocketPyScriptError>
	where
		Self: Sized,
	{
		unsafe {
			if py_istype(value, py_totype(py_getglobal(py_name(c"Tag".as_ptr())))) {
				let val = py_getdict(value, py_name(c"value".as_ptr()));

				Ok(Tag::new(
					CStr::from_ptr(py_tostr(py_getdict(value, py_name(c"name".as_ptr()))))
						.to_string_lossy()
						.to_string(),
					if py_isidentical(val, py_None) {
						None
					} else {
						Some(TagValue::from_pocketpy_value_ptr(val)?)
					},
				))
			} else {
				Err(PocketPyScriptError::WrongType)
			}
		}
	}
}

impl IntoPocketPyValue for TagValue {
	fn into_pocketpy_value(&self, out: *mut py_TValue) {
		unsafe {
			spytvalue!(r0);

			match self {
				TagValue::Bool(b) => py_newbool(out, *b),
				TagValue::Int(i) => py_newint(out, *i),
				TagValue::Float(f) => py_newfloat(out, *f),
				TagValue::Date(d) => {
					new_py_date(out, d);
				}
				TagValue::Text(t) => {
					py_newstr(out, CString::new(t.as_str()).unwrap_or_default().as_ptr())
				}
				TagValue::List(l) => {
					py_newlistn(out, l.len() as i32);

					for (i, v) in l.iter().enumerate() {
						v.into_pocketpy_value(r0);
						py_list_setitem(out, i as i32, r0);
					}
				}
				TagValue::Dictionary(d) => {
					py_newdict(out);

					for (k, v) in d.iter() {
						v.into_pocketpy_value(r0);
						py_dict_setitem_by_str(
							out,
							CString::new(k.as_str()).unwrap_or_default().as_ptr(),
							r0,
						);
					}
				}
				TagValue::Tag(t) => {
					t.into_pocketpy_value(out);
					return;
				}
			}
		}
	}

	fn from_pocketpy_value_ptr(value: *mut py_TValue) -> Result<Self, PocketPyScriptError>
	where
		Self: Sized,
	{
		unsafe {
			if py_istype(value, py_totype(py_getbuiltin(py_name(c"bool".as_ptr())))) {
				Ok(Self::Bool(py_tobool(value)))
			} else if py_istype(value, py_totype(py_getbuiltin(py_name(c"int".as_ptr())))) {
				Ok(Self::Int(py_toint(value)))
			} else if py_istype(value, py_totype(py_getbuiltin(py_name(c"float".as_ptr())))) {
				Ok(Self::Float(py_tofloat(value)))
			} else if py_istype(value, py_totype(py_getglobal(py_name(c"Date".as_ptr())))) {
				Ok(Self::Date(naive_date_from_py_date(value)?))
			} else if py_istype(value, py_totype(py_getbuiltin(py_name(c"str".as_ptr())))) {
				Ok(Self::Text(
					CStr::from_ptr(py_tostr(value))
						.to_string_lossy()
						.to_string(),
				))
			} else if py_istype(value, py_totype(py_getbuiltin(py_name(c"list".as_ptr())))) {
				let mut list = Vec::new();
				py_len(value);

				for i in 0..(py_toint(py_retval()) as i32) {
					list.push(TagValue::from_pocketpy_value_ptr(py_list_getitem(
						value, i,
					))?);
				}

				Ok(Self::List(list))
			} else if py_istype(value, py_totype(py_getbuiltin(py_name(c"dict".as_ptr())))) {
				let mut dict: (HashMap<String, TagValue>, Result<(), PocketPyScriptError>) =
					(HashMap::new(), Ok(()));

				unsafe extern "C" fn unpack_py_dict(
					key: *mut py_TValue,
					value: *mut py_TValue,
					ctx: *mut std::ffi::c_void,
				) -> bool {
					let ctx =
						ctx.cast::<(HashMap<String, TagValue>, Result<(), PocketPyScriptError>)>();

					if !py_istype(key, py_totype(py_getbuiltin(py_name(c"str".as_ptr())))) {
						(*ctx).1 = Err(PocketPyScriptError::DictionaryKeyIsNotString);
						return false;
					}

					let key = CStr::from_ptr(py_tostr(key)).to_string_lossy().to_string();

					let value = match TagValue::from_pocketpy_value_ptr(value) {
						Ok(v) => v,
						Err(e) => {
							(*ctx).1 = Err(e);
							return false;
						}
					};

					(*ctx).0.insert(key, value);
					true
				}

				py_dict_apply(
					value,
					Some(unpack_py_dict),
					((&mut dict)
						as *mut (HashMap<String, TagValue>, Result<(), PocketPyScriptError>))
						.cast::<std::ffi::c_void>(),
				);

				dict.1?;
				Ok(Self::Dictionary(dict.0))
			} else if py_istype(value, py_totype(py_getglobal(py_name(c"Tag".as_ptr())))) {
				Ok(Self::Tag(Box::new(Tag::from_pocketpy_value_ptr(value)?)))
			} else {
				Err(PocketPyScriptError::WrongType)
			}
		}
	}
}

#![allow(non_snake_case)]

use std::{
	ffi::{CStr, CString},
	ptr::null_mut,
};

use chrono::Datelike;
use pocketpy_sys::*;

pub unsafe fn initialize_bindings() {
	// Create the Task python type
	let task_type = py_newtype(
		c"Task".as_ptr(),
		py_totype(py_getbuiltin(py_name(c"object".as_ptr()))),
		null_mut(),
		None,
	);

	py_setglobal(py_name(c"Task".as_ptr()), py_tpobject(task_type));
	py_bindmethod(task_type, c"__new__".as_ptr(), Some(task____new__));
	py_bindmethod(
		task_type,
		c"get_tag_with_name".as_ptr(),
		Some(task__get_tag_with_name),
	);
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
	py_bindmethod(tag_type, c"__new__".as_ptr(), Some(tag____new__));

	// Create the Date python type
	let date_type = py_newtype(
		c"Date".as_ptr(),
		py_totype(py_getbuiltin(py_name(c"object".as_ptr()))),
		null_mut(),
		None,
	);

	py_setglobal(py_name(c"Date".as_ptr()), py_tpobject(date_type));
	py_bindmethod(date_type, c"__new__".as_ptr(), Some(date____new__));
	py_bindmethod(date_type, c"__str__".as_ptr(), Some(date____str_____repr__));
	py_bindmethod(
		date_type,
		c"__repr__".as_ptr(),
		Some(date____str_____repr__),
	);

	let r0 = py_getreg(0);
	py_newnativefunc(r0, Some(run_standalone_script));
	py_setglobal(py_name(c"run_standalone_script".as_ptr()), r0);
}

unsafe extern "C" fn task____new__(_argc: std::os::raw::c_int, _argv: *mut py_TValue) -> bool {
	py_newobject(
		py_retval(),
		py_totype(py_getglobal(py_name(c"Task".as_ptr()))),
		-1,
		0,
	);
	true
}

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

unsafe extern "C" fn tag____new__(_argc: std::os::raw::c_int, _argv: *mut py_TValue) -> bool {
	py_newobject(
		py_retval(),
		py_totype(py_getglobal(py_name(c"Tag".as_ptr()))),
		-1,
		0,
	);
	true
}

unsafe extern "C" fn date____new__(argc: std::os::raw::c_int, argv: *mut py_TValue) -> bool {
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

unsafe extern "C" fn run_standalone_script(
	argc: std::os::raw::c_int,
	argv: *mut py_TValue,
) -> bool {
	if argc != 1 {
		py_newnone(py_retval());
		return py_exception(
			py_totype(py_getbuiltin(py_name(c"Exception".as_ptr()))),
			c"Expected 1 argument".as_ptr(),
		);
	}

	if !py_istype(argv, py_totype(py_getbuiltin(py_name(c"str".as_ptr())))) {
		py_newnone(py_retval());
		return py_exception(
			py_totype(py_getbuiltin(py_name(c"Exception".as_ptr()))),
			c"Expected string argument".as_ptr(),
		);
	}

	let name = CStr::from_ptr(py_tostr(argv)).to_string_lossy();

	crate::app::push_script_to_waitlist(name.to_string());

	py_newnone(py_retval());
	true
}

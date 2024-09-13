#![allow(non_snake_case)]

use std::{
	collections::VecDeque,
	ffi::{CStr, CString},
	ptr::null_mut,
	sync::Mutex,
};

use chrono::Datelike;
use pocketpy_sys::*;

use crate::{session::Session, task::Task};

use super::{value::IntoPocketPyValue, PocketPyScriptError};

static NEW_TASKS_WAITLIST: Mutex<VecDeque<Task>> = Mutex::new(VecDeque::new());

pub fn new_tasks_waitlist_next() -> Option<Task> {
	NEW_TASKS_WAITLIST.lock().unwrap().pop_front()
}

#[macro_export]
macro_rules! spytvalue {
	($name: ident) => {
		let mut __temp_s_tvalue: [u8; 16] = std::mem::MaybeUninit::zeroed().assume_init();
		let $name = __temp_s_tvalue
			.as_mut_ptr()
			.cast::<pocketpy_sys::py_TValue>();
	};
}

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

	// Create the TaskRef python type
	let task_ref_type = py_newtype(
		c"TaskRef".as_ptr(),
		py_totype(py_getbuiltin(py_name(c"object".as_ptr()))),
		null_mut(),
		None,
	);
	py_setglobal(py_name(c"TaskRef".as_ptr()), py_tpobject(task_ref_type));
	py_bindmethod(task_ref_type, c"__new__".as_ptr(), Some(task_ref____new__));
	py_bindmethod(task_ref_type, c"__eq__".as_ptr(), Some(task_ref____eq__));
	py_bindmethod(task_ref_type, c"__str__".as_ptr(), Some(task_ref____str__));
	py_bindmethod(
		task_ref_type,
		c"__repr__".as_ptr(),
		Some(task_ref____repr__),
	);

	// Create the Session python type
	let session_type = py_newtype(
		c"Session".as_ptr(),
		py_totype(py_getbuiltin(py_name(c"object".as_ptr()))),
		null_mut(),
		None,
	);
	py_setglobal(py_name(c"Session".as_ptr()), py_tpobject(session_type));
	py_bindfunc(
		py_tpobject(session_type),
		c"get".as_ptr(),
		Some(session__get),
	);

	spytvalue!(r0);
	py_newnativefunc(r0, Some(run_standalone_script));
	py_setglobal(py_name(c"run_standalone_script".as_ptr()), r0);
	py_newnativefunc(r0, Some(today));
	py_setglobal(py_name(c"today".as_ptr()), r0);
	py_newnativefunc(r0, Some(add_task));
	py_setglobal(py_name(c"add_task".as_ptr()), r0);
}

unsafe extern "C" fn task____new__(argc: std::os::raw::c_int, argv: *mut py_TValue) -> bool {
	if argc != 2 {
		py_newnone(py_retval());
		return py_exception(
			py_totype(py_getbuiltin(py_name(c"Exception".as_ptr()))),
			c"Expected 1 argument".as_ptr(),
		);
	}

	spytvalue!(r0);
	let name = ((argv as usize) + size_of::<usize>() * 2) as *mut py_TValue;

	py_newobject(
		py_retval(),
		py_totype(py_getglobal(py_name(c"Task".as_ptr()))),
		-1,
		0,
	);

	py_setdict(py_retval(), py_name(c"name".as_ptr()), name);
	py_newstr(r0, c"".as_ptr());
	py_setdict(py_retval(), py_name(c"description".as_ptr()), r0);
	py_newlist(r0);
	py_setdict(py_retval(), py_name(c"tags".as_ptr()), r0);
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

unsafe extern "C" fn tag____new__(argc: std::os::raw::c_int, argv: *mut py_TValue) -> bool {
	let (name, value) = match argc {
		2 => {
			let name = ((argv as usize) + size_of::<usize>() * 2) as *mut py_TValue;

			if !py_istype(name, py_totype(py_getbuiltin(py_name(c"str".as_ptr())))) {
				py_newnone(py_retval());
				return py_exception(
					py_totype(py_getbuiltin(py_name(c"Exception".as_ptr()))),
					c"Tag name should be a string".as_ptr(),
				);
			}

			(name, py_None)
		}
		3 => {
			let name = ((argv as usize) + size_of::<usize>() * 2) as *mut py_TValue;

			if !py_istype(name, py_totype(py_getbuiltin(py_name(c"str".as_ptr())))) {
				py_newnone(py_retval());
				return py_exception(
					py_totype(py_getbuiltin(py_name(c"Exception".as_ptr()))),
					c"Tag name should be a string".as_ptr(),
				);
			}

			let value = ((argv as usize) + size_of::<usize>() * 4) as *mut py_TValue;
			(name, value)
		}
		_ => {
			py_newnone(py_retval());
			return py_exception(
				py_totype(py_getbuiltin(py_name(c"Exception".as_ptr()))),
				c"Expected 1 or 2 argument".as_ptr(),
			);
		}
	};

	py_newobject(
		py_retval(),
		py_totype(py_getglobal(py_name(c"Tag".as_ptr()))),
		-1,
		0,
	);
	py_setdict(py_retval(), py_name(c"name".as_ptr()), name);
	py_setdict(py_retval(), py_name(c"value".as_ptr()), value);
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

pub fn naive_date_from_py_date(
	value: *mut py_TValue,
) -> Result<chrono::NaiveDate, PocketPyScriptError> {
	unsafe {
		chrono::NaiveDate::from_ymd_opt(
			py_toint(py_getdict(value, py_name(c"year".as_ptr()))) as i32,
			py_toint(py_getdict(value, py_name(c"month".as_ptr()))) as u32,
			py_toint(py_getdict(value, py_name(c"day".as_ptr()))) as u32,
		)
		.ok_or(PocketPyScriptError::DateOutOfBounds)
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

unsafe extern "C" fn task_ref____new__(argc: std::os::raw::c_int, argv: *mut py_TValue) -> bool {
	if argc != 2 {
		py_newnone(py_retval());
		return py_exception(
			py_totype(py_getbuiltin(py_name(c"Exception".as_ptr()))),
			c"Expected 1 argument".as_ptr(),
		);
	}

	let arg = ((argv as usize) + size_of::<usize>() * 2) as *mut py_TValue;

	if !py_istype(arg, py_totype(py_getbuiltin(py_name(c"str".as_ptr())))) {
		py_newnone(py_retval());
		return py_exception(
			py_totype(py_getbuiltin(py_name(c"Exception".as_ptr()))),
			c"Expected string argument".as_ptr(),
		);
	}

	py_newobject(
		py_retval(),
		py_totype(py_getglobal(py_name(c"TaskRef".as_ptr()))),
		1,
		0,
	);
	py_setslot(py_retval(), 0, arg);
	true
}

unsafe extern "C" fn task_ref____eq__(argc: std::os::raw::c_int, argv: *mut py_TValue) -> bool {
	if argc != 2 {
		py_newnone(py_retval());
		return py_exception(
			py_totype(py_getbuiltin(py_name(c"Exception".as_ptr()))),
			c"Expected 2 argument".as_ptr(),
		);
	}

	let arg2 = ((argv as usize) + size_of::<usize>() * 2) as *mut py_TValue;

	py_newbool(
		py_retval(),
		py_typeof(argv) == py_typeof(arg2)
			&& py_equal(py_getslot(argv, 0), py_getslot(arg2, 0)) == 1,
	);
	true
}

unsafe extern "C" fn task_ref____str__(argc: std::os::raw::c_int, argv: *mut py_TValue) -> bool {
	if argc != 1 {
		py_newnone(py_retval());
		return py_exception(
			py_totype(py_getbuiltin(py_name(c"Exception".as_ptr()))),
			c"Expected 0 argument".as_ptr(),
		);
	}

	py_assign(py_retval(), py_getslot(argv, 0));
	true
}

unsafe extern "C" fn task_ref____repr__(argc: std::os::raw::c_int, argv: *mut py_TValue) -> bool {
	if argc != 1 {
		py_newnone(py_retval());
		return py_exception(
			py_totype(py_getbuiltin(py_name(c"Exception".as_ptr()))),
			c"Expected 0 argument".as_ptr(),
		);
	}

	let repr = CString::new(format!(
		"TaskRef({})",
		CStr::from_ptr(py_tostr(py_getslot(argv, 0))).to_string_lossy()
	))
	.unwrap();
	py_newstr(py_retval(), repr.as_ptr());
	true
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

unsafe extern "C" fn today(_argc: std::os::raw::c_int, _argv: *mut py_TValue) -> bool {
	new_py_date(py_retval(), &chrono::Local::now().date_naive());
	true
}

unsafe extern "C" fn session__get(_argc: std::os::raw::c_int, _argv: *mut py_TValue) -> bool {
	let session = Session::current();
	spytvalue!(r0);
	spytvalue!(r1);

	py_newobject(
		py_retval(),
		py_totype(py_getglobal(py_name(c"Session".as_ptr()))),
		-1,
		0,
	);

	new_py_date(r0, &session.last_session);
	py_setdict(py_retval(), py_name(c"last_session".as_ptr()), r0);

	py_newlist(r0);
	session.set_filters.iter().for_each(|s| {
		let cstring = CString::new(s.as_str()).unwrap();
		py_newstr(r1, cstring.as_ptr());
		py_list_append(r0, r1);
	});
	py_setdict(py_retval(), py_name(c"set_filters".as_ptr()), r0);

	py_newlist(r0);
	session.set_sortings.iter().for_each(|s| {
		let cstring = CString::new(s.as_str()).unwrap();
		py_newstr(r1, cstring.as_ptr());
		py_list_append(r0, r1);
	});
	py_setdict(py_retval(), py_name(c"set_sortings".as_ptr()), r0);

	true
}

unsafe extern "C" fn add_task(argc: std::os::raw::c_int, argv: *mut py_TValue) -> bool {
	if argc != 1 {
		py_newnone(py_retval());
		return py_exception(
			py_totype(py_getbuiltin(py_name(c"Exception".as_ptr()))),
			c"Expected 1 argument".as_ptr(),
		);
	}

	if !py_istype(argv, py_totype(py_getglobal(py_name(c"Task".as_ptr())))) {
		py_newnone(py_retval());
		return py_exception(
			py_totype(py_getbuiltin(py_name(c"Exception".as_ptr()))),
			c"Expected Task argument".as_ptr(),
		);
	}

	match Task::from_pocketpy_value_ptr(argv) {
		Ok(task) => {
			NEW_TASKS_WAITLIST.lock().unwrap().push_back(task);
		}
		Err(e) => {
			py_newnone(py_retval());
			let err_cstring = CString::new(e.to_string()).unwrap();
			return py_exception(
				py_totype(py_getbuiltin(py_name(c"Exception".as_ptr()))),
				err_cstring.as_ptr(),
			);
		}
	}
	true
}

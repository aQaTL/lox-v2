use crate::memory;
use std::ptr;

pub type Value = f64;

pub struct ValueArray {
	pub capacity: usize,
	pub count: usize,
	pub values: *mut Value,
}

impl Default for ValueArray {
	fn default() -> Self {
		ValueArray {
			capacity: 0,
			count: 0,
			values: ptr::null_mut(),
		}
	}
}

impl ValueArray {
	pub fn init(array: *mut ValueArray) {
		unsafe {
			*array = ValueArray::default();
		}
	}

	pub fn write(array: *mut ValueArray, value: Value) {
		unsafe {
			if (*array).capacity < (*array).count + 1 {
				let old_capacity = (*array).capacity;
				(*array).capacity = memory::grow_capacity(old_capacity);
				(*array).values =
					memory::grow_array::<Value>((*array).values, old_capacity, (*array).capacity);
			}

			*((*array).values.add((*array).count)) = value;
			(*array).count += 1;
		}
	}

	pub fn free(array: *mut ValueArray) {
		unsafe {
			memory::free_array::<Value>((*array).values, (*array).capacity);
			ValueArray::init(array)
		}
	}
}

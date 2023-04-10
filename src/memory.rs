use std::alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout};

pub const fn grow_capacity(capacity: usize) -> usize {
	if capacity < 8 {
		8
	} else {
		capacity * 2
	}
}

pub fn grow_array<T>(array: *mut T, old_count: usize, new_count: usize) -> *mut T {
	reallocate(
		array.cast::<u8>(),
		std::mem::size_of::<T>() * old_count,
		std::mem::size_of::<T>() * new_count,
	)
	.cast::<T>()
}

pub fn free_array<T>(array: *mut T, old_count: usize) {
	reallocate(array.cast::<u8>(), std::mem::size_of::<T>() * old_count, 0);
}

fn reallocate(pointer: *mut u8, old_size: usize, new_size: usize) -> *mut u8 {
	unsafe {
		if new_size == 0 {
			dealloc(pointer, Layout::array::<u8>(old_size).unwrap());
			return std::ptr::null_mut();
		}

		let result = if old_size == 0 {
			alloc(Layout::array::<u8>(new_size).unwrap())
		} else {
			realloc(pointer, Layout::array::<u8>(old_size).unwrap(), new_size)
		};

		if result.is_null() {
			handle_alloc_error(Layout::array::<u8>(old_size).unwrap())
		}
		result
	}
}

use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::ptr;

use crate::object::Object;
use crate::value::Value;

pub struct Table {
	entries: *mut Entry,
	len: usize,
	capacity: usize,
}

#[derive(Clone)]
struct Entry {
	key: *mut Object,
	value: Value,
}

impl Drop for Table {
	fn drop(&mut self) {
		free_array(self.entries, self.capacity);
		self.entries = ptr::null_mut();
	}
}

impl Table {
	const MAX_LOAD: f64 = 0.75;

	pub fn new() -> Self {
		Table {
			entries: ptr::null_mut(),
			len: 0,
			capacity: 0,
		}
	}

	pub fn get(&mut self, key: *mut Object) -> Option<&Value> {
		if self.len == 0 {
			return None;
		}

		let entry = find_entry(self.entries, self.capacity, key);
		if entry.key.is_null() {
			return None;
		}

		unsafe { Some(&(*entry).value) }
	}

	pub fn set(&mut self, key: *mut Object, value: Value) -> bool {
		if self.len + 1 > ((self.capacity as f64) * Table::MAX_LOAD) as usize {
			let capacity = grow_capacity(self.capacity);
			self.adjust_capacity(capacity);
		}

		let entry = find_entry(self.entries, self.capacity, key);
		let is_new_key = unsafe { (*entry).key.is_null() };
		if is_new_key {
			self.len += 1;
		}

		unsafe {
			(*entry).key = key;
			(*entry).value = value;
		}

		is_new_key
	}

	pub fn delete(&mut self, key: *mut Object) -> bool {
		todo!()
	}

	pub fn add_all(&mut self, dest: &mut Table) {
		for i in 0..self.capacity {
			let entry = unsafe { &mut *self.entries.add(i) };
			if entry.key.is_null() {
				dest.set(entry.key, entry.value.clone());
			}
		}
	}

	fn adjust_capacity(&mut self, new_capacity: usize) {
		let entries = allocate_array::<Entry>(new_capacity);
		for i in 0..new_capacity {
			unsafe {
				*entries.add(i) = Entry {
					key: ptr::null_mut(),
					value: Value::Nil,
				};
			}
		}

		for i in 0..self.capacity {
			let entry = unsafe { &mut *self.entries.add(i) };
			if entry.key.is_null() {
				continue;
			}
			let dest = find_entry(entries, new_capacity, entry.key);
			unsafe {
				(*dest).key = entry.key;
				(*dest).value = std::mem::take(&mut entry.value);
			}
		}

		free_array(self.entries, self.capacity);
		self.entries = entries;
		self.capacity = new_capacity;
	}
}

impl Default for Table {
	fn default() -> Self {
		Self::new()
	}
}

/// Hashes a string using the "FNV-1a" algorithm
pub fn hash_str(s: &str) -> u32 {
	let mut hash: u32 = 2166136261;
	for b in s.as_bytes() {
		hash ^= *b as u32;
		hash = hash.wrapping_mul(16777619);
	}
	hash
}

fn find_entry(entries: *mut Entry, capacity: usize, key: *mut Object) -> *mut Entry {
	let mut index = unsafe { (*key).hash % (capacity as u32) };
	loop {
		unsafe {
			let entry = entries.add(index as usize);
			if (*entry).key == key || (*entry).key.is_null() {
				return entry;
			}
			index = (index + 1) % (capacity as u32);
		}
	}
}

const fn grow_capacity(capacity: usize) -> usize {
	if capacity < 8 {
		8
	} else {
		capacity * 2
	}
}

fn allocate_array<T>(capacity: usize) -> *mut T {
	unsafe {
		let result = alloc(Layout::array::<T>(capacity).unwrap()).cast::<T>();
		if result.is_null() {
			handle_alloc_error(Layout::array::<u8>(capacity).unwrap())
		}
		result
	}
}

pub fn free_array<T>(array: *mut T, capacity: usize) {
	unsafe { dealloc(array.cast::<u8>(), Layout::array::<T>(capacity).unwrap()) };
}

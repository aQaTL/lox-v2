use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use std::ptr;

use crate::object::ObjString;
use crate::value::Value;

/// Hand rolled HashMap<ObjString, Value>
pub struct Table {
	entries: *mut Entry,
	len: usize,
	capacity: usize,
}

unsafe impl Sync for Table {}

#[derive(Clone)]
struct Entry {
	key: *mut ObjString,
	value: Value,
}

impl Drop for Table {
	fn drop(&mut self) {
		free_array(self.entries, self.capacity);
		self.entries = ptr::null_mut();
	}
}

impl Default for Table {
	fn default() -> Self {
		Self::new()
	}
}

impl Table {
	const MAX_LOAD: f64 = 0.75;

	pub const fn new() -> Self {
		Table {
			entries: ptr::null_mut(),
			len: 0,
			capacity: 0,
		}
	}

	pub fn get(&mut self, key: *mut ObjString) -> Option<&Value> {
		if self.len == 0 {
			return None;
		}

		let entry = find_entry(self.entries, self.capacity, key);
		unsafe {
			if (*entry).key.is_null() {
				return None;
			}

			Some(&(*entry).value)
		}
	}

	pub fn set(&mut self, key: *mut ObjString, value: Value) -> bool {
		if self.len + 1 > ((self.capacity as f64) * Table::MAX_LOAD) as usize {
			let capacity = grow_capacity(self.capacity);
			self.adjust_capacity(capacity);
		}

		let entry = find_entry(self.entries, self.capacity, key);
		unsafe {
			let is_new_key = (*entry).key.is_null();
			if is_new_key && (*entry).value == Value::Nil {
				self.len += 1;
			}

			(*entry).key = key;
			(*entry).value = value;

			is_new_key
		}
	}

	pub fn delete(&mut self, key: *mut ObjString) -> bool {
		if self.len == 0 {
			return false;
		}

		let entry = find_entry(self.entries, self.capacity, key);
		if entry.is_null() {
			return false;
		}
		unsafe {
			(*entry).key = ptr::null_mut();
			(*entry).value = Value::Bool(true);
		}
		true
	}

	pub fn add_all(&mut self, dest: &mut Table) {
		for i in 0..self.capacity {
			let entry = unsafe { &mut *self.entries.add(i) };
			if entry.key.is_null() {
				dest.set(entry.key, entry.value.clone());
			}
		}
	}

	pub fn find_string(&mut self, str: impl AsRef<str>, hash: u32) -> Option<*mut ObjString> {
		if self.len == 0 {
			return None;
		}

		let str = str.as_ref();
		let mut idx = (hash as usize) % self.capacity;
		loop {
			unsafe {
				let entry = self.entries.add(idx);
				if (*entry).key.is_null() {
					if matches!((*entry).value, Value::Nil) {
						return None;
					}
				} else if (*(*entry).key).len() == str.len()
					&& (*(*entry).key).hash == hash
					&& (*(*entry).key).as_str() == str
				{
					return Some((*entry).key);
				}

				idx = (idx + 1) % self.capacity;
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

		self.len = 0;
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
			self.len += 1;
		}

		free_array(self.entries, self.capacity);
		self.entries = entries;
		self.capacity = new_capacity;
	}
}

/// Hashes a byte slice using the "FNV-1a" algorithm
pub fn hash(s: impl AsRef<[u8]>) -> u32 {
	let mut hash: u32 = 2166136261;
	for b in s.as_ref() {
		hash ^= *b as u32;
		hash = hash.wrapping_mul(16777619);
	}
	hash
}

fn find_entry(entries: *mut Entry, capacity: usize, key: *mut ObjString) -> *mut Entry {
	let mut index = unsafe { (*key).hash % (capacity as u32) };
	let mut tombstone = ptr::null_mut::<Entry>();
	loop {
		unsafe {
			let entry = entries.add(index as usize);
			if (*entry).key.is_null() {
				if (*entry).value == Value::Nil {
					return if tombstone.is_null() {
						entry
					} else {
						tombstone
					};
				} else {
					tombstone = entry;
				}
			//This actually compares pointers, not the Objects
			} else if (*entry).key == key {
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

#[cfg(test)]
mod tests {
	use super::Table;
	use crate::object::{Allocator, ObjString};
	use crate::value::Value;

	#[test]
	fn insert_and_get() {
		let mut allocator = Allocator::default();
		let mut table = Table::default();

		{
			let key = allocator
				.take_string("ala".to_string())
				.cast::<ObjString>();

			let value = Value::Object(allocator.take_string("ma kota".to_string()));
			table.set(key, value);
		}

		let key = allocator.copy_string("ala").cast::<ObjString>();
		let value = table.get(key);
		match value {
			Some(Value::Object(value)) => unsafe {
				assert_eq!((**value).as_string().unwrap().as_str(), "ma kota");
			},
			_ => panic!("unexpected value {value:?}"),
		}
	}
}

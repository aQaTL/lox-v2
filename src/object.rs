#![allow(clippy::result_unit_err, clippy::not_unsafe_ptr_arg_deref)]

use crate::table::{hash, Table};
use crate::value::Value;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

pub struct Allocator {
	objects: AtomicPtr<Object>,
	strings: Table,
}

impl Default for Allocator {
	fn default() -> Self {
		Allocator {
			objects: AtomicPtr::new(ptr::null_mut()),
			strings: Table::default(),
		}
	}
}

impl Drop for Allocator {
	fn drop(&mut self) {
		self.free();
	}
}

impl Allocator {
	fn put_obj<T: IsObject>(&mut self, obj: T) -> *mut Object {
		let obj = T::into_object(Box::into_raw(Box::new(obj)));
		unsafe {
			(*obj).next = self.objects.load(Ordering::Acquire);
		}
		self.objects.store(obj, Ordering::Release);
		obj
	}

	/// Use only when you're sure that the `str` is unique (hasn't been allocated already).
	pub fn new_string_object(&mut self, str: String) -> *mut Object {
		let hash = hash(&str);
		let obj = ObjString {
			obj: Object {
				kind: ObjectKind::String,
				next: ptr::null_mut(),
			},
			str,
			hash,
		};
		let obj = self.put_obj(obj);
		self.strings.set(obj.cast::<ObjString>(), Value::Nil);
		obj
	}

	pub fn free(&mut self) {
		// Free objects
		unsafe {
			// This doesn't seem safe in a multithreaded context. Let's hope it won't be used in
			// one.
			let mut object = self.objects.load(Ordering::Relaxed);
			while !object.is_null() {
				let next = (*object).next;
				drop(Box::from_raw(object));
				object = next;
			}
		}
	}

	pub fn copy_object(&mut self, obj: *mut Object) -> *mut Object {
		let obj_ref = unsafe { &*obj };
		match &obj_ref.kind {
			ObjectKind::String => {
				let str: &ObjString = unsafe { obj_ref.as_obj_string_unchecked() };
				let hash = hash(str);
				if let Some(interned) = self.strings.find_string(str, hash) {
					return ObjString::into_object(interned);
				}
				self.new_string_object(str.str.clone())
			}
		}
	}

	pub fn copy_string(&mut self, str: &str) -> *mut Object {
		let hash = hash(str);
		if let Some(interned) = self.strings.find_string(str, hash) {
			return ObjString::into_object(interned);
		}
		self.new_string_object(str.to_string())
	}
}

// Marker trait saying that the a given T has repr(C) and [Object] as a first field
trait IsObject {
	fn into_object(this: *mut Self) -> *mut Object;
}

#[derive(Debug)]
#[repr(C)]
pub struct Object {
	pub kind: ObjectKind,
	pub next: *mut Object,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum ObjectKind {
	String,
}

#[repr(C)]
pub struct ObjString {
	obj: Object,
	str: String,
	pub hash: u32,
}

impl IsObject for ObjString {
	fn into_object(this: *mut Self) -> *mut Object {
		unsafe {
			// Asserts that [Object] is the first field in the struct
			debug_assert!(ptr::eq(
				(&mut (*this).obj) as *mut Object,
				this.cast::<Object>()
			));
			(&mut (*this).obj) as *mut Object
		}
	}
}

impl ObjString {
	pub fn as_str(&self) -> &str {
		self
	}
}

impl Display for ObjString {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.str.fmt(f)
	}
}

impl AsRef<str> for ObjString {
	fn as_ref(&self) -> &str {
		&self.str
	}
}

impl AsRef<[u8]> for ObjString {
	fn as_ref(&self) -> &[u8] {
		self.str.as_bytes()
	}
}

impl Deref for ObjString {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		&self.str
	}
}

impl Object {
	pub fn as_obj_string(&self) -> Result<&ObjString, ()> {
		match self.kind {
			ObjectKind::String => {
				let obj_str: &ObjString = unsafe { &*(self as *const Self).cast::<ObjString>() };
				Ok(obj_str)
			}
		}
	}

	/// # Safety
	/// TODO(aqatl): Add safety doc
	pub unsafe fn as_obj_string_unchecked(&self) -> &ObjString {
		&*(self as *const Self).cast::<ObjString>()
	}

	/// # Safety
	/// TODO(aqatl): Add safety doc
	pub unsafe fn as_mut_obj_string_unchecked(&mut self) -> &mut ObjString {
		&mut *(self as *mut Self).cast::<ObjString>()
	}

	/// # Safety
	/// TODO(aqatl): Add safety doc
	pub fn as_string(&self) -> Result<&String, ()> {
		let obj_str = self.as_obj_string()?;
		Ok(&obj_str.str)
	}

	/// # Safety
	/// TODO(aqatl): Add safety doc
	pub unsafe fn as_string_unchecked(&self) -> &String {
		let obj_str: &ObjString = unsafe { &*(self as *const Self).cast::<ObjString>() };
		&obj_str.str
	}

	pub fn as_obj_string_ptr_mut(this: *mut Self) -> Result<*mut ObjString, ()> {
		unsafe {
			match (*this).kind {
				ObjectKind::String => {
					let obj_str = this.cast::<ObjString>();
					Ok(obj_str)
				}
			}
		}
	}
}

impl Display for Object {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match &self.kind {
			ObjectKind::String => Display::fmt(unsafe { self.as_string_unchecked() }, f),
		}
	}
}

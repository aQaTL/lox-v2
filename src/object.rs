use crate::table::hash_str;
use std::fmt::{Display, Formatter};
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

static ALLOCATOR: Allocator = Allocator {
	objects: AtomicPtr::new(ptr::null_mut()),
};

pub struct Allocator {
	objects: AtomicPtr<Object>,
}

impl Default for Allocator {
	fn default() -> Self {
		Allocator {
			objects: AtomicPtr::new(ptr::null_mut()),
		}
	}
}

impl Drop for Allocator {
	fn drop(&mut self) {
		self.free();
	}
}

impl Allocator {
	pub fn global() -> &'static Allocator {
		&ALLOCATOR
	}

	pub fn new_global_object(obj: ObjectKind) -> *mut Object {
		Allocator::global().new_object(obj)
	}

	pub fn new_object(&self, obj: ObjectKind) -> *mut Object {
		let hash = hash_str(obj.as_str());
		let object = Box::new(Object {
			kind: obj,
			hash,
			next: self.objects.load(Ordering::Acquire),
		});
		let object = Box::into_raw(object);
		self.objects.store(object, Ordering::Release);
		object
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
}

#[derive(Debug, Clone)]
pub struct Object {
	pub kind: ObjectKind,
	pub hash: u32,
	pub next: *mut Object,
}

#[derive(Debug, Clone)]
pub enum ObjectKind {
	String(String),
}

impl ObjectKind {
	pub fn string(s: String) -> ObjectKind {
		ObjectKind::String(s)
	}
}

impl PartialEq for ObjectKind {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::String(a), Self::String(b)) => a == b,
		}
	}
}

impl Display for Object {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match &self.kind {
			ObjectKind::String(s) => std::fmt::Display::fmt(s, f),
		}
	}
}

impl ObjectKind {
	pub fn as_str(&self) -> &str {
		match self {
			ObjectKind::String(ref s) => s.as_str(),
			_ => panic!("expected string object"),
		}
	}
}

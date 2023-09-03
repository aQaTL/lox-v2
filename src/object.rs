use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct Object {
	pub kind: ObjectKind,
	pub next: *mut Object,
}

#[derive(Debug, Clone)]
pub enum ObjectKind {
	String(String),
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

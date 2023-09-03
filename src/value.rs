use std::fmt::{Display, Formatter};

#[derive(Default, Clone, Debug)]
pub enum Value {
	#[default]
	Nil,
	Bool(bool),
	Number(f64),
}

impl Value {
	pub fn is_falsey(&self) -> bool {
		matches!(self, Self::Nil | Self::Bool(false))
	}
}

impl PartialEq for Value {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Value::Nil, Value::Nil) => true,
			(Value::Bool(a), Value::Bool(b)) => a == b,
			(Value::Number(a), Value::Number(b)) => a == b,
			_ => false,
		}
	}
}

impl Display for Value {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Nil => write!(f, "nil"),
			Self::Bool(b) => std::fmt::Display::fmt(b, f),
			Self::Number(n) => std::fmt::Display::fmt(n, f),
		}
	}
}

impl From<f64> for Value {
	fn from(n: f64) -> Self {
		Value::Number(n)
	}
}

impl From<bool> for Value {
	fn from(b: bool) -> Self {
		Value::Bool(b)
	}
}

impl From<()> for Value {
	fn from(_: ()) -> Self {
		Value::Nil
	}
}

impl TryFrom<Value> for f64 {
	type Error = Value;

	fn try_from(v: Value) -> Result<Self, Self::Error> {
		match v {
			Value::Number(n) => Ok(n),
			_ => Err(v),
		}
	}
}

impl TryFrom<Value> for () {
	type Error = Value;

	fn try_from(v: Value) -> Result<Self, Self::Error> {
		match v {
			Value::Nil => Ok(()),
			_ => Err(v),
		}
	}
}

impl TryFrom<Value> for bool {
	type Error = Value;

	fn try_from(v: Value) -> Result<Self, Self::Error> {
		match v {
			Value::Bool(b) => Ok(b),
			_ => Err(v),
		}
	}
}

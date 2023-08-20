use std::fmt::{Display, Formatter};
use std::ptr;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum OpCode {
	Constant = 0,
	Add,
	Subtract,
	Multiply,
	Divide,
	Negate,
	Return,
}

impl Display for OpCode {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		match self {
			OpCode::Constant => write!(f, "OP_CONSTANT"),
			OpCode::Add => write!(f, "OP_ADD"),
			OpCode::Subtract => write!(f, "OP_SUBTRACT"),
			OpCode::Multiply => write!(f, "OP_MULTIPLY"),
			OpCode::Divide => write!(f, "OP_DIVIDE"),
			OpCode::Negate => write!(f, "OP_NEGATE"),
			OpCode::Return => write!(f, "OP_RETURN"),
		}
	}
}

#[derive(Debug)]
pub struct UnknownOpCode(u8);

impl Display for UnknownOpCode {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "Unknown opcode {}", self.0)
	}
}

impl std::error::Error for UnknownOpCode {}

impl TryFrom<u8> for OpCode {
	type Error = UnknownOpCode;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		if value > OpCode::Return as u8 {
			Err(UnknownOpCode(value))
		} else {
			unsafe { Ok(std::mem::transmute::<u8, OpCode>(value)) }
		}
	}
}

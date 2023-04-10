use crate::memory;
use std::fmt::{Display, Formatter};
use std::ptr;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum OpCode {
	Return = 1,
}

impl Display for OpCode {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		match self {
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
		match value {
			1 => Ok(Self::Return),
			_ => Err(UnknownOpCode(value)),
		}
	}
}

pub struct Chunk {
	pub count: usize,
	pub capacity: usize,
	pub code: *mut u8,
}

impl Default for Chunk {
	fn default() -> Self {
		Chunk {
			count: 0,
			capacity: 0,
			code: ptr::null_mut(),
		}
	}
}

impl Chunk {
	pub fn init(chunk: *mut Chunk) {
		unsafe {
			(*chunk) = Chunk::default();
		}
	}

	pub fn write(chunk: *mut Chunk, byte: OpCode) {
		unsafe {
			if (*chunk).capacity < (*chunk).count + 1 {
				let old_capacity = (*chunk).capacity;
				(*chunk).capacity = memory::grow_capacity(old_capacity);
				(*chunk).code =
					memory::grow_array::<u8>((*chunk).code, old_capacity, (*chunk).capacity);
			}

			*((*chunk).code.add((*chunk).count)) = byte as u8;
			(*chunk).count += 1;
		}
	}

	pub fn free(chunk: *mut Chunk) {
		unsafe {
			memory::free_array::<u8>((*chunk).code, (*chunk).capacity);
			Chunk::init(chunk)
		}
	}
}

use crate::memory;
use crate::value::{Value, ValueArray};
use std::fmt::{Display, Formatter};
use std::ptr;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum OpCode {
	Constant = 0,
	Return = 1,
}

impl Display for OpCode {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		match self {
			OpCode::Constant => write!(f, "OP_CONSTANT"),
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

pub struct Chunk {
	pub count: usize,
	pub capacity: usize,
	pub code: *mut u8,

	pub lines: *mut u32,
	pub constants: ValueArray,
}

impl Default for Chunk {
	fn default() -> Self {
		Chunk {
			count: 0,
			capacity: 0,
			code: ptr::null_mut(),

			lines: ptr::null_mut(),
			constants: ValueArray::default(),
		}
	}
}

impl Chunk {
	pub fn init(chunk: *mut Chunk) {
		unsafe {
			(*chunk) = Chunk::default();
		}
	}

	pub fn write(chunk: *mut Chunk, byte: u8, line: u32) {
		unsafe {
			if (*chunk).capacity < (*chunk).count + 1 {
				let old_capacity = (*chunk).capacity;
				(*chunk).capacity = memory::grow_capacity(old_capacity);

				(*chunk).code =
					memory::grow_array::<u8>((*chunk).code, old_capacity, (*chunk).capacity);

				(*chunk).lines =
					memory::grow_array::<u32>((*chunk).lines, old_capacity, (*chunk).capacity);
			}

			*((*chunk).code.add((*chunk).count)) = byte;
			*((*chunk).lines.add((*chunk).count)) = line;
			(*chunk).count += 1;
		}
	}

	pub fn add_constant(chunk: *mut Chunk, value: Value) -> u8 {
		unsafe {
			ValueArray::write(&mut (*chunk).constants, value);
			((*chunk).constants.count - 1).try_into().unwrap()
		}
	}

	pub fn free(chunk: *mut Chunk) {
		unsafe {
			memory::free_array::<u8>((*chunk).code, (*chunk).capacity);
			memory::free_array::<u32>((*chunk).lines, (*chunk).capacity);
			ValueArray::free(&mut (*chunk).constants);
			Chunk::init(chunk)
		}
	}
}

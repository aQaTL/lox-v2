use thiserror::Error;

use crate::chunk::InstructionKind;
use crate::value::Value;
use crate::{Chunk, OpCode};

#[derive(Debug, Error)]
pub enum InterpretError {
	#[error("Compile error")]
	Compile,

	#[error("Runtime error")]
	Runtime,

	#[error(transparent)]
	UnknownOpCode(#[from] crate::chunk::UnknownOpCode),
}

pub struct Vm<'a> {
	pub debug: bool,

	chunk: &'a mut Chunk,

	stack: Vec<Value>,
}

impl<'a> Vm<'a> {
	pub fn new(chunk: &'a mut Chunk) -> Self {
		Vm {
			debug: false,
			chunk,
			stack: Vec::with_capacity(256),
		}
	}

	pub fn set_chunk(&'a mut self, chunk: &'a mut Chunk) {
		self.chunk = chunk;
	}

	pub fn interpret(&mut self) -> Result<(), InterpretError> {
		self.run()
	}

	pub fn run(&mut self) -> Result<(), InterpretError> {
		let chunk_iter = self.chunk.iter().with_offset();

		for instruction in chunk_iter {
			let (instruction, offset) = instruction?;

			if self.debug {
				println!("{:?}", self.stack);
				let mut s = String::new();
				self.chunk
					.disassemble_instruction_to_write(offset, &instruction, &mut s)
					.unwrap();
				println!("{s}");
			}

			match (instruction.opcode, instruction.kind) {
				(OpCode::Return, _) => {
					println!("{:?}", self.stack.pop());
					return Ok(());
				}
				(OpCode::Add, _) => {
					let value_a = self.stack.pop().ok_or(InterpretError::Runtime)?;
					let value_b = self.stack.pop().ok_or(InterpretError::Runtime)?;
					self.stack.push(value_a + value_b);
				}
				(OpCode::Subtract, _) => {
					let value_a = self.stack.pop().ok_or(InterpretError::Runtime)?;
					let value_b = self.stack.pop().ok_or(InterpretError::Runtime)?;
					self.stack.push(value_a - value_b);
				}
				(OpCode::Multiply, _) => {
					let value_a = self.stack.pop().ok_or(InterpretError::Runtime)?;
					let value_b = self.stack.pop().ok_or(InterpretError::Runtime)?;
					self.stack.push(value_a * value_b);
				}
				(OpCode::Divide, _) => {
					let value_a = self.stack.pop().ok_or(InterpretError::Runtime)?;
					let value_b = self.stack.pop().ok_or(InterpretError::Runtime)?;
					self.stack.push(value_a / value_b);
				}
				(OpCode::Negate, _) => {
					let value = self.stack.pop().ok_or(InterpretError::Runtime)?;
					self.stack.push(-value);
				}
				(_, InstructionKind::Constant { v, idx: _idx }) => {
					self.stack.push(v);
				}
				_ => unimplemented!(),
			}
		}

		Ok(())
	}
}

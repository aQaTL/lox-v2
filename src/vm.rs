use thiserror::Error;

use crate::chunk::InstructionKind;
use crate::value::Value;
use crate::{compiler, Chunk, OpCode};

#[derive(Debug, Error)]
pub enum InterpretError {
	#[error("Compile: {0}")]
	Compile(#[from] compiler::Error),

	#[error("Runtime error")]
	Runtime,

	#[error(transparent)]
	UnknownOpCode(#[from] crate::chunk::UnknownOpCode),
}

#[derive(Default)]
pub struct Vm {
	pub debug: bool,

	stack: Vec<Value>,
}

impl Vm {
	pub fn interpret(&mut self, source: &str) -> Result<(), InterpretError> {
		let mut chunk = Chunk::default();
		compiler::compile(source, &mut chunk, self.debug)?;
		self.run(&mut chunk)?;
		Ok(())
	}

	pub fn run(&mut self, chunk: &mut Chunk) -> Result<(), InterpretError> {
		let chunk_iter = chunk.iter().with_offset();

		for instruction in chunk_iter {
			let (instruction, offset) = instruction?;

			if self.debug {
				println!("{:?}", self.stack);
				let mut s = String::new();
				chunk
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

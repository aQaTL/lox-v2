use std::fmt::{Debug, Display, Formatter};

use thiserror::Error;

use crate::value::Value;

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

impl From<OpCode> for u8 {
	fn from(v: OpCode) -> Self {
		v as u8
	}
}

impl Display for OpCode {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		match self {
			OpCode::Constant => f.pad("OP_CONSTANT"),
			OpCode::Add => f.pad("OP_ADD"),
			OpCode::Subtract => f.pad("OP_SUBTRACT"),
			OpCode::Multiply => f.pad("OP_MULTIPLY"),
			OpCode::Divide => f.pad("OP_DIVIDE"),
			OpCode::Negate => f.pad("OP_NEGATE"),
			OpCode::Return => f.pad("OP_RETURN"),
		}
	}
}

#[derive(Debug, Error)]
#[error("Unknown opcode {0}")]
pub struct UnknownOpCode(u8);

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

#[derive(Debug, Default)]
pub struct Chunk {
	code: Vec<u8>,
	constants: Vec<Value>,
	lines: Vec<usize>,
}

impl Chunk {
	pub fn write(&mut self, v: impl Into<u8>, line: usize) {
		self.code.push(v.into());
		self.lines.resize_with(self.code.len(), Default::default);
		self.lines.insert(self.code.len() - 1, line);
	}

	pub fn write_constant(&mut self, v: Value) -> usize {
		self.constants.push(v);
		self.constants.len() - 1
	}

	pub fn disassemble(&self, name: &'static str) -> String {
		let mut out = String::new();
		self.disassemble_chunk_to_writer(name, &mut out).unwrap();
		out
	}

	pub fn disassemble_chunk_to_writer<W>(&self, name: &'static str, w: &mut W) -> std::fmt::Result
	where
		W: std::fmt::Write,
	{
		write!(w, "== {name} ==")?;
		let mut iter = self.iter();
		loop {
			let offset = iter.offset;

			match iter.next() {
				Some(Ok(instruction)) => {
					self.disassemble_instruction_to_write(offset, &instruction, w)?
				}
				Some(Err(err)) => write!(w, "\n{offset:04} {err}")?,
				None => break,
			}
		}

		Ok(())
	}

	pub fn disassemble_instruction_to_write<W>(
		&self,
		offset: usize,
		instruction: &Instruction,
		w: &mut W,
	) -> std::fmt::Result
	where
		W: std::fmt::Write,
	{
		let line = self
			.lines
			.get(offset)
			.cloned()
			.expect("code and lines arrays out of sync");

		let same_line = offset
			.checked_sub(1)
			.and_then(|offset| self.lines.get(offset))
			.map(|previous_line| line == *previous_line)
			.unwrap_or_default();

		write!(w, "{offset:04} ")?;

		match same_line {
			true => write!(w, "   | ")?,
			false => write!(w, "{:>4} ", line)?,
		}

		write!(w, "{instruction}")?;

		Ok(())
	}

	pub fn decode_instruction(&self, offset: usize) -> Option<Result<Instruction, UnknownOpCode>> {
		let instruction = self.code.get(offset)?;

		let opcode = match OpCode::try_from(*instruction) {
			Ok(v) => v,
			Err(err) => return Some(Err(err)),
		};

		match opcode {
			OpCode::Return => Some(Ok(Instruction::simple(opcode))),

			OpCode::Constant => {
				let constant_idx = *self.code.get(offset + 1)? as usize;
				let constant = *self.constants.get(constant_idx)?;
				Some(Ok(Instruction::constant(opcode, constant, constant_idx)))
			}

			_ => unimplemented!(),
		}
	}

	pub fn iter(&self) -> ChunkIter<'_> {
		ChunkIter::new(self)
	}
}

pub struct ChunkIter<'a> {
	chunk: &'a Chunk,
	offset: usize,
}

impl<'a> ChunkIter<'a> {
	pub fn new(chunk: &'a Chunk) -> Self {
		ChunkIter { chunk, offset: 0 }
	}

	pub fn with_offset(self) -> ChunkWithOffsetIter<'a> {
		ChunkWithOffsetIter { chunk_iter: self }
	}
}

impl<'a> Iterator for ChunkIter<'a> {
	type Item = Result<Instruction, UnknownOpCode>;

	fn next(&mut self) -> Option<Self::Item> {
		let instruction = self.chunk.decode_instruction(self.offset);
		if let Some(Ok(ref instruction)) = instruction {
			self.offset += instruction.byte_len();
		}
		instruction
	}
}

pub struct ChunkWithOffsetIter<'a> {
	chunk_iter: ChunkIter<'a>,
}

impl<'a> Iterator for ChunkWithOffsetIter<'a> {
	type Item = Result<(Instruction, usize), UnknownOpCode>;

	fn next(&mut self) -> Option<Self::Item> {
		let offset = self.chunk_iter.offset;
		match self.chunk_iter.next() {
			Some(Ok(instruction)) => Some(Ok((instruction, offset))),
			Some(Err(err)) => Some(Err(err)),
			None => None,
		}
	}
}

#[derive(Clone, Debug)]
pub struct Instruction {
	pub kind: InstructionKind,
	pub opcode: OpCode,
}

impl Instruction {
	pub fn simple(opcode: OpCode) -> Self {
		Instruction {
			kind: InstructionKind::Simple,
			opcode,
		}
	}

	pub fn constant(opcode: OpCode, v: Value, idx: usize) -> Self {
		Instruction {
			kind: InstructionKind::Constant { v, idx },
			opcode,
		}
	}

	pub fn byte_len(&self) -> usize {
		self.kind.size()
	}
}

impl Display for Instruction {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{:<16} ", self.opcode)?;
		match self.kind {
			InstructionKind::Simple => (),
			InstructionKind::Constant { v, idx } => write!(f, "{idx:>4} '{v}'")?,
		}
		Ok(())
	}
}

#[derive(Debug, Clone, Copy)]
pub enum InstructionKind {
	Simple,
	Constant { v: Value, idx: usize },
}

impl InstructionKind {
	const fn size(&self) -> usize {
		match self {
			Self::Simple => 1,
			Self::Constant { .. } => 2,
		}
	}
}

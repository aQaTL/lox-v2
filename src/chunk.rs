use crate::value::Value;
use std::fmt::{Debug, Display, Formatter};

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

#[derive(Debug, Default)]
pub struct Chunk {
	code: Vec<u8>,
	constants: Vec<Value>,
	lines: Vec<usize>,
}

impl Chunk {
	pub fn write(&mut self, v: impl Into<u8>, line: usize) {
		self.code.push(v.into());
		self.lines.push(line);
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
				Some(Ok(instruction)) => write!(w, "\n{offset:04} {instruction}")?,
				Some(Err(err)) => write!(w, "\n{offset:04} {err}")?,
				None => break,
			}
		}

		Ok(())
	}

	pub fn disassemble_instruction(
		&self,
		offset: usize,
	) -> Option<Result<Instruction, UnknownOpCode>> {
		let instruction = self.code.get(offset)?;
		let line = *self.lines.get(offset)?;
		let same_line = offset
			.checked_sub(1)
			.and_then(|offset| self.lines.get(offset))
			.map(|previous_line| line == *previous_line)
			.unwrap_or_default();

		let opcode = match OpCode::try_from(*instruction) {
			Ok(v) => v,
			Err(err) => return Some(Err(err)),
		};
		match opcode {
			OpCode::Return => Some(Ok(Instruction::simple(opcode, line, same_line))),
			OpCode::Constant => {
				let constant_idx = *self.code.get(offset + 1)? as usize;
				let constant = *self.constants.get(constant_idx)?;
				Some(Ok(Instruction::constant(
					opcode,
					line,
					same_line,
					constant,
					constant_idx,
				)))
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
}

impl<'a> Iterator for ChunkIter<'a> {
	type Item = Result<Instruction, UnknownOpCode>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.chunk.disassemble_instruction(self.offset) {
			Some(Ok(instruction)) => {
				self.offset += instruction.byte_len();
				Some(Ok(instruction))
			}
			err => err,
		}
	}
}

#[derive(Clone, Debug)]
pub struct Instruction {
	kind: InstructionKind,
	opcode: OpCode,
	line: usize,
	same_line: bool,
}

impl Instruction {
	pub fn simple(opcode: OpCode, line: usize, same_line: bool) -> Self {
		Instruction {
			kind: InstructionKind::Simple,
			opcode,
			line,
			same_line,
		}
	}

	pub fn constant(opcode: OpCode, line: usize, same_line: bool, v: Value, idx: usize) -> Self {
		Instruction {
			kind: InstructionKind::Constant { v, idx },
			opcode,
			line,
			same_line,
		}
	}

	pub fn byte_len(&self) -> usize {
		self.kind.size()
	}
}

impl Display for Instruction {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		match self.same_line {
			true => write!(f, "   | ")?,
			false => write!(f, "{:>4} ", self.line)?,
		}
		write!(f, "{:<16} ", self.opcode)?;
		match self.kind {
			InstructionKind::Simple => (),
			InstructionKind::Constant { v, idx } => write!(f, "{idx:>4} '{v}'")?,
		}
		Ok(())
	}
}

#[derive(Debug, Clone, Copy)]
enum InstructionKind {
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

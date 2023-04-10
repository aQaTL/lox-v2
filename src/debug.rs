use crate::value::print_value;
use crate::{Chunk, OpCode};

pub fn disassemble_chunk(chunk: *mut Chunk, name: &str) {
	unsafe {
		println!("== {name} ==");
		let mut offset = 0;
		while offset < (*chunk).count {
			offset = disassemble_instruction(chunk, offset);
		}
	}
}

fn disassemble_instruction(chunk: *mut Chunk, offset: usize) -> usize {
	unsafe {
		print!("{offset:04} ");

		if offset > 0 && *(*chunk).lines.add(offset) == *(*chunk).lines.add(offset - 1) {
			print!("   | ");
		} else {
			print!("{:4} ", *(*chunk).lines.add(offset));
		}

		let instruction: u8 = *(*chunk).code.add(offset);
		match instruction.try_into() {
			Ok(opcode @ OpCode::Constant) => constant_instruction(opcode, chunk, offset),
			Ok(opcode @ OpCode::Return) => simple_instruction(opcode, offset),
			Err(err) => {
				println!("{err}");
				offset + 1
			}
		}
	}
}

fn simple_instruction(op_code: OpCode, offset: usize) -> usize {
	println!("{op_code}");
	offset + 1
}

fn constant_instruction(op_code: OpCode, chunk: *mut Chunk, offset: usize) -> usize {
	unsafe {
		let constant = *(*chunk).code.add(offset + 1);
		print!("{op_code:<16} {constant:>4} '");
		print_value(*(*chunk).constants.values.add(constant.into()));
		println!("'");
		offset + 2
	}
}

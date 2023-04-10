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

		let instruction: u8 = *(*chunk).code.add(offset);
		match instruction.try_into() {
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

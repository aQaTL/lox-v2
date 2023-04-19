use crate::chunk::{Chunk, OpCode};
use crate::debug::disassemble_chunk;
use crate::vm::VM;

mod chunk;
mod debug;
mod memory;
mod value;
mod vm;

fn main() {
	VM::init();

	let mut chunk = Chunk::default();
	Chunk::init(&mut chunk);

	let constant = Chunk::add_constant(&mut chunk, 1.2);
	Chunk::write(&mut chunk, OpCode::Constant as u8, 143);
	Chunk::write(&mut chunk, constant, 143);

	let constant = Chunk::add_constant(&mut chunk, 3.4);
	Chunk::write(&mut chunk, OpCode::Constant as u8, 143);
	Chunk::write(&mut chunk, constant, 143);

	Chunk::write(&mut chunk, OpCode::Add as u8, 143);

	let constant = Chunk::add_constant(&mut chunk, 5.6);
	Chunk::write(&mut chunk, OpCode::Constant as u8, 143);
	Chunk::write(&mut chunk, constant, 143);

	Chunk::write(&mut chunk, OpCode::Divide as u8, 143);
	Chunk::write(&mut chunk, OpCode::Negate as u8, 143);

	Chunk::write(&mut chunk, OpCode::Return as u8, 143);

	if cfg!(feature = "debug_trace_execution") {
		disassemble_chunk(&mut chunk, "test chunk");
		println!();
	}

	if let Err(err) = VM::interpret(&mut chunk) {
		eprintln!("{err}");
	}

	VM::free();
	Chunk::free(&mut chunk);
}

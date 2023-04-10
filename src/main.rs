use crate::chunk::{Chunk, OpCode};
use crate::debug::disassemble_chunk;

mod chunk;
mod debug;
mod memory;
mod value;

fn main() {
	let mut chunk = Chunk::default();
	Chunk::init(&mut chunk);

	let constant = Chunk::add_constant(&mut chunk, 1.2);
	Chunk::write(&mut chunk, OpCode::Constant as u8, 143);
	Chunk::write(&mut chunk, constant, 143);

	Chunk::write(&mut chunk, OpCode::Return as u8, 143);

	disassemble_chunk(&mut chunk, "test chunk");
	Chunk::free(&mut chunk);
}

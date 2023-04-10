use crate::chunk::{Chunk, OpCode};
use crate::debug::disassemble_chunk;

mod chunk;
mod debug;
mod memory;

fn main() {
	let mut chunk = Chunk::default();
	Chunk::init(&mut chunk);
	Chunk::write(&mut chunk, OpCode::Return);
	disassemble_chunk(&mut chunk, "test chunk");
	Chunk::free(&mut chunk);
}

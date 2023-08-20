use crate::chunk::{Chunk, OpCode};

mod chunk;
mod value;

fn main() {
	let mut chunk = Chunk::default();

	chunk.write(OpCode::Constant, 123);
	let constant_idx = chunk.write_constant(1.2);
	chunk.write(constant_idx as u8, 123);

	chunk.write(OpCode::Return, 123);

	println!("{}", chunk.disassemble("test chunk"));
}

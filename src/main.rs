use crate::chunk::{Chunk, OpCode};
use crate::vm::Vm;
use std::io::stdin;

mod chunk;
mod compiler;
mod scanner;
mod value;
mod vm;

fn main() {
	let args: Vec<String> = std::env::args().skip(1).collect();
	let result = match args.as_slice() {
		[] => repl(),
		[filename] => run_file(filename),
		_ => {
			eprintln!("Usage:\n\tlox-v2 [path]\n");
			std::process::exit(64);
		}
	};

	if let Err(err) = result {
		eprintln!("Error: {err}");
		std::process::exit(1);
	}
}

fn repl() -> Result<(), Box<dyn std::error::Error>> {
	let mut vm = Vm::default();

	for line in stdin().lines() {
		let line = line?;
		vm.interpret(&line)?;
	}

	Ok(())
}

fn run_file(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
	let source = std::fs::read_to_string(filename)?;

	let mut vm = Vm::default();
	vm.interpret(&source)?;

	Ok(())
}

use crate::chunk::{Chunk, OpCode};
use crate::debug::disassemble_chunk;
use crate::vm::VM;
use std::fmt::{Display, Formatter};
use std::io::{stdin, stdout, Write};
use std::{env, io};

mod chunk;
mod compiler;
mod debug;
mod memory;
mod scanner;
mod value;
mod vm;

fn main() {
	VM::init();

	let args: Vec<String> = env::args().skip(1).collect();
	let result = match args.len() {
		0 => repl(),
		1 => run_file(&args[0]),
		_ => {
			eprintln!("Usage: lox-v2 [path]");
			std::process::exit(64);
		}
	};

	if let Err(err) = result {
		eprintln!("{err}");

		let exit_code = match err {
			Error::Io(_) => 74,
			Error::Interpret(vm::InterpretError::CompileError) => 65,
			Error::Interpret(vm::InterpretError::RuntimeError(_)) => 70,
		};

		std::process::exit(exit_code);
	}

	VM::free();
}

#[derive(Debug)]
enum Error {
	Io(io::Error),
	Interpret(vm::InterpretError),
}

impl From<io::Error> for Error {
	fn from(v: io::Error) -> Self {
		Error::Io(v)
	}
}

impl From<vm::InterpretError> for Error {
	fn from(v: vm::InterpretError) -> Self {
		Error::Interpret(v)
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		match self {
			Error::Io(err) => err.fmt(f),
			Error::Interpret(err) => err.fmt(f),
		}
	}
}

impl std::error::Error for Error {}

fn repl() -> Result<(), Error> {
	let mut line = String::new();
	let stdin = stdin();
	let mut stdout = stdout();

	loop {
		line.clear();
		print!("> ");
		stdout.flush()?;

		let num_bytes = stdin.read_line(&mut line)?;
		if num_bytes == 0 {
			break;
		}

		VM::interpret(&line)?;
	}
	Ok(())
}

fn run_file(filename: &str) -> Result<(), Error> {
	let source = std::fs::read_to_string(filename)?;
	VM::interpret(&source)?;
	Ok(())
}

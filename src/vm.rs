use crate::debug::disassemble_instruction;
use crate::value::Value;
use crate::{chunk, Chunk, OpCode};
use std::fmt::{Display, Formatter};

static mut vm: VM = VM {
	chunk: std::ptr::null_mut(),
	ip: std::ptr::null_mut(),
	stack: [0.0; STACK_MAX],
	stack_top: std::ptr::null_mut(),
};

const STACK_MAX: usize = 256;

pub struct VM {
	chunk: *mut Chunk,
	ip: *mut u8,
	stack: [Value; STACK_MAX],
	/// Points past the last `stack` item
	stack_top: *mut Value,
}

#[derive(Debug)]
pub enum InterpretError {
	CompileError,
	RuntimeError(RuntimeError),
}

impl Display for InterpretError {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		match self {
			Self::CompileError => write!(f, "compile error"),
			Self::RuntimeError(err) => write!(f, "{err}"),
		}
	}
}

impl std::error::Error for InterpretError {}

#[derive(Debug)]
pub enum RuntimeError {
	UnknownOpCode(chunk::UnknownOpCode),
}

impl Display for RuntimeError {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		match self {
			Self::UnknownOpCode(err) => write!(f, "{err}"),
		}
	}
}

impl std::error::Error for RuntimeError {}

impl From<chunk::UnknownOpCode> for InterpretError {
	fn from(err: chunk::UnknownOpCode) -> Self {
		InterpretError::RuntimeError(RuntimeError::UnknownOpCode(err))
	}
}

impl VM {
	pub fn init() {
		VM::reset_stack();
	}

	fn reset_stack() {
		unsafe {
			vm.stack_top = vm.stack.as_mut_ptr();
		}
	}

	pub fn free() {}

	pub fn interpret(chunk: *mut Chunk) -> Result<(), InterpretError> {
		unsafe {
			vm.chunk = chunk;
			vm.ip = (*vm.chunk).code;
			run()
		}
	}

	pub fn push(value: Value) {
		unsafe {
			*vm.stack_top = value;
			vm.stack_top = vm.stack_top.add(1);
		}
	}

	pub fn pop() -> Value {
		unsafe {
			vm.stack_top = vm.stack_top.sub(1);
			*vm.stack_top
		}
	}
}

fn read_byte() -> u8 {
	unsafe {
		let b = *vm.ip;
		vm.ip = vm.ip.add(1);
		b
	}
}

fn read_constant() -> Value {
	unsafe {
		let constant_idx = read_byte();
		*(*vm.chunk).constants.values.add(constant_idx as usize)
	}
}

fn run() -> Result<(), InterpretError> {
	unsafe {
		loop {
			if cfg!(feature = "debug_trace_execution") {
				let mut slot = vm.stack.as_mut_ptr();
				while slot < vm.stack_top {
					print!("[ {} ]", *slot);
					slot = slot.add(1);
				}
				println!();
				disassemble_instruction(vm.chunk, (vm.ip as usize) - ((*vm.chunk).code as usize));
			}

			let instruction: OpCode = read_byte().try_into()?;
			match instruction {
				OpCode::Constant => {
					let constant = read_constant();
					VM::push(constant);
				}
				OpCode::Negate => VM::push(-VM::pop()),
				OpCode::Return => {
					println!("{}", VM::pop());
					return Ok(());
				}
			}
		}
	}
}

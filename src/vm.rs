use std::io::{Stdout, Write};
use thiserror::Error;

use crate::object::ObjString;
use crate::{
	chunk::{Chunk, InstructionKind, OpCode},
	compiler,
	object::{self, Object, ObjectKind},
	table::Table,
	value::Value,
};

#[derive(Debug, Error)]
pub enum InterpretError {
	#[error("Compile: {0}")]
	Compile(#[from] compiler::Error),

	#[error("Runtime error")]
	GenericRuntime,

	#[error("[line {line}] {source}")]
	Runtime { source: RuntimeError, line: usize },

	#[error(transparent)]
	UnknownOpCode(#[from] crate::chunk::UnknownOpCode),
}

#[derive(Debug, Error)]
pub enum RuntimeError {
	#[error(transparent)]
	InvalidType(#[from] InvalidTypeError),

	#[error(transparent)]
	InvalidTypes(InvalidTypesError),

	#[error("Undefined variable '{0}'.")]
	UndefinedVariable(String),
}

#[derive(Debug, Error)]
#[error("{kind}, got {value:?}")]
pub struct InvalidTypeError {
	pub value: Value,
	pub kind: InvalidTypeErrorKind,
}

#[derive(Debug, Error)]
#[error("{kind}, got {values:?}")]
pub struct InvalidTypesError {
	pub values: Vec<Value>,
	pub kind: InvalidTypeErrorKind,
}

#[derive(Debug, Error)]
pub enum InvalidTypeErrorKind {
	#[error("Operand must be a number")]
	ExpectedNumberOperand,

	#[error("Operands must be two numbers or two strings")]
	ExpectedNumberOrStringOperand,
}

pub struct Vm<W> {
	pub debug: bool,

	stack: Vec<Value>,
	objects: object::Allocator,
	globals: Table,

	stdout: W,
}

impl Default for Vm<Stdout> {
	fn default() -> Self {
		Vm::new(std::io::stdout())
	}
}

impl<W: Write> Vm<W> {
	pub fn new(stdout: W) -> Vm<W> {
		Vm {
			debug: false,
			stack: Vec::new(),
			objects: Default::default(),
			globals: Default::default(),
			stdout,
		}
	}

	pub fn interpret(&mut self, source: &str) -> Result<Value, InterpretError> {
		let mut chunk = Chunk::default();
		compiler::compile(source, &mut chunk, self.debug, &mut self.objects)?;
		self.run(&mut chunk)
	}

	pub fn run(&mut self, chunk: &mut Chunk) -> Result<Value, InterpretError> {
		let chunk_iter = chunk.iter().with_offset();

		for instruction in chunk_iter {
			let (instruction, offset) = instruction?;

			if self.debug {
				println!("{:?}", self.stack);
				let mut s = String::new();
				chunk
					.disassemble_instruction_to_write(offset, &instruction, &mut s)
					.unwrap();
				println!("{s}");
			}

			match (instruction.opcode, instruction.kind) {
				(OpCode::Return, _) => {
					let val = self.stack.pop().unwrap_or_default();
					return Ok(val);
				}
				(OpCode::Nil, _) => {
					self.stack.push(Value::Nil);
				}
				(OpCode::False, _) => {
					self.stack.push(Value::Bool(false));
				}
				(OpCode::True, _) => {
					self.stack.push(Value::Bool(true));
				}
				(OpCode::Equal, _) => {
					let value_b = self.stack.pop().ok_or(InterpretError::GenericRuntime)?;
					let value_a = self.stack.pop().ok_or(InterpretError::GenericRuntime)?;
					self.stack.push(Value::Bool(value_a == value_b));
				}
				(OpCode::Greater, _) => {
					let value_b = self.pop_number(
						InvalidTypeErrorKind::ExpectedNumberOperand,
						chunk,
						offset,
					)?;
					let value_a = self.pop_number(
						InvalidTypeErrorKind::ExpectedNumberOperand,
						chunk,
						offset,
					)?;
					self.stack.push(Value::Bool(value_a > value_b));
				}
				(OpCode::Less, _) => {
					let value_b = self.pop_number(
						InvalidTypeErrorKind::ExpectedNumberOperand,
						chunk,
						offset,
					)?;
					let value_a = self.pop_number(
						InvalidTypeErrorKind::ExpectedNumberOperand,
						chunk,
						offset,
					)?;
					self.stack.push(Value::Bool(value_a < value_b));
				}
				(OpCode::Add, _) => {
					let value_b = self.stack.pop().ok_or(InterpretError::GenericRuntime)?;
					let value_a = self.stack.pop().ok_or(InterpretError::GenericRuntime)?;
					match (&value_a, &value_b) {
						(Value::Number(a), Value::Number(b)) => {
							self.stack.push(Value::Number(a + b))
						}
						(Value::Object(a), Value::Object(b)) => unsafe {
							let (a, b): (&Object, &Object) = (&**a, &**b);
							match (&a.kind, &b.kind) {
								(ObjectKind::String, ObjectKind::String) => {
									let str_a = a.as_obj_string().unwrap();
									let str_b = b.as_obj_string().unwrap();
									let object =
										self.objects.take_string(format!("{str_a}{str_b}"));
									self.stack.push(Value::Object(object));
								}
								_ => {
									return Err(InterpretError::Runtime {
										source: RuntimeError::InvalidTypes(InvalidTypesError {
											kind:
												InvalidTypeErrorKind::ExpectedNumberOrStringOperand,
											values: vec![value_a, value_b],
										}),
										line: *chunk.lines.get(offset).expect("fix your lines"),
									})
								}
							}
						},
						_ => {
							return Err(InterpretError::Runtime {
								source: RuntimeError::InvalidTypes(InvalidTypesError {
									kind: InvalidTypeErrorKind::ExpectedNumberOrStringOperand,
									values: vec![value_a, value_b],
								}),
								line: *chunk.lines.get(offset).expect("fix your lines"),
							})
						}
					}
				}
				(OpCode::Subtract, _) => {
					let value_b = self.pop_number(
						InvalidTypeErrorKind::ExpectedNumberOperand,
						chunk,
						offset,
					)?;
					let value_a = self.pop_number(
						InvalidTypeErrorKind::ExpectedNumberOperand,
						chunk,
						offset,
					)?;
					self.stack.push(Value::Number(value_a - value_b));
				}
				(OpCode::Multiply, _) => {
					let value_b = self.pop_number(
						InvalidTypeErrorKind::ExpectedNumberOperand,
						chunk,
						offset,
					)?;
					let value_a = self.pop_number(
						InvalidTypeErrorKind::ExpectedNumberOperand,
						chunk,
						offset,
					)?;
					self.stack.push(Value::Number(value_a * value_b));
				}
				(OpCode::Divide, _) => {
					let value_b = self.pop_number(
						InvalidTypeErrorKind::ExpectedNumberOperand,
						chunk,
						offset,
					)?;
					let value_a = self.pop_number(
						InvalidTypeErrorKind::ExpectedNumberOperand,
						chunk,
						offset,
					)?;
					self.stack.push(Value::Number(value_a / value_b));
				}
				(OpCode::Not, _) => {
					let value = self.stack.pop().ok_or(InterpretError::GenericRuntime)?;
					self.stack.push(Value::Bool(value.is_falsey()));
				}
				(OpCode::Negate, _) => {
					let value = self.pop_number(
						InvalidTypeErrorKind::ExpectedNumberOperand,
						chunk,
						offset,
					)?;
					self.stack.push(Value::Number(-value));
				}
				(OpCode::Print, _) => {
					let value = self.stack.pop().ok_or(InterpretError::GenericRuntime)?;
					self.stdout.write_fmt(format_args!("{value}")).unwrap();
				}
				(OpCode::Pop, _) => {
					self.stack.pop().ok_or(InterpretError::GenericRuntime)?;
				}
				(OpCode::Constant, InstructionKind::Constant { v, idx: _idx }) => {
					self.stack.push(v);
				}
				(OpCode::DefineGlobal, InstructionKind::Constant { v, idx: _idx }) => {
					let name = match v {
						Value::Object(obj) => obj.cast::<ObjString>(),
						_ => panic!(),
					};
					let value = self.stack.pop().unwrap();
					self.globals.set(name, value);
				}
				(OpCode::GetGlobal, InstructionKind::Constant { v, idx: _idx }) => {
					let name = match v {
						Value::Object(obj) => obj.cast::<ObjString>(),
						_ => panic!(),
					};
					let value = self.globals.get(name).ok_or(InterpretError::Runtime {
						source: RuntimeError::UndefinedVariable(unsafe { (*name).to_string() }),
						line: *chunk.lines.get(offset).expect("fix your lines"),
					})?;
					self.stack.push(value.clone());
				}
				(opcode, instruction_kind) => unimplemented!("{opcode:?}, {instruction_kind:?}"),
			}
		}

		Ok(Value::Nil)
	}

	fn pop_number(
		&mut self,
		err_kind: InvalidTypeErrorKind,
		chunk: &Chunk,
		offset: usize,
	) -> Result<f64, InterpretError> {
		let n: f64 = self
			.stack
			.pop()
			.ok_or(InterpretError::GenericRuntime)?
			.try_into()
			.map_err(|val| InterpretError::Runtime {
				source: RuntimeError::InvalidType(InvalidTypeError {
					value: val,
					kind: err_kind,
				}),
				line: *chunk.lines.get(offset).expect("fix your line vec"),
			})?;
		Ok(n)
	}
}

use std::ptr;
use thiserror::Error;

use crate::chunk::InstructionKind;
use crate::compiler::Error::ParserError;
use crate::object::{Object, ObjectKind};
use crate::value::Value;
use crate::{
	chunk::{Chunk, OpCode},
	compiler,
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

pub struct Vm {
	pub debug: bool,

	stack: Vec<Value>,
	objects: *mut Object,
}

impl Default for Vm {
	fn default() -> Self {
		Vm {
			debug: false,
			stack: Vec::new(),
			objects: ptr::null_mut(),
		}
	}
}

impl Drop for Vm {
	fn drop(&mut self) {
		// Free objects
		unsafe {
			let mut object = self.objects;
			while !object.is_null() {
				let next = (*object).next;
				drop(Box::from_raw(object));
				object = next;
			}
		}
	}
}

impl Vm {
	pub fn interpret(&mut self, source: &str) -> Result<Value, InterpretError> {
		let mut chunk = Chunk::default();
		compiler::compile(source, &mut chunk, self.debug)?;
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
					println!("{val}");
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
								(ObjectKind::String(str_a), ObjectKind::String(str_b)) => {
									let object = self.allocate_object(ObjectKind::String(format!(
										"{str_a}{str_b}"
									)));
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
				(_, InstructionKind::Constant { v, idx: _idx }) => {
					self.stack.push(v);
				}
				_ => unimplemented!(),
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

	fn allocate_object(&mut self, obj: ObjectKind) -> *mut Object {
		let object = Box::new(Object {
			kind: obj,
			next: self.objects,
		});
		let object = Box::into_raw(object);
		self.objects = object;
		object
	}
}

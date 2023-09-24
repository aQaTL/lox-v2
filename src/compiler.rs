use thiserror::Error;

use crate::chunk::{Chunk, OpCode};
use crate::object;
use crate::scanner::{self, Scanner, Token, TokenKind};
use crate::value::Value;

pub fn compile(
	source: &str,
	chunk: &mut Chunk,
	debug: bool,
	objects: &mut object::Allocator,
) -> Result<(), Error> {
	Compiler::new(source, chunk, debug, objects).compile()
}

#[derive(Debug, Error)]
pub enum Error {
	#[error(transparent)]
	Scanner(#[from] scanner::Error),

	#[error("Expected end of expression")]
	ExpectedEndOfExpr,

	#[error("Parser error")]
	ParserError,

	#[error("Too many constants in one chunk")]
	TooManyConstants,

	#[error("Expected '{token}' after {after}")]
	ExpectedToken {
		token: &'static str,
		after: &'static str,
	},

	#[error("Expected expression")]
	ExpectedExpression,
}

struct Compiler<'a, 'b, 'c> {
	scanner: Scanner<'a>,
	chunk: &'b mut Chunk,
	debug: bool,

	parser: Parser<'a>,

	parser_had_error: bool,
	parser_panic_mode: bool,

	objects: &'c mut object::Allocator,
}

struct ParseRule<'a, 'b, 'c> {
	prefix: Option<ParseFn<'a, 'b, 'c>>,
	infix: Option<ParseFn<'a, 'b, 'c>>,
	precedence: Precedence,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u32)]
enum Precedence {
	None = 0,
	Assignment,
	Or,
	And,
	Equality,
	Comparison,
	Term,
	Factor,
	Unary,
	Call,
	Primary,
}

type ParseFn<'a, 'b, 'c> = fn(&mut Compiler<'a, 'b, 'c>) -> Result<(), Error>;

impl<'a, 'b, 'c> Compiler<'a, 'b, 'c> {
	pub fn new(
		source: &'a str,
		chunk: &'b mut Chunk,
		debug: bool,
		objects: &'c mut object::Allocator,
	) -> Self {
		Compiler {
			scanner: Scanner::new(source),
			chunk,
			debug,
			parser: Parser {
				previous: None,
				current: None,
			},
			parser_had_error: false,
			parser_panic_mode: false,

			objects,
		}
	}

	pub fn compile(mut self) -> Result<(), Error> {
		self.parser_had_error = false;
		self.parser_panic_mode = false;

		self.advance()?;
		self.expression()?;
		self.consume(None, Error::ExpectedEndOfExpr)?;
		self.end_compiler();

		Ok(())
	}

	fn advance(&mut self) -> Result<(), Error> {
		self.parser.previous = self.parser.current.clone();

		loop {
			match self.scanner.scan_token() {
				Some(Ok(token)) => {
					self.parser.current = Some(token);
					break;
				}
				None => {
					self.parser.current = None;
					break;
				}
				Some(Err(err)) => {
					if !self.parser_panic_mode {
						eprintln!("{err}");
					}
					self.parser_panic_mode = true;
					self.parser_had_error = true;
				}
			};
		}

		if self.parser_had_error {
			Err(Error::ParserError)
		} else {
			Ok(())
		}
	}

	fn consume(&mut self, token_kind: Option<TokenKind>, err: Error) -> Result<(), Error> {
		if self.parser.current.as_ref().map(|t| t.kind) == token_kind {
			self.advance()?;
			Ok(())
		} else {
			Err(err)
		}
	}

	fn current_chunk(&mut self) -> &mut Chunk {
		self.chunk
	}

	fn emit_byte(&mut self, byte: u8) {
		let line = self
			.parser
			.previous
			.as_ref()
			.map(|token| token.line)
			.unwrap_or(0);
		self.current_chunk().write(byte, line);
	}

	fn emit_bytes<const N: usize>(&mut self, bytes: [u8; N]) {
		for byte in bytes {
			self.emit_byte(byte)
		}
	}

	fn emit_return(&mut self) {
		self.emit_byte(OpCode::Return as u8);
	}

	fn emit_constant(&mut self, v: Value) -> Result<(), Error> {
		let const_idx = self.make_constant(v)?;
		self.emit_bytes([OpCode::Constant as u8, const_idx]);
		Ok(())
	}

	fn make_constant(&mut self, v: Value) -> Result<u8, Error> {
		let const_idx = self.chunk.write_constant(v);
		u8::try_from(const_idx).map_err(|_| Error::TooManyConstants)
	}

	fn end_compiler(&mut self) {
		self.emit_return();
	}

	fn expression(&mut self) -> Result<(), Error> {
		self.parse_precedence(Precedence::Assignment)
	}

	fn number(&mut self) -> Result<(), Error> {
		let TokenKind::Number(num) = self.parser.previous.as_ref().unwrap().kind else {
			panic!("expected number");
		};
		let num: f64 = num.parse().unwrap();
		self.emit_constant(Value::Number(num))?;
		Ok(())
	}

	fn string(&mut self) -> Result<(), Error> {
		let TokenKind::String(str) = self.parser.previous.as_ref().unwrap().kind else {
			panic!("expected string");
		};
		let object = self.objects.copy_string(str);
		self.emit_constant(Value::Object(object))?;
		Ok(())
	}

	fn grouping(&mut self) -> Result<(), Error> {
		self.expression()?;
		self.consume(
			Some(TokenKind::RightParen),
			Error::ExpectedToken {
				token: ")",
				after: "expression",
			},
		)?;
		Ok(())
	}

	fn unary(&mut self) -> Result<(), Error> {
		let op_kind = self.parser.previous.as_ref().unwrap().kind;
		self.parse_precedence(Precedence::Unary)?;
		match op_kind {
			TokenKind::Minus => self.emit_byte(OpCode::Negate as u8),
			TokenKind::Bang => self.emit_byte(OpCode::Not as u8),
			_ => unreachable!(),
		}
		Ok(())
	}

	fn binary(&mut self) -> Result<(), Error> {
		let operator_kind = self.parser.previous.as_ref().unwrap().kind;
		let rule = self.get_rule(&operator_kind);
		self.parse_precedence(unsafe {
			std::mem::transmute::<u32, Precedence>(rule.precedence as u32 + 1)
		})?;
		match operator_kind {
			TokenKind::Plus => self.emit_byte(OpCode::Add as u8),
			TokenKind::Minus => self.emit_byte(OpCode::Subtract as u8),
			TokenKind::Star => self.emit_byte(OpCode::Multiply as u8),
			TokenKind::Slash => self.emit_byte(OpCode::Divide as u8),
			TokenKind::BangEqual => self.emit_bytes([OpCode::Equal as u8, OpCode::Not as u8]),
			TokenKind::EqualEqual => self.emit_byte(OpCode::Equal as u8),
			TokenKind::Greater => self.emit_byte(OpCode::Greater as u8),
			TokenKind::GreaterEqual => self.emit_bytes([OpCode::Less as u8, OpCode::Not as u8]),
			TokenKind::Less => self.emit_byte(OpCode::Less as u8),
			TokenKind::LessEqual => self.emit_bytes([OpCode::Greater as u8, OpCode::Not as u8]),
			_ => panic!("invalid operator: {:?}", operator_kind),
		}
		Ok(())
	}

	fn literal(&mut self) -> Result<(), Error> {
		match self.parser.previous.as_ref().unwrap().kind {
			TokenKind::Nil => self.emit_byte(OpCode::Nil as u8),
			TokenKind::False => self.emit_byte(OpCode::False as u8),
			TokenKind::True => self.emit_byte(OpCode::True as u8),
			t => panic!("Invalid token: {:?}", t),
		}
		Ok(())
	}

	fn parse_precedence(&mut self, precedence: Precedence) -> Result<(), Error> {
		self.advance()?;

		let Some(prefix_rule): Option<ParseFn> = self
			.parser
			.previous
			.as_ref()
			.map(|t| t.kind)
			.and_then(|k| self.get_rule(&k).prefix)
		else {
			return Err(Error::ExpectedExpression);
		};

		prefix_rule(self)?;

		loop {
			let Some(ref current_token) = self.parser.current else {
				break;
			};
			if precedence as u32 > self.get_rule(&current_token.kind).precedence as u32 {
				break;
			}

			self.advance()?;
			let infix_rule: ParseFn = self
				.get_rule(&self.parser.previous.as_ref().unwrap().kind)
				.infix
				.unwrap();
			infix_rule(self)?;
		}

		Ok(())
	}

	fn get_rule(&self, kind: &TokenKind<'a>) -> ParseRule<'a, 'b, 'c> {
		match kind {
			TokenKind::LeftParen => ParseRule {
				prefix: Some(Compiler::grouping),
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::RightParen => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::LeftBrace => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::RightBrace => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::Semicolon => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::Comma => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::Dot => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::Minus => ParseRule {
				prefix: Some(Compiler::unary),
				infix: Some(Compiler::binary),
				precedence: Precedence::Term,
			},
			TokenKind::Plus => ParseRule {
				prefix: None,
				infix: Some(Compiler::binary),
				precedence: Precedence::Term,
			},
			TokenKind::Slash => ParseRule {
				prefix: None,
				infix: Some(Compiler::binary),
				precedence: Precedence::Factor,
			},
			TokenKind::Star => ParseRule {
				prefix: None,
				infix: Some(Compiler::binary),
				precedence: Precedence::Factor,
			},
			TokenKind::Bang => ParseRule {
				prefix: Some(Compiler::unary),
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::BangEqual => ParseRule {
				prefix: None,
				infix: Some(Compiler::binary),
				precedence: Precedence::Equality,
			},
			TokenKind::Equal => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::EqualEqual => ParseRule {
				prefix: None,
				infix: Some(Compiler::binary),
				precedence: Precedence::Equality,
			},
			TokenKind::Greater => ParseRule {
				prefix: None,
				infix: Some(Compiler::binary),
				precedence: Precedence::Comparison,
			},
			TokenKind::GreaterEqual => ParseRule {
				prefix: None,
				infix: Some(Compiler::binary),
				precedence: Precedence::Comparison,
			},
			TokenKind::Less => ParseRule {
				prefix: None,
				infix: Some(Compiler::binary),
				precedence: Precedence::Comparison,
			},
			TokenKind::LessEqual => ParseRule {
				prefix: None,
				infix: Some(Compiler::binary),
				precedence: Precedence::Comparison,
			},
			TokenKind::Identifier(_) => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::String(_) => ParseRule {
				prefix: Some(Compiler::string),
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::Number(_) => ParseRule {
				prefix: Some(Compiler::number),
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::And => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::Class => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::Else => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::False => ParseRule {
				prefix: Some(Compiler::literal),
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::For => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::Fun => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::If => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::Nil => ParseRule {
				prefix: Some(Compiler::literal),
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::Or => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::Print => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::Return => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::Super => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::This => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::True => ParseRule {
				prefix: Some(Compiler::literal),
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::Var => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
			TokenKind::While => ParseRule {
				prefix: None,
				infix: None,
				precedence: Precedence::None,
			},
		}
	}
}

struct Parser<'a> {
	current: Option<Token<'a>>,
	previous: Option<Token<'a>>,
}

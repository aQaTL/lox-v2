use std::fmt::{Display, Formatter};

pub struct Scanner<'a> {
	source: &'a str,
	start: *const u8,
	current: *const u8,
	last: *const u8,
	line: i32,
}

impl<'a> Scanner<'a> {
	pub fn new(source: &'a str) -> Scanner {
		Scanner {
			source,
			start: source.as_ptr(),
			current: source.as_ptr(),
			last: source.as_ptr().wrapping_add(source.len()),
			line: 1,
		}
	}

	pub fn scan_token(&mut self) -> Result<Option<Token>, Error> {
		self.skip_whitespace();
		self.start = self.current;

		if self.is_at_end() {
			return Ok(None);
		}

		let c = self.advance();

		match c {
			b'(' => Ok(Some(self.make_token(TokenKind::LeftParen))),
			b')' => Ok(Some(self.make_token(TokenKind::RightParen))),
			b'{' => Ok(Some(self.make_token(TokenKind::LeftBrace))),
			b'}' => Ok(Some(self.make_token(TokenKind::RightBrace))),
			b';' => Ok(Some(self.make_token(TokenKind::Semicolon))),
			b',' => Ok(Some(self.make_token(TokenKind::Comma))),
			b'.' => Ok(Some(self.make_token(TokenKind::Dot))),
			b'-' => Ok(Some(self.make_token(TokenKind::Minus))),
			b'+' => Ok(Some(self.make_token(TokenKind::Plus))),
			b'/' => Ok(Some(self.make_token(TokenKind::Slash))),
			b'*' => Ok(Some(self.make_token(TokenKind::Star))),
			b'!' => {
				let kind = if self.matches(b'=') {
					TokenKind::BangEqual
				} else {
					TokenKind::Bang
				};
				Ok(Some(self.make_token(kind)))
			}
			b'=' => {
				let kind = if self.matches(b'=') {
					TokenKind::EqualEqual
				} else {
					TokenKind::Equal
				};
				Ok(Some(self.make_token(kind)))
			}
			b'<' => {
				let kind = if self.matches(b'=') {
					TokenKind::LessEqual
				} else {
					TokenKind::Less
				};
				Ok(Some(self.make_token(kind)))
			}
			b'>' => {
				let kind = if self.matches(b'=') {
					TokenKind::GreaterEqual
				} else {
					TokenKind::Greater
				};
				Ok(Some(self.make_token(kind)))
			}
			b'"' => self.string(),
			c if c.is_ascii_digit() => self.number(),
			c if is_alpha(c) => self.identifier(),
			_ => Err(Error {
				line: self.line,
				kind: ErrorKind::UnexpectedCharacter(c),
			}),
		}
	}

	fn skip_whitespace(&mut self) {
		loop {
			let Some(c) = self.peek() else {
				return;
			};
			match c {
				b' ' | b'\r' | b'\t' => {
					self.advance();
				}
				b'\n' => {
					self.line += 1;
					self.advance();
				}
				b'/' => {
					if let Some(b'/') = self.peek_next() {
						while self.peek().map(|c| c != b'\n').unwrap_or_default() {
							self.advance();
						}
					} else {
						return;
					}
				}
				_ => return,
			}
		}
	}

	fn string(&mut self) -> Result<Option<Token>, Error> {
		while self.peek().map(|c| c != b'"').unwrap_or_default() {
			if let Some(b'\n') = self.peek() {
				self.line += 1;
				self.advance();
			}
		}

		if self.is_at_end() {
			return Err(Error::new(self, ErrorKind::UnterminatedString));
		}

		// The closing quote.
		self.advance();
		Ok(Some(self.make_token(TokenKind::String)))
	}

	fn number(&mut self) -> Result<Option<Token>, Error> {
		while self.peek().map(|c| c.is_ascii_digit()).unwrap_or_default() {
			self.advance();
		}

		if self.peek().map(|c| c == b'.').unwrap_or_default()
			&& self
				.peek_next()
				.map(|c| c.is_ascii_digit())
				.unwrap_or_default()
		{
			// Consume the `.`.
			self.advance();

			while self.peek().map(|c| c.is_ascii_digit()).unwrap_or_default() {
				self.advance();
			}
		}

		Ok(Some(self.make_token(TokenKind::Number)))
	}

	fn identifier(&mut self) -> Result<Option<Token>, Error> {
		loop {
			let Some(c) = self.peek() else {
				break;
			};
			if !is_alpha(c) && !c.is_ascii_digit() {
				break;
			}
			self.advance();
		}
		Ok(Some(self.make_token(self.identifier_kind())))
	}

	fn identifier_kind(&self) -> TokenKind {
		match unsafe { *self.start } {
			b'a' => self.check_keyword(1, "nd", TokenKind::And),
			b'c' => self.check_keyword(1, "lass", TokenKind::Class),
			b'e' => self.check_keyword(1, "lse", TokenKind::Else),
			b'f' if self.current as usize - self.start as usize > 1 => {
				match unsafe { *self.start.wrapping_add(1) } {
					b'a' => self.check_keyword(2, "lse", TokenKind::False),
					b'o' => self.check_keyword(2, "r", TokenKind::For),
					b'u' => self.check_keyword(2, "n", TokenKind::Fun),
					_ => TokenKind::Identifier,
				}
			}
			b'i' => self.check_keyword(1, "f", TokenKind::If),
			b'n' => self.check_keyword(1, "il", TokenKind::Nil),
			b'o' => self.check_keyword(1, "r", TokenKind::Or),
			b'p' => self.check_keyword(1, "rint", TokenKind::Print),
			b'r' => self.check_keyword(1, "eturn", TokenKind::Return),
			b's' => self.check_keyword(1, "uper", TokenKind::Super),
			b't' if self.current as usize - self.start as usize > 1 => {
				match unsafe { *self.start.wrapping_add(1) } {
					b'h' => self.check_keyword(2, "is", TokenKind::This),
					b'r' => self.check_keyword(2, "ue", TokenKind::True),
					_ => TokenKind::Identifier,
				}
			}
			b'v' => self.check_keyword(1, "ar", TokenKind::Var),
			b'w' => self.check_keyword(1, "hile", TokenKind::While),
			_ => TokenKind::Identifier,
		}
	}

	fn check_keyword(&self, check_idx: usize, rest: &str, kind: TokenKind) -> TokenKind {
		if self.current as usize - self.start as usize == rest.len() + check_idx {
			let current_rest = unsafe {
				std::slice::from_raw_parts(self.start.wrapping_add(check_idx), rest.len())
			};
			if current_rest == rest.as_bytes() {
				return kind;
			}
		}
		TokenKind::Identifier
	}

	fn is_at_end(&self) -> bool {
		self.current == self.last
	}

	fn make_token(&'a self, kind: TokenKind) -> Token<'a> {
		let lexeme: &'a str = unsafe {
			std::str::from_utf8_unchecked(std::slice::from_raw_parts(
				self.start,
				self.current as usize - self.start as usize,
			))
		};
		Token {
			kind,
			lexeme,
			line: self.line,
		}
	}

	fn advance(&mut self) -> u8 {
		self.current = self.current.wrapping_add(1);
		unsafe { *self.current.wrapping_sub(1) }
	}

	fn matches(&mut self, expected: u8) -> bool {
		if self.is_at_end() {
			return false;
		}

		if unsafe { *self.current } != expected {
			return false;
		}

		self.current = self.current.wrapping_add(1);
		true
	}

	fn peek(&self) -> Option<u8> {
		if self.is_at_end() {
			None
		} else {
			Some(unsafe { *self.current })
		}
	}

	fn peek_next(&self) -> Option<u8> {
		if self.is_at_end() {
			return None;
		}
		Some(unsafe { *self.current.wrapping_add(1) })
	}
}

#[derive(Debug)]
pub struct Token<'a> {
	pub kind: TokenKind,
	pub lexeme: &'a str,
	pub line: i32,
}

#[derive(Debug, Eq, PartialEq)]
pub enum TokenKind {
	// Single-character tokens.
	LeftParen,
	RightParen,
	LeftBrace,
	RightBrace,
	Comma,
	Dot,
	Minus,
	Plus,
	Semicolon,
	Slash,
	Star,

	// One or two character tokens.
	Bang,
	BangEqual,
	Equal,
	EqualEqual,
	Greater,
	GreaterEqual,
	Less,
	LessEqual,

	// Literals.
	Identifier,
	String,
	Number,

	// Keywords.
	And,
	Class,
	Else,
	False,
	Fun,
	For,
	If,
	Nil,
	Or,
	Print,
	Return,
	Super,
	This,
	True,
	Var,
	While,

	Eof,
}

#[derive(Debug)]
pub struct Error {
	line: i32,
	kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
	UnexpectedCharacter(u8),
	UnterminatedString,
}

impl Error {
	fn new(scanner: &Scanner, kind: ErrorKind) -> Self {
		Error {
			line: scanner.line,
			kind,
		}
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "[line {}] ", self.line)?;
		match self.kind {
			ErrorKind::UnexpectedCharacter(c) => write!(f, "unexpected character `{}`", c as char)?,
			ErrorKind::UnterminatedString => write!(f, "unterminated string")?,
		}
		Ok(())
	}
}

impl std::error::Error for Error {}

fn is_alpha(c: u8) -> bool {
	matches!(c, b'a'..=b'z' | b'A'..=b'Z' | b'_')
}

#[cfg(test)]
mod tests {
	use crate::scanner::{Scanner, TokenKind};

	#[test]
	fn scan_keyword() {
		let mut scanner = Scanner::new("while");
		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::While, "got: {token:#?}");
		assert_eq!(token.lexeme, "while", "got: {token:#?}");

		let mut scanner = Scanner::new("fun");
		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::Fun, "got: {token:#?}");
		assert_eq!(token.lexeme, "fun", "got: {token:#?}");

		let mut scanner = Scanner::new("super");
		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::Super, "got: {token:#?}");
		assert_eq!(token.lexeme, "super", "got: {token:#?}");

		let mut scanner = Scanner::new("for");
		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::For, "got: {token:#?}");
		assert_eq!(token.lexeme, "for", "got: {token:#?}");

		let mut scanner = Scanner::new("false");
		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::False, "got: {token:#?}");
		assert_eq!(token.lexeme, "false", "got: {token:#?}");

		let mut scanner = Scanner::new("true");
		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::True, "got: {token:#?}");
		assert_eq!(token.lexeme, "true", "got: {token:#?}");

		let mut scanner = Scanner::new("this");
		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::This, "got: {token:#?}");
		assert_eq!(token.lexeme, "this", "got: {token:#?}");

		let mut scanner = Scanner::new("print");
		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::Print, "got: {token:#?}");
		assert_eq!(token.lexeme, "print", "got: {token:#?}");
	}

	#[test]
	fn scan_identifier() {
		let mut scanner = Scanner::new("alamakota");
		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::Identifier, "got: {token:#?}");
		assert_eq!(token.lexeme, "alamakota", "got: {token:#?}");

		let mut scanner = Scanner::new("f");
		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::Identifier, "got: {token:#?}");
		assert_eq!(token.lexeme, "f", "got: {token:#?}");

		let mut scanner = Scanner::new("fori");
		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::Identifier, "got: {token:#?}");
		assert_eq!(token.lexeme, "fori", "got: {token:#?}");

		let mut scanner = Scanner::new("thiss");
		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::Identifier, "got: {token:#?}");
		assert_eq!(token.lexeme, "thiss", "got: {token:#?}");

		let mut scanner = Scanner::new("i");
		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::Identifier, "got: {token:#?}");
		assert_eq!(token.lexeme, "i", "got: {token:#?}");

		let mut scanner = Scanner::new("Print");
		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::Identifier, "got: {token:#?}");
		assert_eq!(token.lexeme, "Print", "got: {token:#?}");
	}

	#[test]
	fn scan_single_character_token() {
		let source = "({.;,>}!)";
		let mut scanner = Scanner::new(source);

		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::LeftParen);
		assert_eq!(token.lexeme, "(");

		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::LeftBrace);
		assert_eq!(token.lexeme, "{");

		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::Dot);
		assert_eq!(token.lexeme, ".");

		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::Semicolon);
		assert_eq!(token.lexeme, ";");

		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::Comma);
		assert_eq!(token.lexeme, ",");

		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::Greater);
		assert_eq!(token.lexeme, ">");

		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::RightBrace);
		assert_eq!(token.lexeme, "}");

		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::Bang);
		assert_eq!(token.lexeme, "!");

		let token = scanner.scan_token().unwrap().unwrap();
		assert_eq!(token.kind, TokenKind::RightParen);
		assert_eq!(token.lexeme, ")");
	}
}

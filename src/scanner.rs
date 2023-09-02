use thiserror::Error;

#[derive(Debug, Copy, Clone)]
pub enum TokenKind<'a> {
	// Single-character
	LeftParen,
	RightParen,
	LeftBrace,
	RightBrace,
	Semicolon,
	Comma,
	Dot,
	Minus,
	Plus,
	Slash,
	Star,

	// One or two character
	Bang,
	BangEqual,
	Equal,
	EqualEqual,
	Greater,
	GreaterEqual,
	Less,
	LessEqual,

	// Literals
	Identifier(&'a str),
	String(&'a str),
	Number(&'a str),

	// Keywords
	And,
	Class,
	Else,
	False,
	For,
	Fun,
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
}

#[derive(Debug)]
pub struct Token<'a> {
	pub kind: TokenKind<'a>,
	pub line: usize,
}

#[derive(Debug, Error)]
#[error("[line {line}] {err}")]
pub struct Error {
	err: ErrorKind,
	line: usize,
}

#[derive(Debug, Error)]
pub enum ErrorKind {
	#[error("Unexpected character: {0}")]
	UnexpectedCharacter(String),
	#[error("Unterminated string")]
	UnterminatedString,
}

pub struct Scanner<'a> {
	source: &'a str,
	start: usize,
	current: usize,
	pub line: usize,
}

impl<'a> Scanner<'a> {
	pub fn new(source: &'a str) -> Self {
		Self {
			source,
			start: 0,
			current: 0,
			line: 1,
		}
	}

	pub fn scan_token(&mut self) -> Option<Result<Token<'a>, Error>> {
		self.skip_whitespace();

		self.start = self.current;

		let c = self.advance()?;
		match c {
			b'(' => Some(Ok(self.make_token(TokenKind::LeftParen))),
			b')' => Some(Ok(self.make_token(TokenKind::RightParen))),
			b'{' => Some(Ok(self.make_token(TokenKind::LeftBrace))),
			b'}' => Some(Ok(self.make_token(TokenKind::RightBrace))),
			b';' => Some(Ok(self.make_token(TokenKind::Semicolon))),
			b',' => Some(Ok(self.make_token(TokenKind::Comma))),
			b'.' => Some(Ok(self.make_token(TokenKind::Dot))),
			b'-' => Some(Ok(self.make_token(TokenKind::Minus))),
			b'+' => Some(Ok(self.make_token(TokenKind::Plus))),
			b'/' => Some(Ok(self.make_token(TokenKind::Slash))),
			b'*' => Some(Ok(self.make_token(TokenKind::Star))),

			b'!' => {
				let kind = if self.matches(b'=') {
					TokenKind::BangEqual
				} else {
					TokenKind::Bang
				};
				Some(Ok(self.make_token(kind)))
			}

			b'=' => {
				let kind = if self.matches(b'=') {
					TokenKind::EqualEqual
				} else {
					TokenKind::Equal
				};
				Some(Ok(self.make_token(kind)))
			}

			b'<' => {
				let kind = if self.matches(b'=') {
					TokenKind::LessEqual
				} else {
					TokenKind::Less
				};
				Some(Ok(self.make_token(kind)))
			}

			b'>' => {
				let kind = if self.matches(b'=') {
					TokenKind::GreaterEqual
				} else {
					TokenKind::Greater
				};
				Some(Ok(self.make_token(kind)))
			}

			b'"' => Some(self.string().map(|k| self.make_token(k))),

			c if c.is_ascii_digit() => Some(self.number().map(|k| self.make_token(k))),

			c if is_alpha(c) => Some(self.identifier().map(|k| self.make_token(k))),

			_ => Some(Err(
				self.make_error(ErrorKind::UnexpectedCharacter(c.to_string()))
			)),
		}
	}

	fn make_token(&self, kind: TokenKind<'a>) -> Token<'a> {
		Token {
			kind,
			line: self.line,
		}
	}

	fn advance(&mut self) -> Option<u8> {
		self.current += 1;
		self.source.as_bytes().get(self.current - 1).copied()
	}

	fn matches(&mut self, expected: u8) -> bool {
		if let Some(current) = self.source.as_bytes().get(self.current) {
			if *current == expected {
				self.current += 1;
				return true;
			}
		}
		false
	}

	fn peek(&self) -> Option<u8> {
		self.source.as_bytes().get(self.current).copied()
	}

	fn peek_next(&self) -> Option<u8> {
		self.source.as_bytes().get(self.current + 1).copied()
	}

	fn string(&mut self) -> Result<TokenKind<'a>, Error> {
		loop {
			match self.peek() {
				Some(b'"') => {
					self.advance();
					break;
				}
				Some(c) => {
					if c == b'\n' {
						self.line += 1;
					}
					self.advance();
				}
				None => return Err(self.make_error(ErrorKind::UnterminatedString)),
			}
		}

		let str = &self.source[(self.start + 1)..self.current];
		Ok(TokenKind::String(str))
	}

	fn number(&mut self) -> Result<TokenKind<'a>, Error> {
		while self.peek().map(|x| x.is_ascii_digit()).unwrap_or_default() {
			self.advance();
		}

		if matches!(self.peek(), Some(b'.'))
			&& matches!(self.peek_next(), Some(c) if c.is_ascii_digit())
		{
			self.advance();

			while self.peek().map(|x| x.is_ascii_digit()).unwrap_or_default() {
				self.advance();
			}
		}

		let num = &self.source[self.start..=self.current];
		Ok(TokenKind::Number(num))
	}

	fn identifier(&mut self) -> Result<TokenKind<'a>, Error> {
		while self
			.peek()
			.map(|c| is_alpha(c) || c.is_ascii_digit())
			.unwrap_or_default()
		{
			self.advance();
		}
		Ok(self.identifier_kind())
	}

	fn identifier_kind(&self) -> TokenKind<'a> {
		let ident = &self.source[self.start..=self.current];
		let rest = &ident[1..];
		match ident.as_bytes()[0] {
			b'a' if rest == "nd" => TokenKind::And,
			b'c' if rest == "lass" => TokenKind::Class,
			b'e' if rest == "lse" => TokenKind::Else,
			b'i' if rest == "f" => TokenKind::If,
			b'n' if rest == "il" => TokenKind::Nil,
			b'o' if rest == "r" => TokenKind::Or,
			b'p' if rest == "rint" => TokenKind::Print,
			b'r' if rest == "eturn" => TokenKind::Return,
			b's' if rest == "uper" => TokenKind::Super,
			b'v' if rest == "ar" => TokenKind::Var,
			b'w' if rest == "hile" => TokenKind::While,
			b'f' if ident.len() > 1 => match ident.as_bytes()[1] {
				b'a' if &ident[2..] == "lse" => TokenKind::False,
				b'o' if &ident[2..] == "r" => TokenKind::For,
				b'u' if &ident[2..] == "n" => TokenKind::Fun,
				_ => TokenKind::Identifier(ident),
			},
			b't' if ident.len() > 1 => match ident.as_bytes()[1] {
				b'h' if &ident[2..] == "is" => TokenKind::This,
				b'r' if &ident[2..] == "rue" => TokenKind::True,
				_ => TokenKind::Identifier(ident),
			},
			_ => {
				let ident = &self.source[self.start..self.current];
				TokenKind::Identifier(ident)
			}
		}
	}

	fn skip_whitespace(&mut self) {
		while let Some(c) = self.peek() {
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
						while let Some(c_2) = self.peek() {
							if c_2 == b'\n' {
								break;
							}
							self.advance();
						}
					}
				}
				_ => break,
			}
		}
	}

	#[track_caller]
	fn make_error(&self, err: ErrorKind) -> Error {
		Error {
			err,
			line: self.line,
		}
	}
}

fn is_alpha(c: u8) -> bool {
	matches!(c, b'a'..=b'z' | b'A'..=b'Z' | b'_')
}

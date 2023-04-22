use crate::scanner::{Scanner, TokenKind};

pub fn compile(source: &str) {
	let mut scanner = Scanner::new(source);

	let mut line = -1;

	loop {
		let token = scanner.scan_token().unwrap().unwrap();
		if token.line != line {
			line = token.line;
			print!("{line:4} ");
		} else {
			print!("   | ");
		}
		println!("{:12?} '{}'", token.kind, token.lexeme);

		if matches!(token.kind, TokenKind::Eof) {
			break;
		}
	}
}

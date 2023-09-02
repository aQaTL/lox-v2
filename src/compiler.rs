use crate::scanner::{self, Scanner};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
	#[error(transparent)]
	Scanner(#[from] scanner::Error),
}

pub fn compile(source: &str, debug: bool) -> Result<(), Error> {
	let mut scanner = Scanner::new(source);

	let mut line = 0;

	while let Some(token) = scanner.scan_token().transpose()? {
		if token.line != line {
			if debug {
				print!("{:>4} ", token.line);
			}
			line = token.line;
		} else if debug {
			print!("   | ");
		}
		if debug {
			println!("{:?}", token);
		}
	}

	Ok(())
}

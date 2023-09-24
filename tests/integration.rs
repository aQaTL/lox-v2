use lox_v2::vm::Vm;

fn run_and_capture_stdout(source: &str) -> String {
	let mut stdout = Vec::new();
	let mut vm = Vm::new(&mut stdout);
	vm.interpret(source).unwrap();
	String::from_utf8(stdout).unwrap()
}

#[test]
fn add() {
	let result = run_and_capture_stdout("print 9+5;");
	let expected = "14";
	assert_eq!(result, expected);

	let result = run_and_capture_stdout("print -9+5;");
	let expected = "-4";
	assert_eq!(result, expected);

	let result = run_and_capture_stdout("print 9+-5;");
	let expected = "4";
	assert_eq!(result, expected);
}

#[test]
fn divide() {
	let result = run_and_capture_stdout("print 8/2.5;");
	let expected = "3.2";
	assert_eq!(result, expected);

	let result = run_and_capture_stdout("print 150.5/8;");
	let expected = "18.8125";
	assert_eq!(result, expected);

	let result = run_and_capture_stdout("print 14/4;");
	let expected = "3.5";
	assert_eq!(result, expected);
}

#[test]
fn literals() {
	let stdout = run_and_capture_stdout("print true;");
	assert_eq!(stdout, "true");

	let stdout = run_and_capture_stdout("print false;");
	assert_eq!(stdout, "false");

	let stdout = run_and_capture_stdout("print nil;");
	assert_eq!(stdout, "nil");
}

#[test]
fn falsey_comparisons() {
	let stdout = run_and_capture_stdout("print !(5 - 4 > 3 * 2 == !nil);");
	assert_eq!(stdout, "true");
}

#[test]
fn print_statement() {
	let stdout = run_and_capture_stdout(
		r#"
	var bevarage = "cafe au lait";
	var breakfast = "beignets with " + bevarage;
	print breakfast;
	"#,
	);
	assert_eq!(stdout, "beignets with cafe au lait");
}

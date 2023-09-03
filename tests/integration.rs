use lox_v2::vm::{InterpretError, Vm};

#[test]
fn add() {
	let mut vm = Vm::default();

	let result: f64 = vm.interpret("9+5").unwrap().try_into().unwrap();
	let expected = 14.;
	assert_eq!(result, expected);

	let result: f64 = vm.interpret("-9+5").unwrap().try_into().unwrap();
	let expected = -4.;
	assert_eq!(result, expected);

	let result: f64 = vm.interpret("9+-5").unwrap().try_into().unwrap();
	let expected = 4.;
	assert_eq!(result, expected);
}

#[test]
fn divide() {
	let mut vm = Vm::default();

	let result: f64 = vm.interpret("8/2.5").unwrap().try_into().unwrap();
	let expected = 3.2;
	assert_eq!(result, expected);

	let result: f64 = vm.interpret("150.5/8").unwrap().try_into().unwrap();
	let expected = 18.8125;
	assert_eq!(result, expected);

	let result: f64 = vm.interpret("14/4").unwrap().try_into().unwrap();
	let expected = 3.5;
	assert_eq!(result, expected);
}

#[test]
fn literals() {
	let mut vm = Vm::default();

	let result: bool = vm.interpret("true").unwrap().try_into().unwrap();
	assert!(result);

	let result: bool = vm.interpret("false").unwrap().try_into().unwrap();
	assert!(!result);

	let _: () = vm.interpret("nil").unwrap().try_into().unwrap();
}

#[test]
fn compiler_expects_expression() {
	let mut vm = Vm::default();

	let result = vm.interpret("\n").unwrap_err();
	assert!(matches!(
		result,
		InterpretError::Compile(lox_v2::compiler::Error::ExpectedExpression)
	));

	let result = vm.interpret("").unwrap_err();
	assert!(matches!(
		result,
		InterpretError::Compile(lox_v2::compiler::Error::ExpectedExpression)
	));
}

#[test]
fn falsey_comparisons() {
	let mut vm = Vm::default();

	let result: bool = vm
		.interpret("!(5 - 4 > 3 * 2 == !nil)")
		.unwrap()
		.try_into()
		.unwrap();
	assert!(result);
}

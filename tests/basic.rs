#[test]
fn sanity() {
    assert!(true);
}

#[test]
fn test_lexer_simple() {
    let src = "fn add a b => a+b";
    let tokens = c_dsl::lexer::lex(src);
    // Expect 'fn' as Symbol, 'add' as Ident, etc.
    assert!(tokens
        .iter()
        .any(|t| matches!(t, c_dsl::lexer::Token::Ident(s) if s == "add")));
    assert!(tokens.iter().any(|t| matches!(t, c_dsl::lexer::Token::Fn)));
}

#[test]
fn test_builtin_eval_number() {
    use c_dsl::builtins::builtin_eval;
    use c_dsl::interpreter::{Environment, Value};
    let env = Environment::new();
    let result = builtin_eval(&env, vec![Value::String("42".to_string())]);
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_builtin_eval_nil_on_empty() {
    use c_dsl::builtins::builtin_eval;
    use c_dsl::interpreter::{Environment, Value};
    let env = Environment::new();
    let result = builtin_eval(&env, vec![]);
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_builtin_eval_arithmetic() {
    use c_dsl::builtins::builtin_eval;
    use c_dsl::interpreter::{Environment, Value};
    let env = Environment::new();
    let result = builtin_eval(&env, vec![Value::String("2 + 3".to_string())]);
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_builtin_eval_non_string_returns_nil() {
    use c_dsl::builtins::builtin_eval;
    use c_dsl::interpreter::{Environment, Value};
    let env = Environment::new();
    let result = builtin_eval(&env, vec![Value::Number(99.0)]);
    assert_eq!(result, Value::Nil);
}

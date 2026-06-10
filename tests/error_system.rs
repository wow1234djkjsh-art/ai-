use c_dsl::lexer::{lex, Token};
use c_dsl::parser::{parse_src, Expr};
use c_dsl::interpreter::{execute, Value};

#[test]
fn lexer_emits_dot_token() {
    let tokens = lex("err.message");
    assert!(
        tokens.iter().any(|t| matches!(t, Token::Dot)),
        "expected Token::Dot in {:?}",
        tokens
    );
}

#[test]
fn parser_field_access_simple() {
    let ast = parse_src("err.message").unwrap();
    match ast {
        Expr::Block(stmts) => match &stmts[0] {
            Expr::FieldAccess { object, field } => {
                assert_eq!(field, "message");
                assert!(matches!(object.as_ref(), Expr::Ident(n) if n == "err"));
            }
            other => panic!("expected FieldAccess, got {:?}", other),
        },
        other => panic!("expected Block, got {:?}", other),
    }
}

#[test]
fn parser_field_access_chained_with_index() {
    // list[0].name  →  FieldAccess(Index(Ident("list"), Number(0)), "name")
    let ast = parse_src("list[0].name").unwrap();
    match ast {
        Expr::Block(stmts) => match &stmts[0] {
            Expr::FieldAccess { object, field } => {
                assert_eq!(field, "name");
                assert!(matches!(object.as_ref(), Expr::Index { .. }));
            }
            other => panic!("expected FieldAccess, got {:?}", other),
        },
        other => panic!("expected Block, got {:?}", other),
    }
}

#[test]
fn field_access_error_message() {
    let result = execute("e = unknown_fn()\ne.message");
    match result {
        Value::String(s) => assert!(s.contains("unknown function"), "got: {}", s),
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn field_access_error_type() {
    let result = execute("e = unknown_fn()\ne.type");
    assert_eq!(result, Value::String("error".into()));
}

#[test]
fn field_access_dict_field() {
    let result = execute("d = {name: \"alice\"}\nd.name");
    assert_eq!(result, Value::String("alice".into()));
}

#[test]
fn field_access_dict_missing_key_is_nil() {
    let result = execute("d = {name: \"alice\"}\nd.age");
    assert_eq!(result, Value::Nil);
}

#[test]
fn field_access_on_non_dict_non_error_is_error() {
    let result = execute("x = 42\nx.foo");
    assert!(matches!(result, Value::Error(_)), "expected error, got {:?}", result);
}

#[test]
fn undefined_variable_is_error() {
    let result = execute("foo");
    assert!(matches!(result, Value::Error(ref msg) if msg.contains("undefined variable")),
        "expected undefined variable error, got {:?}", result);
}

#[test]
fn type_error_string_plus_number() {
    let result = execute("\"hello\" + 1");
    assert!(matches!(result, Value::Error(ref msg) if msg.contains("type error")),
        "expected type error, got {:?}", result);
}

#[test]
fn division_by_zero_is_error() {
    let result = execute("10 / 0");
    assert!(matches!(result, Value::Error(ref msg) if msg.contains("division by zero")),
        "expected division by zero error, got {:?}", result);
}

#[test]
fn neg_on_string_is_error() {
    let result = execute("-\"hello\"");
    assert!(matches!(result, Value::Error(ref msg) if msg.contains("type error")),
        "expected type error on neg, got {:?}", result);
}

#[test]
fn index_out_of_bounds_is_error() {
    let result = execute("x = [1, 2, 3]\nx[10]");
    assert!(matches!(result, Value::Error(ref msg) if msg.contains("out of bounds")),
        "expected out of bounds error, got {:?}", result);
}

#[test]
fn negative_list_index_is_error() {
    let result = execute("x = [1, 2]\nx[-1]");
    assert!(matches!(result, Value::Error(ref msg) if msg.contains("invalid index")),
        "expected invalid index error, got {:?}", result);
}

#[test]
fn index_wrong_type_is_error() {
    let result = execute("x = [1, 2]\nx[\"a\"]");
    assert!(matches!(result, Value::Error(ref msg) if msg.contains("type error")),
        "expected type error on list index, got {:?}", result);
}

#[test]
fn each_with_non_function_is_error() {
    let result = execute("each 1, 2, 3: 42");
    assert!(matches!(result, Value::Error(ref msg) if msg.contains("function")),
        "expected function error from each, got {:?}", result);
}

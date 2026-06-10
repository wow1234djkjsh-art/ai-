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

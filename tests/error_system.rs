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

#[test]
fn parse_error_returns_value_error() {
    // "fn fn" is invalid syntax — should return Error, not Nil
    let result = execute("fn fn");
    assert!(matches!(result, Value::Error(ref msg) if msg.contains("parse error")),
        "expected parse error, got {:?}", result);
}

// Integration tests for error-as-first-class-value

#[test]
fn error_value_inspectable_in_language() {
    // Call an undefined function, capture the error, inspect its type field
    let result = execute(
        "e = bad_fn()\n\
         e.type"
    );
    assert_eq!(result, Value::String("error".into()),
        "e.type should be 'error', got {:?}", result);
}

#[test]
fn error_propagates_through_block_standalone() {
    // Standalone error expression (not an assignment) propagates
    let result = execute(
        "x = 1\n\
         undefined_var\n\
         z = 999"
    );
    assert!(matches!(result, Value::Error(_)),
        "expected error to propagate, got {:?}", result);
}

#[test]
fn error_message_field_is_string() {
    let result = execute("e = unknown_fn()\ne.message");
    assert!(matches!(result, Value::String(_)),
        "e.message should be a String, got {:?}", result);
}

#[test]
fn dict_field_access_still_works() {
    let result = execute(
        "user = {name: \"bob\", age: 30}\n\
         user.name"
    );
    assert_eq!(result, Value::String("bob".into()));
}

#[test]
fn lexer_emits_try_catch_end_tokens() {
    let tokens = lex("try\ncatch err\nend");
    assert!(tokens.iter().any(|t| matches!(t, Token::Try)), "missing Try");
    assert!(tokens.iter().any(|t| matches!(t, Token::Catch)), "missing Catch");
    assert!(tokens.iter().any(|t| matches!(t, Token::End)), "missing End");
}

#[test]
fn parser_try_catch_basic() {
    use c_dsl::parser::{parse_src, Expr};
    let ast = parse_src("try\nbad_fn()\ncatch err\nprint(err)\nend").unwrap();
    match ast {
        Expr::Block(stmts) => match &stmts[0] {
            Expr::TryCatch { catch_var, .. } => {
                assert_eq!(catch_var, "err");
            }
            other => panic!("expected TryCatch, got {:?}", other),
        },
        other => panic!("expected Block, got {:?}", other),
    }
}

#[test]
fn try_catch_catches_standalone_error() {
    let result = execute(
        "try\n\
         unknown_fn()\n\
         catch err\n\
         err.message\n\
         end"
    );
    match result {
        Value::String(s) => assert!(s.contains("unknown function"), "got: {}", s),
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn try_catch_no_error_runs_body() {
    let result = execute(
        "try\n\
         42\n\
         catch err\n\
         0\n\
         end"
    );
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn try_catch_binds_error_to_var() {
    let result = execute(
        "try\n\
         unknown_fn()\n\
         catch e\n\
         e.type\n\
         end"
    );
    assert_eq!(result, Value::String("error".into()));
}

#[test]
fn try_catch_nested() {
    let result = execute(
        "try\n\
         try\n\
         unknown_fn()\n\
         catch inner\n\
         inner.message\n\
         end\n\
         catch outer\n\
         \"outer caught\"\n\
         end"
    );
    // inner catch fires; outer never fires
    match result {
        Value::String(s) => assert!(s.contains("unknown function"), "got: {}", s),
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn try_catch_handler_error_propagates() {
    // If the catch handler itself throws, the error propagates out
    let result = execute(
        "try\nunknown_fn()\ncatch e\nanother_unknown()\nend"
    );
    assert!(matches!(result, Value::Error(_)),
        "expected error from handler to propagate, got {:?}", result);
}

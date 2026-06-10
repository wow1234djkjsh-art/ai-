use c_dsl::lexer::{lex, Token};
use c_dsl::parser::{parse_src, Expr};

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

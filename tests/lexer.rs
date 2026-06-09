use c_dsl::lexer::{lex, Token};

#[test]
fn test_lex_assign_number() {
    assert_eq!(lex("x=42"), vec![
        Token::Ident("x".into()), Token::Sym('='), Token::Number(42.0), Token::Eof,
    ]);
}

#[test]
fn test_lex_string() {
    assert_eq!(lex("\"hello\""), vec![Token::Str("hello".into()), Token::Eof]);
}

#[test]
fn test_lex_fn_def() {
    assert_eq!(lex("fn add a,b=>a+b"), vec![
        Token::Fn,
        Token::Ident("add".into()),
        Token::Ident("a".into()),
        Token::Sym(','),
        Token::Ident("b".into()),
        Token::Arrow,
        Token::Ident("a".into()),
        Token::Sym('+'),
        Token::Ident("b".into()),
        Token::Eof,
    ]);
}

#[test]
fn test_lex_no_whitespace_stmts() {
    assert_eq!(lex("x=3;y=4"), vec![
        Token::Ident("x".into()), Token::Sym('='), Token::Number(3.0), Token::Sep,
        Token::Ident("y".into()), Token::Sym('='), Token::Number(4.0), Token::Eof,
    ]);
}

#[test]
fn test_lex_pipe_and_conditional() {
    assert_eq!(lex("?x>0:x:0|print"), vec![
        Token::Sym('?'),
        Token::Ident("x".into()), Token::Sym('>'), Token::Number(0.0),
        Token::Sym(':'), Token::Ident("x".into()),
        Token::Sym(':'), Token::Number(0.0),
        Token::Sym('|'), Token::Ident("print".into()),
        Token::Eof,
    ]);
}

#[test]
fn test_lex_each() {
    assert_eq!(lex("each 1,2:fn x=>x"), vec![
        Token::Each,
        Token::Number(1.0), Token::Sym(','), Token::Number(2.0),
        Token::Sym(':'),
        Token::Fn,
        Token::Ident("x".into()),
        Token::Arrow,
        Token::Ident("x".into()),
        Token::Eof,
    ]);
}

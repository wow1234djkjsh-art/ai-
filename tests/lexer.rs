use c_dsl::lexer::{lex, Token};

#[test]
fn test_lex_assign_number() {
    assert_eq!(
        lex("x=42"),
        vec![
            Token::Ident("x".into()),
            Token::Sym('='),
            Token::Number(42.0),
            Token::Eof,
        ]
    );
}

#[test]
fn test_lex_string() {
    assert_eq!(
        lex("\"hello\""),
        vec![Token::Str("hello".into()), Token::Eof]
    );
}

#[test]
fn test_lex_fn_def() {
    assert_eq!(
        lex("fn add a,b=>a+b"),
        vec![
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
        ]
    );
}

#[test]
fn test_lex_no_whitespace_stmts() {
    assert_eq!(
        lex("x=3;y=4"),
        vec![
            Token::Ident("x".into()),
            Token::Sym('='),
            Token::Number(3.0),
            Token::Sep,
            Token::Ident("y".into()),
            Token::Sym('='),
            Token::Number(4.0),
            Token::Eof,
        ]
    );
}

#[test]
fn test_lex_pipe_and_conditional() {
    assert_eq!(
        lex("?x>0:x:0|print"),
        vec![
            Token::Sym('?'),
            Token::Ident("x".into()),
            Token::Sym('>'),
            Token::Number(0.0),
            Token::Sym(':'),
            Token::Ident("x".into()),
            Token::Sym(':'),
            Token::Number(0.0),
            Token::Sym('|'),
            Token::Ident("print".into()),
            Token::Eof,
        ]
    );
}

#[test]
fn test_lex_each() {
    assert_eq!(
        lex("each 1,2:fn x=>x"),
        vec![
            Token::Each,
            Token::Number(1.0),
            Token::Sym(','),
            Token::Number(2.0),
            Token::Sym(':'),
            Token::Fn,
            Token::Ident("x".into()),
            Token::Arrow,
            Token::Ident("x".into()),
            Token::Eof,
        ]
    );
}

#[test]
fn test_lex_brackets() {
    let tokens = lex("[1,2]");
    assert!(tokens.iter().any(|t| t == &Token::Sym('[')));
    assert!(tokens.iter().any(|t| t == &Token::Sym(']')));
}

#[test]
fn test_lex_braces() {
    let tokens = lex("{a:1}");
    assert!(tokens.iter().any(|t| t == &Token::Sym('{')));
    assert!(tokens.iter().any(|t| t == &Token::Sym('}')));
}

#[test]
fn test_lex_logical_keywords() {
    let tokens = lex("a and b or not c");
    assert!(tokens.iter().any(|t| t == &Token::And));
    assert!(tokens.iter().any(|t| t == &Token::Or));
    assert!(tokens.iter().any(|t| t == &Token::Not));
}

#[test]
fn test_lex_line_continuation() {
    // backslash + newline must not emit a Sep token
    let tokens = lex("1\\\n2");
    let seps: Vec<_> = tokens.iter().filter(|t| **t == Token::Sep).collect();
    assert!(seps.is_empty(), "backslash continuation must suppress the newline Sep");
}

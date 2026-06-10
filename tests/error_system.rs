use c_dsl::lexer::{lex, Token};

#[test]
fn lexer_emits_dot_token() {
    let tokens = lex("err.message");
    assert!(
        tokens.iter().any(|t| matches!(t, Token::Dot)),
        "expected Token::Dot in {:?}",
        tokens
    );
}

#![allow(dead_code)]
// Basic lexer for Compact‑DSL

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Number(f64),
    String(String),
    Symbol(String),
    Newline,
    Eof,
}

/// Very simple lexer: split on whitespace, map known keywords to Symbol, otherwise Ident.
/// This is a placeholder; a full lexer would handle numbers, strings, etc.
pub fn lex(src: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    for part in src.split_whitespace() {
        match part {
            "fn" | "if" | "then" | "do" | "end" => tokens.push(Token::Symbol(part.to_string())),
            _ => tokens.push(Token::Ident(part.to_string())),
        }
    }
    tokens.push(Token::Eof);
    tokens
}

#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Str(String),
    Ident(String),
    Fn,
    Each,
    And,
    Or,
    Not,
    Arrow,
    Sep,
    Sym(char),
    Eof,
}

pub fn lex(src: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' => {
                i += 1;
            }
            '\n' => {
                tokens.push(Token::Sep);
                i += 1;
            }
            ';' => {
                tokens.push(Token::Sep);
                i += 1;
            }
            '=' => {
                if i + 1 < chars.len() && chars[i + 1] == '>' {
                    tokens.push(Token::Arrow);
                    i += 2;
                } else {
                    tokens.push(Token::Sym('='));
                    i += 1;
                }
            }
            '+' | '-' | '*' | '/' | '>' | '<' | '?' | ':' | '|' | ',' | '(' | ')'
            | '[' | ']' | '{' | '}' => {
                tokens.push(Token::Sym(chars[i]));
                i += 1;
            }
            '"' => {
                i += 1;
                let start = i;
                while i < chars.len() && chars[i] != '"' {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                tokens.push(Token::Str(s));
                if i < chars.len() {
                    i += 1;
                }
            }
            c if c.is_ascii_digit() => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                tokens.push(Token::Number(s.parse().unwrap_or(0.0)));
            }
            c if c.is_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                match word.as_str() {
                    "fn" => tokens.push(Token::Fn),
                    "each" => tokens.push(Token::Each),
                    "and" => tokens.push(Token::And),
                    "or" => tokens.push(Token::Or),
                    "not" => tokens.push(Token::Not),
                    _ => tokens.push(Token::Ident(word)),
                }
            }
            '\\' if i + 1 < chars.len() && chars[i + 1] == '\n' => {
                i += 2; // backslash + newline → skip both, no Sep emitted
            }
            '\\' => {
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    tokens.push(Token::Eof);
    tokens
}

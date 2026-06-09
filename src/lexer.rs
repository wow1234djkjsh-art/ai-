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

/// Lex source into tokens.  Also returns a parallel `Vec<bool>` where `spaces[i]` is
/// `true` when token `i` was preceded by at least one space or tab (not newline/sep).
/// This lets the parser distinguish `lst[0]` (no space → subscript) from
/// `first [7,8,9]` (space → space-call argument).
pub fn lex(src: &str) -> Vec<Token> {
    lex_with_spaces(src).0
}

pub fn lex_with_spaces(src: &str) -> (Vec<Token>, Vec<bool>) {
    let mut tokens = Vec::new();
    let mut spaces: Vec<bool> = Vec::new();
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut had_space = false;
    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' => {
                had_space = true;
                i += 1;
            }
            '\n' => {
                had_space = false;
                tokens.push(Token::Sep);
                spaces.push(false);
                i += 1;
            }
            ';' => {
                had_space = false;
                tokens.push(Token::Sep);
                spaces.push(false);
                i += 1;
            }
            '=' => {
                if i + 1 < chars.len() && chars[i + 1] == '>' {
                    tokens.push(Token::Arrow);
                    spaces.push(had_space);
                    had_space = false;
                    i += 2;
                } else {
                    tokens.push(Token::Sym('='));
                    spaces.push(had_space);
                    had_space = false;
                    i += 1;
                }
            }
            '+' | '-' | '*' | '/' | '>' | '<' | '?' | ':' | '|' | ',' | '(' | ')'
            | '[' | ']' | '{' | '}' => {
                tokens.push(Token::Sym(chars[i]));
                spaces.push(had_space);
                had_space = false;
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
                spaces.push(had_space);
                had_space = false;
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
                spaces.push(had_space);
                had_space = false;
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
                spaces.push(had_space);
                had_space = false;
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
    spaces.push(false);
    (tokens, spaces)
}

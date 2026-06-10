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
    Dot,
    Try,
    Catch,
    End,
    Eq,   // ==
    Neq,  // !=
    Gte,  // >=
    Lte,  // <=
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
                } else if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Eq);
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
            '!' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Neq);
                    spaces.push(had_space);
                    had_space = false;
                    i += 2;
                } else {
                    i += 1; // lone '!' ignored
                }
            }
            '>' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Gte);
                    spaces.push(had_space);
                    had_space = false;
                    i += 2;
                } else {
                    tokens.push(Token::Sym('>'));
                    spaces.push(had_space);
                    had_space = false;
                    i += 1;
                }
            }
            '<' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Lte);
                    spaces.push(had_space);
                    had_space = false;
                    i += 2;
                } else {
                    tokens.push(Token::Sym('<'));
                    spaces.push(had_space);
                    had_space = false;
                    i += 1;
                }
            }
            '+' | '-' | '*' | '/' | '?' | ':' | '|' | ',' | '(' | ')'
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
                    "try" => tokens.push(Token::Try),
                    "catch" => tokens.push(Token::Catch),
                    "end" => tokens.push(Token::End),
                    _ => tokens.push(Token::Ident(word)),
                }
                spaces.push(had_space);
                had_space = false;
            }
            '\\' if i + 1 < chars.len() && chars[i + 1] == '\n' => {
                i += 2; // backslash + newline → skip both, no Sep emitted
            }
            '.' => {
                tokens.push(Token::Dot);
                spaces.push(had_space);
                had_space = false;
                i += 1;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_eq() {
        assert_eq!(lex("=="), vec![Token::Eq, Token::Eof]);
    }
    #[test]
    fn lex_neq() {
        assert_eq!(lex("!="), vec![Token::Neq, Token::Eof]);
    }
    #[test]
    fn lex_gte() {
        assert_eq!(lex(">="), vec![Token::Gte, Token::Eof]);
    }
    #[test]
    fn lex_lte() {
        assert_eq!(lex("<="), vec![Token::Lte, Token::Eof]);
    }
    #[test]
    fn lex_gt_alone() {
        assert_eq!(lex(">"), vec![Token::Sym('>'), Token::Eof]);
    }
    #[test]
    fn lex_lt_alone() {
        assert_eq!(lex("<"), vec![Token::Sym('<'), Token::Eof]);
    }
    #[test]
    fn lex_assign_unchanged() {
        assert_eq!(
            lex("x = 1"),
            vec![Token::Ident("x".into()), Token::Sym('='), Token::Number(1.0), Token::Eof]
        );
    }
    #[test]
    fn lex_arrow_unchanged() {
        assert_eq!(lex("=>"), vec![Token::Arrow, Token::Eof]);
    }
}

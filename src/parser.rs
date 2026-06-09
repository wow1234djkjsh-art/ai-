#![allow(dead_code)]
// Simple parser for Compact‑DSL

use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    FnDef {
        name: String,
        params: Vec<String>,
        body: String,
    },
    // Future expression types can be added here
}

/// Parse a sequence of tokens into an AST.
/// Currently only supports simple function definitions of the form:
/// `fn <name> <param1> <param2> ... => <body>`
pub fn parse(tokens: &[Token]) -> Result<Expr, String> {
    let mut iter = tokens.iter().peekable();
    // Expect leading "fn" symbol
    match iter.next() {
        Some(Token::Symbol(s)) if s == "fn" => {}
        _ => return Err("expected 'fn' keyword".into()),
    }
    // Function name
    let name = match iter.next() {
        Some(Token::Ident(id)) => id.clone(),
        _ => return Err("expected function name identifier".into()),
    };
    // Collect parameters until we see the "=>" symbol
    let mut params = Vec::new();
    while let Some(tok) = iter.peek() {
        match tok {
            Token::Symbol(sym) if sym == "=>" => {
                // consume the symbol and break
                iter.next();
                break;
            }
            Token::Ident(id) => {
                params.push(id.clone());
                iter.next();
            }
            _ => return Err("unexpected token while parsing parameters".into()),
        }
    }
    // The rest of the tokens constitute the body (joined with spaces)
    let mut body_parts = Vec::new();
    for tok in iter {
        match tok {
            Token::Ident(s) | Token::Symbol(s) => body_parts.push(s.clone()),
            Token::Number(n) => body_parts.push(n.to_string()),
            Token::String(s) => body_parts.push(s.clone()),
            Token::Newline => body_parts.push("\n".to_string()),
            Token::Eof => break,
        }
    }
    let body = body_parts.join(" ");
    Ok(Expr::FnDef { name, params, body })
}

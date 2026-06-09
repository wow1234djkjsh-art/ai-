use crate::lexer::{lex_with_spaces, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Str(String),
    Ident(String),
    Neg(Box<Expr>),
    Assign {
        name: String,
        value: Box<Expr>,
    },
    BinOp {
        op: char,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    FnDef {
        name: String,
        params: Vec<String>,
        body: Box<Expr>,
    },
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        else_: Box<Expr>,
    },
    Pipe {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Each {
        items: Vec<Expr>,
        func: Box<Expr>,
    },
    Block(Vec<Expr>),
    List(Vec<Expr>),
    Dict(Vec<(String, Expr)>),
    Index { object: Box<Expr>, index: Box<Expr> },
}

struct Parser<'a> {
    tokens: &'a [Token],
    spaces: &'a [bool],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token], spaces: &'a [bool]) -> Self {
        Parser { tokens, spaces, pos: 0 }
    }

    /// Returns true if the current (next-to-be-consumed) token was preceded by whitespace.
    fn peek_has_space(&self) -> bool {
        self.spaces.get(self.pos).copied().unwrap_or(false)
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn eat_sym(&mut self, c: char) -> Result<(), String> {
        if self.peek() == &Token::Sym(c) {
            self.advance();
            Ok(())
        } else {
            Err(format!("expected '{}', got {:?}", c, self.peek()))
        }
    }

    fn eat_arrow(&mut self) -> Result<(), String> {
        if self.peek() == &Token::Arrow {
            self.advance();
            Ok(())
        } else {
            Err(format!("expected '=>', got {:?}", self.peek()))
        }
    }

    fn skip_seps(&mut self) {
        while self.peek() == &Token::Sep {
            self.advance();
        }
    }

    fn parse_block(&mut self) -> Result<Expr, String> {
        self.skip_seps();
        let mut stmts = Vec::new();
        while !matches!(self.peek(), Token::Eof) {
            stmts.push(self.parse_stmt()?);
            if matches!(self.peek(), Token::Sep) {
                while matches!(self.peek(), Token::Sep) {
                    self.advance();
                }
            } else {
                break;
            }
        }
        Ok(Expr::Block(stmts))
    }

    fn parse_stmt(&mut self) -> Result<Expr, String> {
        match self.peek().clone() {
            Token::Fn => self.parse_fn_def(),
            Token::Each => self.parse_each(),
            Token::Sym('?') => self.parse_if(),
            Token::Ident(name) => {
                if self.tokens.get(self.pos + 1) == Some(&Token::Sym('=')) {
                    self.advance(); // Ident
                    self.advance(); // '='
                    let value = self.parse_expr()?;
                    Ok(Expr::Assign {
                        name,
                        value: Box::new(value),
                    })
                } else {
                    self.parse_expr()
                }
            }
            _ => self.parse_expr(),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        if self.peek() == &Token::Sym('?') {
            self.parse_if()
        } else {
            self.parse_pipe()
        }
    }

    fn parse_fn_def(&mut self) -> Result<Expr, String> {
        self.advance(); // Fn
        let name = match self.advance() {
            Token::Ident(n) => n,
            tok => return Err(format!("expected fn name, got {:?}", tok)),
        };
        let params = self.parse_params()?;
        self.eat_arrow()?;
        let body = self.parse_expr()?;
        Ok(Expr::FnDef {
            name,
            params,
            body: Box::new(body),
        })
    }

    fn parse_lambda(&mut self) -> Result<Expr, String> {
        self.advance(); // Fn
        let params = self.parse_params()?;
        self.eat_arrow()?;
        let body = self.parse_expr()?;
        Ok(Expr::Lambda {
            params,
            body: Box::new(body),
        })
    }

    fn parse_params(&mut self) -> Result<Vec<String>, String> {
        let mut params = Vec::new();
        while let Token::Ident(p) = self.peek().clone() {
            params.push(p);
            self.advance();
            if self.peek() == &Token::Sym(',') {
                self.advance();
            }
        }
        Ok(params)
    }

    fn parse_each(&mut self) -> Result<Expr, String> {
        self.advance(); // Each
        let mut items = Vec::new();
        loop {
            if matches!(self.peek(), Token::Sym(':') | Token::Eof) {
                break;
            }
            items.push(self.parse_add()?);
            if self.peek() == &Token::Sym(',') {
                self.advance();
            } else {
                break;
            }
        }
        self.eat_sym(':')?;
        let func = if self.peek() == &Token::Fn {
            self.parse_lambda()?
        } else {
            self.parse_pipe()?
        };
        Ok(Expr::Each {
            items,
            func: Box::new(func),
        })
    }

    fn parse_if(&mut self) -> Result<Expr, String> {
        self.advance(); // '?'
        let cond = self.parse_cmp()?;
        self.eat_sym(':')?;
        let then = self.parse_expr()?;
        self.eat_sym(':')?;
        let else_ = self.parse_expr()?;
        Ok(Expr::If {
            cond: Box::new(cond),
            then: Box::new(then),
            else_: Box::new(else_),
        })
    }

    fn parse_pipe(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_cmp()?;
        while self.peek() == &Token::Sym('|') {
            self.advance();
            let right = self.parse_cmp()?;
            left = Expr::Pipe {
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_cmp(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_add()?;
        while let Token::Sym(op) = self.peek().clone() {
            if op != '>' && op != '<' {
                break;
            }
            self.advance();
            let right = self.parse_add()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_add(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_mul()?;
        while let Token::Sym(op) = self.peek().clone() {
            if op != '+' && op != '-' {
                break;
            }
            self.advance();
            let right = self.parse_mul()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        while let Token::Sym(op) = self.peek().clone() {
            if op != '*' && op != '/' {
                break;
            }
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.peek() == &Token::Sym('-') {
            self.advance();
            Ok(Expr::Neg(Box::new(self.parse_unary()?)))
        } else {
            self.parse_primary()
        }
    }

    fn parse_list(&mut self) -> Result<Expr, String> {
        self.advance(); // consume '['
        let mut items = Vec::new();
        while self.peek() != &Token::Sym(']') && !matches!(self.peek(), Token::Eof) {
            items.push(self.parse_pipe()?);
            if self.peek() == &Token::Sym(',') { self.advance(); } else { break; }
        }
        self.eat_sym(']')?;
        Ok(Expr::List(items))
    }

    fn parse_dict(&mut self) -> Result<Expr, String> {
        self.advance(); // consume '{'
        let mut pairs = Vec::new();
        while self.peek() != &Token::Sym('}') && !matches!(self.peek(), Token::Eof) {
            let key = match self.advance() {
                Token::Str(s)   => s,
                Token::Ident(s) => s,
                tok => return Err(format!("expected dict key, got {:?}", tok)),
            };
            self.eat_sym(':')?;
            let val = self.parse_pipe()?;
            pairs.push((key, val));
            if self.peek() == &Token::Sym(',') { self.advance(); } else { break; }
        }
        self.eat_sym('}')?;
        Ok(Expr::Dict(pairs))
    }

    fn apply_subscript(&mut self, mut expr: Expr) -> Result<Expr, String> {
        while self.peek() == &Token::Sym('[') {
            self.advance();
            let index = self.parse_pipe()?;
            self.eat_sym(']')?;
            expr = Expr::Index { object: Box::new(expr), index: Box::new(index) };
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.peek().clone() {
            Token::Number(n) => { self.advance(); self.apply_subscript(Expr::Number(n)) }
            Token::Str(s)    => { self.advance(); self.apply_subscript(Expr::Str(s)) }
            Token::Sym('(')  => {
                self.advance();
                let e = self.parse_pipe()?;
                self.eat_sym(')')?;
                self.apply_subscript(e)
            }
            Token::Sym('[') => { let e = self.parse_list()?; self.apply_subscript(e) }
            Token::Sym('{') => { let e = self.parse_dict()?; self.apply_subscript(e) }
            Token::Ident(name) => {
                self.advance();
                if self.peek() == &Token::Sym('(') {
                    self.advance();
                    let args = self.parse_call_args_paren()?;
                    self.apply_subscript(Expr::Call { name, args })
                } else if self.is_value_start() {
                    let args = self.parse_call_args_space()?;
                    self.apply_subscript(Expr::Call { name, args })
                } else {
                    self.apply_subscript(Expr::Ident(name))
                }
            }
            tok => Err(format!("unexpected token {:?}", tok)),
        }
    }

    fn is_value_start(&self) -> bool {
        matches!(
            self.peek(),
            Token::Number(_) | Token::Str(_) | Token::Ident(_)
                | Token::Sym('{')
        ) || (matches!(self.peek(), Token::Sym('[')) && self.peek_has_space())
    }

    fn parse_call_args_paren(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if self.peek() == &Token::Sym(')') {
            self.advance();
            return Ok(args);
        }
        loop {
            args.push(self.parse_pipe()?);
            match self.peek().clone() {
                Token::Sym(',') => {
                    self.advance();
                    if self.peek() == &Token::Sym(')') {
                        self.advance();
                        break;
                    }
                }
                Token::Sym(')') => {
                    self.advance();
                    break;
                }
                tok => return Err(format!("expected ',' or ')' in call, got {:?}", tok)),
            }
        }
        Ok(args)
    }

    fn parse_call_args_space(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        loop {
            args.push(self.parse_add()?);
            if self.peek() == &Token::Sym(',') {
                self.advance();
                if !self.is_value_start() {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(args)
    }
}

/// Parse a pre-lexed token slice. No whitespace info is available; subscript (`[`) is
/// assumed to have no preceding space (i.e. it is always a subscript, never a space-call
/// argument). For full accuracy, prefer [`parse_src`].
#[allow(dead_code)]
pub fn parse(tokens: &[Token]) -> Result<Expr, String> {
    // No spacing info available when called with a pre-lexed slice; treat all as un-spaced
    // so that `lst[0]` is a subscript, not a space-call. Use parse_src for full accuracy.
    let spaces: Vec<bool> = vec![false; tokens.len()];
    Parser::new(tokens, &spaces).parse_block()
}

pub fn parse_src(src: &str) -> Result<Expr, String> {
    let (tokens, spaces) = lex_with_spaces(src);
    Parser::new(&tokens, &spaces).parse_block()
}

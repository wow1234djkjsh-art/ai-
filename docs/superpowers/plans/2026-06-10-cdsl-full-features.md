# C-DSL Full Feature Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite the lexer and parser, add a full AST-walking interpreter, and connect everything so `execute(src)` supports variables, functions, conditionals, pipes, `each`, and `print`.

**Architecture:** Three-pass pipeline — `lex(src) → parse(tokens) → eval_expr(env, ast)`. The lexer is character-by-character (whitespace optional). The parser is recursive-descent producing a typed `Expr` AST. The interpreter walks the AST with a mutable `Environment`. `execute()` is rewritten to wire all three. The old `eval(env, src)` is kept as a wrapper so `builtin_eval` and existing interpreter tests require minimal changes.

**Tech Stack:** Rust, cargo, c-dsl v0.1.0

---

## File Map

| File | Change |
|------|--------|
| `src/lexer.rs` | Complete rewrite — char-by-char, new `Token` enum |
| `src/parser.rs` | Complete rewrite — full `Expr` AST, recursive descent |
| `src/interpreter.rs` | Add `eval_expr`, `eval_binop`, `call_fn`; change `Function.body` to `Expr`; rewrite `execute()` and `eval()` |
| `src/builtins.rs` | Add `builtin_print()` |
| `tests/lexer.rs` | New file |
| `tests/parser.rs` | Replace content |
| `tests/interpreter.rs` | Replace content (rewrite tests to use `execute()`) |
| `tests/integration.rs` | Add 4 new end-to-end tests |
| `src/cache.rs`, `src/lib.rs`, `src/main.rs` | No changes |

---

## Task 1: Rewrite Lexer

**Files:**
- Modify: `src/lexer.rs`
- Test: `tests/lexer.rs` (new)

New `Token` enum drops the old whitespace-split approach. Tokens: `Number(f64)`, `Str(String)`, `Ident(String)`, `Fn`, `Each`, `Arrow` (`=>`), `Sep` (`;` or `\n`), `Sym(char)` (single operator), `Eof`. Whitespace (space, tab) is silently skipped. `=>` is two chars but emits one `Arrow` token.

- [ ] **Step 1: Write failing tests** — create `tests/lexer.rs`:

```rust
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
```

- [ ] **Step 2: Run to confirm failure**

```
cargo test --test lexer 2>&1
```

Expected: compile errors — `Token` variants `Fn`, `Each`, `Arrow`, `Sep` do not exist yet.

- [ ] **Step 3: Replace `src/lexer.rs` entirely**

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Str(String),
    Ident(String),
    Fn,
    Each,
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
            ' ' | '\t' => { i += 1; }
            '\n' => { tokens.push(Token::Sep); i += 1; }
            ';'  => { tokens.push(Token::Sep); i += 1; }
            '='  => {
                if i + 1 < chars.len() && chars[i + 1] == '>' {
                    tokens.push(Token::Arrow);
                    i += 2;
                } else {
                    tokens.push(Token::Sym('='));
                    i += 1;
                }
            }
            '+' | '-' | '*' | '/' | '>' | '<' | '?' | ':' | '|' | ',' | '(' | ')' => {
                tokens.push(Token::Sym(chars[i]));
                i += 1;
            }
            '"' => {
                i += 1;
                let start = i;
                while i < chars.len() && chars[i] != '"' { i += 1; }
                let s: String = chars[start..i].iter().collect();
                tokens.push(Token::Str(s));
                if i < chars.len() { i += 1; }
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
                    "fn"   => tokens.push(Token::Fn),
                    "each" => tokens.push(Token::Each),
                    _      => tokens.push(Token::Ident(word)),
                }
            }
            _ => { i += 1; }
        }
    }
    tokens.push(Token::Eof);
    tokens
}
```

- [ ] **Step 4: Run tests to confirm all pass**

```
cargo test --test lexer 2>&1
```

Expected: 6 tests pass. (Other test files may fail to compile until later tasks — that is expected.)

- [ ] **Step 5: Commit**

```
git add src/lexer.rs tests/lexer.rs
git commit -m "feat: rewrite lexer with character-by-character tokenization"
```

---

## Task 2: Rewrite Parser

**Files:**
- Modify: `src/parser.rs`
- Modify: `tests/parser.rs`

Full recursive-descent parser. Precedence (low → high): Block → Stmt → Pipe (`|`) → Compare (`>` `<`) → Add/Sub → Mul/Div → Unary (`-`) → Primary.

Statement-level rules:
- `fn name p1,p2=>body` → `FnDef`
- `each e1,e2:func` → `Each` (func can be a `Lambda` or any expr)
- `?cond:then:else` → `If`
- `name=expr` (Ident followed by `=`) → `Assign`
- anything else → expression

In `parse_primary`, after an `Ident`:
- followed by `(` → paren call
- followed by `Number`, `Str`, or another `Ident` → space call (args are comma-separated `parse_add` exprs)
- otherwise → plain `Ident`

`Lambda` is an anonymous function used as the `func` part of `each`, or inside expressions: `fn p1,p2=>body` (no name after `fn`). Its body parses at `parse_add` level. `Neg` handles unary minus.

- [ ] **Step 1: Write failing tests** — replace ALL of `tests/parser.rs`:

```rust
use c_dsl::lexer::lex;
use c_dsl::parser::{parse, Expr};

fn p(src: &str) -> Expr {
    parse(&lex(src)).expect("parse failed")
}

#[test]
fn test_parse_number() {
    assert_eq!(p("42"), Expr::Block(vec![Expr::Number(42.0)]));
}

#[test]
fn test_parse_assign() {
    assert_eq!(p("x=5"), Expr::Block(vec![
        Expr::Assign { name: "x".into(), value: Box::new(Expr::Number(5.0)) }
    ]));
}

#[test]
fn test_parse_binop_add() {
    assert_eq!(p("1+2"), Expr::Block(vec![
        Expr::BinOp { op: '+', left: Box::new(Expr::Number(1.0)), right: Box::new(Expr::Number(2.0)) }
    ]));
}

#[test]
fn test_parse_neg() {
    assert_eq!(p("-x"), Expr::Block(vec![
        Expr::Neg(Box::new(Expr::Ident("x".into())))
    ]));
}

#[test]
fn test_parse_fn_def() {
    assert_eq!(p("fn add a,b=>a+b"), Expr::Block(vec![
        Expr::FnDef {
            name: "add".into(),
            params: vec!["a".into(), "b".into()],
            body: Box::new(Expr::BinOp {
                op: '+',
                left:  Box::new(Expr::Ident("a".into())),
                right: Box::new(Expr::Ident("b".into())),
            }),
        }
    ]));
}

#[test]
fn test_parse_call_space() {
    assert_eq!(p("add 1,2"), Expr::Block(vec![
        Expr::Call { name: "add".into(), args: vec![Expr::Number(1.0), Expr::Number(2.0)] }
    ]));
}

#[test]
fn test_parse_call_paren() {
    assert_eq!(p("add(1,2)"), Expr::Block(vec![
        Expr::Call { name: "add".into(), args: vec![Expr::Number(1.0), Expr::Number(2.0)] }
    ]));
}

#[test]
fn test_parse_if() {
    assert_eq!(p("?x>0:x:0"), Expr::Block(vec![
        Expr::If {
            cond:  Box::new(Expr::BinOp { op: '>', left: Box::new(Expr::Ident("x".into())), right: Box::new(Expr::Number(0.0)) }),
            then:  Box::new(Expr::Ident("x".into())),
            else_: Box::new(Expr::Number(0.0)),
        }
    ]));
}

#[test]
fn test_parse_pipe() {
    assert_eq!(p("add 1,2|print"), Expr::Block(vec![
        Expr::Pipe {
            left:  Box::new(Expr::Call { name: "add".into(), args: vec![Expr::Number(1.0), Expr::Number(2.0)] }),
            right: Box::new(Expr::Ident("print".into())),
        }
    ]));
}

#[test]
fn test_parse_block() {
    assert_eq!(p("x=1;y=2"), Expr::Block(vec![
        Expr::Assign { name: "x".into(), value: Box::new(Expr::Number(1.0)) },
        Expr::Assign { name: "y".into(), value: Box::new(Expr::Number(2.0)) },
    ]));
}

#[test]
fn test_parse_each() {
    assert_eq!(p("each 1,2:fn x=>x"), Expr::Block(vec![
        Expr::Each {
            items: vec![Expr::Number(1.0), Expr::Number(2.0)],
            func: Box::new(Expr::Lambda {
                params: vec!["x".into()],
                body:   Box::new(Expr::Ident("x".into())),
            }),
        }
    ]));
}
```

- [ ] **Step 2: Run to confirm failure**

```
cargo test --test parser 2>&1
```

Expected: compile errors — new `Expr` variants don't exist yet.

- [ ] **Step 3: Replace ALL of `src/parser.rs`**

```rust
use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Str(String),
    Ident(String),
    Neg(Box<Expr>),
    Assign { name: String, value: Box<Expr> },
    BinOp  { op: char, left: Box<Expr>, right: Box<Expr> },
    FnDef  { name: String, params: Vec<String>, body: Box<Expr> },
    Lambda { params: Vec<String>, body: Box<Expr> },
    Call   { name: String, args: Vec<Expr> },
    If     { cond: Box<Expr>, then: Box<Expr>, else_: Box<Expr> },
    Pipe   { left: Box<Expr>, right: Box<Expr> },
    Each   { items: Vec<Expr>, func: Box<Expr> },
    Block  (Vec<Expr>),
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        if self.pos < self.tokens.len() { self.pos += 1; }
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
        while self.peek() == &Token::Sep { self.advance(); }
    }

    fn parse_block(&mut self) -> Result<Expr, String> {
        self.skip_seps();
        let mut stmts = Vec::new();
        while !matches!(self.peek(), Token::Eof) {
            stmts.push(self.parse_stmt()?);
            if matches!(self.peek(), Token::Sep) {
                while matches!(self.peek(), Token::Sep) { self.advance(); }
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
                    let value = self.parse_pipe()?;
                    Ok(Expr::Assign { name, value: Box::new(value) })
                } else {
                    self.parse_pipe()
                }
            }
            _ => self.parse_pipe(),
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
        let body = self.parse_pipe()?;
        Ok(Expr::FnDef { name, params, body: Box::new(body) })
    }

    fn parse_lambda(&mut self) -> Result<Expr, String> {
        self.advance(); // Fn
        let params = self.parse_params()?;
        self.eat_arrow()?;
        let body = self.parse_add()?;
        Ok(Expr::Lambda { params, body: Box::new(body) })
    }

    fn parse_params(&mut self) -> Result<Vec<String>, String> {
        let mut params = Vec::new();
        while let Token::Ident(p) = self.peek().clone() {
            params.push(p);
            self.advance();
            if self.peek() == &Token::Sym(',') { self.advance(); }
        }
        Ok(params)
    }

    fn parse_each(&mut self) -> Result<Expr, String> {
        self.advance(); // Each
        let mut items = Vec::new();
        loop {
            if matches!(self.peek(), Token::Sym(':') | Token::Eof) { break; }
            items.push(self.parse_add()?);
            if self.peek() == &Token::Sym(',') { self.advance(); } else { break; }
        }
        self.eat_sym(':')?;
        let func = if self.peek() == &Token::Fn {
            self.parse_lambda()?
        } else {
            self.parse_pipe()?
        };
        Ok(Expr::Each { items, func: Box::new(func) })
    }

    fn parse_if(&mut self) -> Result<Expr, String> {
        self.advance(); // '?'
        let cond  = self.parse_cmp()?;
        self.eat_sym(':')?;
        let then  = self.parse_cmp()?;
        self.eat_sym(':')?;
        let else_ = self.parse_pipe()?;
        Ok(Expr::If { cond: Box::new(cond), then: Box::new(then), else_: Box::new(else_) })
    }

    fn parse_pipe(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_cmp()?;
        while self.peek() == &Token::Sym('|') {
            self.advance();
            let right = self.parse_cmp()?;
            left = Expr::Pipe { left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_cmp(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_add()?;
        while let Token::Sym(op) = self.peek().clone() {
            if op != '>' && op != '<' { break; }
            self.advance();
            let right = self.parse_add()?;
            left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_add(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_mul()?;
        while let Token::Sym(op) = self.peek().clone() {
            if op != '+' && op != '-' { break; }
            self.advance();
            let right = self.parse_mul()?;
            left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        while let Token::Sym(op) = self.peek().clone() {
            if op != '*' && op != '/' { break; }
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.peek() == &Token::Sym('-') {
            self.advance();
            Ok(Expr::Neg(Box::new(self.parse_primary()?)))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.peek().clone() {
            Token::Number(n) => { self.advance(); Ok(Expr::Number(n)) }
            Token::Str(s)    => { self.advance(); Ok(Expr::Str(s)) }
            Token::Sym('(')  => {
                self.advance();
                let e = self.parse_pipe()?;
                self.eat_sym(')')?;
                Ok(e)
            }
            Token::Ident(name) => {
                self.advance();
                if self.peek() == &Token::Sym('(') {
                    self.advance();
                    let args = self.parse_call_args_paren()?;
                    Ok(Expr::Call { name, args })
                } else if self.is_value_start() {
                    let args = self.parse_call_args_space()?;
                    Ok(Expr::Call { name, args })
                } else {
                    Ok(Expr::Ident(name))
                }
            }
            tok => Err(format!("unexpected token {:?}", tok)),
        }
    }

    fn is_value_start(&self) -> bool {
        matches!(self.peek(), Token::Number(_) | Token::Str(_) | Token::Ident(_))
    }

    fn parse_call_args_paren(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if self.peek() == &Token::Sym(')') { self.advance(); return Ok(args); }
        loop {
            args.push(self.parse_add()?);
            match self.peek().clone() {
                Token::Sym(',') => { self.advance(); }
                Token::Sym(')') => { self.advance(); break; }
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
                if !self.is_value_start() { break; }
            } else {
                break;
            }
        }
        Ok(args)
    }
}

pub fn parse(tokens: &[Token]) -> Result<Expr, String> {
    Parser::new(tokens).parse_block()
}
```

- [ ] **Step 4: Run tests to confirm all pass**

```
cargo test --test parser 2>&1
```

Expected: 11 tests pass.

- [ ] **Step 5: Commit**

```
git add src/parser.rs tests/parser.rs
git commit -m "feat: rewrite parser with full recursive-descent AST"
```

---

## Task 3: Rewrite Interpreter

**Files:**
- Modify: `src/interpreter.rs`
- Modify: `tests/interpreter.rs`

Three changes:
1. `Function.body` changes from `String` to `Expr` (needs `use crate::parser::Expr`).
2. Add `eval_expr(env: &mut Environment, expr: &Expr) -> Value`, `eval_binop`, and `call_fn`.
3. Rewrite `execute()` to call `lex → parse → eval_expr`. Rewrite `eval()` as a wrapper (clones env so `builtin_eval` still compiles unchanged).

`Function::call()` is also updated to use `eval_expr`.

- [ ] **Step 1: Write failing tests** — replace ALL of `tests/interpreter.rs`:

```rust
use c_dsl::interpreter::{execute, Value};

#[test]
fn test_number_literal() {
    assert_eq!(execute("42"), Value::Number(42.0));
}

#[test]
fn test_string_literal() {
    assert_eq!(execute("\"hello\""), Value::String("hello".into()));
}

#[test]
fn test_arithmetic() {
    assert_eq!(execute("2+3"),   Value::Number(5.0));
    assert_eq!(execute("10-4"),  Value::Number(6.0));
    assert_eq!(execute("6*7"),   Value::Number(42.0));
    assert_eq!(execute("20/4"),  Value::Number(5.0));
}

#[test]
fn test_variable_assign_and_lookup() {
    assert_eq!(execute("x=10;x"), Value::Number(10.0));
}

#[test]
fn test_fn_def_and_call() {
    assert_eq!(execute("fn double x=>x*2;double 5"), Value::Number(10.0));
}

#[test]
fn test_fn_call_paren() {
    assert_eq!(execute("fn add a,b=>a+b;add(3,4)"), Value::Number(7.0));
}

#[test]
fn test_conditional_true() {
    assert_eq!(execute("x=5;?x>3:x*2:0"), Value::Number(10.0));
}

#[test]
fn test_conditional_false() {
    assert_eq!(execute("x=1;?x>3:x*2:0"), Value::Number(0.0));
}

#[test]
fn test_pipe_to_builtin() {
    // print returns its argument; check the value comes through
    assert_eq!(execute("fn double x=>x*2;3|double"), Value::Number(6.0));
}

#[test]
fn test_pipe_chain() {
    assert_eq!(execute("fn double x=>x*2;3|double|double"), Value::Number(12.0));
}

#[test]
fn test_neg() {
    assert_eq!(execute("x=5;-x"), Value::Number(-5.0));
}

#[test]
fn test_each() {
    assert_eq!(execute("each 1,2,3:fn x=>x*2"), Value::Number(6.0));
}
```

- [ ] **Step 2: Run to confirm failure**

```
cargo test --test interpreter 2>&1
```

Expected: compile errors or test failures — `execute` not implemented correctly yet.

- [ ] **Step 3: Replace ALL of `src/interpreter.rs`**

```rust
use std::collections::HashMap;
use std::rc::Rc;
use crate::lexer::lex;
use crate::parser::{parse, Expr};

#[derive(Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Function(Function),
    Nil,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Function(a), Value::Function(b)) => a.params == b.params && a.body == b.body,
            _ => false,
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n)   => write!(f, "Number({})", n),
            Value::String(s)   => write!(f, "String({})", s),
            Value::Function(_) => write!(f, "Function(...)"),
            Value::Nil         => write!(f, "Nil"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Environment {
    name_value: HashMap<String, Value>,
    parent: Option<Rc<Environment>>,
}

impl Default for Environment {
    fn default() -> Self { Self::new() }
}

impl Environment {
    pub fn new() -> Self {
        Environment { name_value: HashMap::new(), parent: None }
    }
    pub fn with_parent(parent: Environment) -> Self {
        Environment { name_value: HashMap::new(), parent: Some(Rc::new(parent)) }
    }
    pub fn find(&self, name: &str) -> Option<Value> {
        self.name_value.get(name).cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.find(name)))
    }
    pub fn define(&mut self, name: String, value: Value) {
        self.name_value.insert(name, value);
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub params: Vec<String>,
    pub body: Expr,
    pub parent_env: Rc<Environment>,
}

impl Function {
    pub fn call(&self, args: Vec<Value>) -> Value {
        let parent = (*self.parent_env).clone();
        let mut env = Environment::with_parent(parent);
        for (param, arg) in self.params.iter().zip(args) {
            env.define(param.clone(), arg);
        }
        eval_expr(&mut env, &self.body)
    }
}

pub fn eval_expr(env: &mut Environment, expr: &Expr) -> Value {
    match expr {
        Expr::Number(n)  => Value::Number(*n),
        Expr::Str(s)     => Value::String(s.clone()),
        Expr::Ident(name) => env.find(name).unwrap_or(Value::Nil),
        Expr::Neg(inner) => match eval_expr(env, inner) {
            Value::Number(n) => Value::Number(-n),
            _ => Value::Nil,
        },
        Expr::Block(stmts) => {
            let mut last = Value::Nil;
            for stmt in stmts { last = eval_expr(env, stmt); }
            last
        }
        Expr::Assign { name, value } => {
            let val = eval_expr(env, value);
            env.define(name.clone(), val.clone());
            val
        }
        Expr::BinOp { op, left, right } => {
            let l = eval_expr(env, left);
            let r = eval_expr(env, right);
            eval_binop(*op, l, r)
        }
        Expr::FnDef { name, params, body } => {
            let f = Value::Function(Function {
                params: params.clone(),
                body: *body.clone(),
                parent_env: Rc::new(env.clone()),
            });
            env.define(name.clone(), f.clone());
            f
        }
        Expr::Lambda { params, body } => {
            Value::Function(Function {
                params: params.clone(),
                body: *body.clone(),
                parent_env: Rc::new(env.clone()),
            })
        }
        Expr::Call { name, args } => {
            let eval_args: Vec<Value> = args.iter().map(|a| eval_expr(env, a)).collect();
            call_fn(env, name, eval_args)
        }
        Expr::If { cond, then, else_ } => {
            let truthy = match eval_expr(env, cond) {
                Value::Number(n) => n != 0.0,
                Value::String(s) => !s.is_empty(),
                Value::Nil       => false,
                Value::Function(_) => true,
            };
            if truthy { eval_expr(env, then) } else { eval_expr(env, else_) }
        }
        Expr::Pipe { left, right } => {
            let left_val = eval_expr(env, left);
            match right.as_ref() {
                Expr::Ident(name) => call_fn(env, name, vec![left_val]),
                Expr::Call { name, args } => {
                    let mut eval_args: Vec<Value> =
                        args.iter().map(|a| eval_expr(env, a)).collect();
                    eval_args.insert(0, left_val);
                    call_fn(env, name, eval_args)
                }
                _ => Value::Nil,
            }
        }
        Expr::Each { items, func } => {
            let func_val = eval_expr(env, func);
            let mut last = Value::Nil;
            for item in items {
                let item_val = eval_expr(env, item);
                last = match &func_val {
                    Value::Function(f) => f.call(vec![item_val]),
                    _ => Value::Nil,
                };
            }
            last
        }
    }
}

fn eval_binop(op: char, left: Value, right: Value) -> Value {
    match (op, &left, &right) {
        ('+', Value::Number(l), Value::Number(r)) => Value::Number(l + r),
        ('-', Value::Number(l), Value::Number(r)) => Value::Number(l - r),
        ('*', Value::Number(l), Value::Number(r)) => Value::Number(l * r),
        ('/', Value::Number(l), Value::Number(r)) => Value::Number(l / r),
        ('>', Value::Number(l), Value::Number(r)) => Value::Number(if l > r { 1.0 } else { 0.0 }),
        ('<', Value::Number(l), Value::Number(r)) => Value::Number(if l < r { 1.0 } else { 0.0 }),
        ('+', Value::String(l), Value::String(r)) => Value::String(l.clone() + r),
        _ => Value::Nil,
    }
}

fn call_fn(env: &mut Environment, name: &str, args: Vec<Value>) -> Value {
    match name {
        "print" => crate::builtins::builtin_print(args),
        "eval"  => crate::builtins::builtin_eval(env, args),
        "model" => crate::builtins::model(env, args),
        _ => match env.find(name) {
            Some(Value::Function(f)) => f.call(args),
            _ => Value::Nil,
        }
    }
}

/// Execute source: lex → parse → eval_expr with a fresh mutable environment.
pub fn execute(src: &str) -> Value {
    let tokens = lex(src);
    match parse(&tokens) {
        Ok(ast) => { let mut env = Environment::new(); eval_expr(&mut env, &ast) }
        Err(_)  => Value::Nil,
    }
}

/// Legacy wrapper kept for builtin_eval compatibility. Clones env so callers
/// see an immutable view; assignments inside eval do not escape.
pub fn eval(env: &Environment, src: &str) -> Value {
    let tokens = lex(src);
    match parse(&tokens) {
        Ok(ast) => { let mut local = env.clone(); eval_expr(&mut local, &ast) }
        Err(_)  => Value::Nil,
    }
}
```

- [ ] **Step 4: Run tests to confirm all pass**

```
cargo test --test interpreter 2>&1
```

Expected: 12 tests pass.

- [ ] **Step 5: Run the full suite to check nothing regressed**

```
cargo test 2>&1
```

Expected: all tests pass (lexer, parser, interpreter, integration, basic).

- [ ] **Step 6: Commit**

```
git add src/interpreter.rs tests/interpreter.rs
git commit -m "feat: add eval_expr AST walker, connect execute() to full pipeline"
```

---

## Task 4: Add print Builtin

**Files:**
- Modify: `src/builtins.rs`

`builtin_print` takes `Vec<Value>`, prints the first value to stdout, and returns it (so `x|print` is transparent — the value flows through).

- [ ] **Step 1: Write a failing test** — add to `tests/interpreter.rs`:

```rust
#[test]
fn test_print_returns_value() {
    // print is transparent: the value it receives is also returned
    assert_eq!(execute("fn double x=>x*2;4|double|double"), Value::Number(16.0));
}
```

Run:
```
cargo test test_print_returns_value 2>&1
```

Expected: PASS (this test doesn't test print directly — it verifies pipe chain works, which is needed before the print integration test). Now add a direct print test:

```rust
#[test]
fn test_print_pipeline() {
    // 5 piped through print — value should come through unchanged
    let result = execute("5|print");
    assert_eq!(result, Value::Number(5.0));
}
```

Run:
```
cargo test test_print_pipeline 2>&1
```

Expected: FAIL — `builtin_print` not defined yet.

- [ ] **Step 2: Add `builtin_print` to `src/builtins.rs`**

Add this function at the end of the file (do not modify or remove existing functions):

```rust
/// Print the first argument to stdout and return it unchanged.
pub fn builtin_print(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(v) => {
            match &v {
                Value::Number(n)   => println!("{}", n),
                Value::String(s)   => println!("{}", s),
                Value::Nil         => println!("nil"),
                Value::Function(_) => println!("<fn>"),
            }
            v
        }
        None => Value::Nil,
    }
}
```

- [ ] **Step 3: Run tests to confirm pass**

```
cargo test test_print 2>&1
```

Expected: both print tests pass.

- [ ] **Step 4: Run full suite**

```
cargo test 2>&1
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```
git add src/builtins.rs tests/interpreter.rs
git commit -m "feat: add print builtin, returns value for transparent pipe chaining"
```

---

## Task 5: Add Integration Tests

**Files:**
- Modify: `tests/integration.rs`

Four end-to-end tests that exercise the full pipeline: source string → `execute()` → `Value`. The existing 3 tests in this file must continue to pass untouched.

- [ ] **Step 1: Add tests to `tests/integration.rs`** — append after the existing tests:

```rust
use c_dsl::interpreter::{execute, Value};

#[test]
fn test_e2e_variable_conditional() {
    assert_eq!(execute("x=5;?x>3:x*2:0"), Value::Number(10.0));
}

#[test]
fn test_e2e_fn_def_and_call() {
    assert_eq!(execute("fn add a,b=>a+b;add 3,4"), Value::Number(7.0));
}

#[test]
fn test_e2e_pipe_chain() {
    assert_eq!(execute("fn double x=>x*2;3|double|double"), Value::Number(12.0));
}

#[test]
fn test_e2e_each() {
    // each returns the last result
    assert_eq!(execute("each 1,2,3:fn x=>x*2"), Value::Number(6.0));
}
```

- [ ] **Step 2: Run to confirm all pass**

```
cargo test --test integration 2>&1
```

Expected: all 7 tests pass (3 existing + 4 new).

- [ ] **Step 3: Run full suite one final time**

```
cargo test 2>&1
```

Expected: all tests pass across every test file.

- [ ] **Step 4: Commit**

```
git add tests/integration.rs
git commit -m "test: add end-to-end integration tests for all language features"
```

# C-DSL Collections, Logical Operators & Line Continuation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend C-DSL with list/dict literals and bracket indexing, `and`/`or`/`not` logical operators, and `\` line continuation.

**Architecture:** Three independent layers extended in order: lexer tokens first, then AST+parser+eval for collections, then AST+parser+eval for logical operators. Rust's exhaustive match means new `Expr` variants and their `eval_expr` arms must land in the same commit.

**Tech Stack:** Rust 2021 edition. No new dependencies. Files changed: `src/lexer.rs`, `src/parser.rs`, `src/interpreter.rs`, `tests/lexer.rs`, `tests/parser.rs`, `tests/interpreter.rs`.

---

### Task 1: Lexer extensions

**Files:**
- Modify: `src/lexer.rs`
- Modify: `tests/lexer.rs`

- [ ] **Step 1: Write failing lexer tests**

Append to `tests/lexer.rs`:

```rust
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
```

- [ ] **Step 2: Run tests — verify they fail**

```bash
cd c-dsl && cargo test test_lex_brackets test_lex_braces test_lex_logical_keywords test_lex_line_continuation 2>&1
```

Expected: compile error — `Token::And`, `Token::Or`, `Token::Not` not found.

- [ ] **Step 3: Add three keyword tokens to the Token enum**

In `src/lexer.rs`, replace the `Token` enum with:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Str(String),
    Ident(String),
    Fn,
    Each,
    Arrow,
    And,
    Or,
    Not,
    Sep,
    Sym(char),
    Eof,
}
```

- [ ] **Step 4: Add `[`, `]`, `{`, `}` to the symbol dispatch**

In `src/lexer.rs`, replace this line:

```rust
'+' | '-' | '*' | '/' | '>' | '<' | '?' | ':' | '|' | ',' | '(' | ')' => {
```

with:

```rust
'+' | '-' | '*' | '/' | '>' | '<' | '?' | ':' | '|' | ',' | '(' | ')'
| '[' | ']' | '{' | '}' => {
```

- [ ] **Step 5: Add `and`, `or`, `not` keyword matching and `\` continuation**

In `src/lexer.rs`, replace:

```rust
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
            _ => {
                i += 1;
            }
```

with:

```rust
            c if c.is_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                match word.as_str() {
                    "fn"   => tokens.push(Token::Fn),
                    "each" => tokens.push(Token::Each),
                    "and"  => tokens.push(Token::And),
                    "or"   => tokens.push(Token::Or),
                    "not"  => tokens.push(Token::Not),
                    _      => tokens.push(Token::Ident(word)),
                }
            }
            '\\' => {
                if i + 1 < chars.len() && chars[i + 1] == '\n' {
                    i += 2; // backslash + newline → skip both, no Sep emitted
                } else {
                    i += 1;
                }
            }
            _ => {
                i += 1;
            }
```

- [ ] **Step 6: Run tests — all four new tests must pass**

```bash
cargo test test_lex_brackets test_lex_braces test_lex_logical_keywords test_lex_line_continuation 2>&1
```

Expected: 4 passed, 0 failed. Also run `cargo test` to confirm existing 51 tests still pass.

- [ ] **Step 7: Commit**

```bash
git add src/lexer.rs tests/lexer.rs
git commit -m "feat: lexer — brackets/braces, and/or/not keywords, backslash continuation"
```

---

### Task 2: Collection types — List, Dict, Index

**Files:**
- Modify: `src/parser.rs`
- Modify: `src/interpreter.rs`
- Modify: `tests/parser.rs`
- Modify: `tests/interpreter.rs`

- [ ] **Step 1: Write failing parser tests for collections**

Append to `tests/parser.rs`:

```rust
#[test]
fn test_parse_list_literal() {
    assert_eq!(
        p("[1,2,3]"),
        Expr::Block(vec![Expr::List(vec![
            Expr::Number(1.0),
            Expr::Number(2.0),
            Expr::Number(3.0),
        ])])
    );
}

#[test]
fn test_parse_empty_list() {
    assert_eq!(p("[]"), Expr::Block(vec![Expr::List(vec![])]));
}

#[test]
fn test_parse_dict_literal() {
    assert_eq!(
        p("{a:1}"),
        Expr::Block(vec![Expr::Dict(vec![
            ("a".to_string(), Expr::Number(1.0))
        ])])
    );
}

#[test]
fn test_parse_index_ident() {
    assert_eq!(
        p("lst[0]"),
        Expr::Block(vec![Expr::Index {
            object: Box::new(Expr::Ident("lst".into())),
            index:  Box::new(Expr::Number(0.0)),
        }])
    );
}

#[test]
fn test_parse_index_dict() {
    assert_eq!(
        p("d[\"k\"]"),
        Expr::Block(vec![Expr::Index {
            object: Box::new(Expr::Ident("d".into())),
            index:  Box::new(Expr::Str("k".into())),
        }])
    );
}

#[test]
fn test_parse_index_chain() {
    assert_eq!(
        p("lst[0][1]"),
        Expr::Block(vec![Expr::Index {
            object: Box::new(Expr::Index {
                object: Box::new(Expr::Ident("lst".into())),
                index:  Box::new(Expr::Number(0.0)),
            }),
            index: Box::new(Expr::Number(1.0)),
        }])
    );
}

#[test]
fn test_parse_inline_list_index() {
    // list literal immediately subscripted
    assert_eq!(
        p("[10,20][1]"),
        Expr::Block(vec![Expr::Index {
            object: Box::new(Expr::List(vec![Expr::Number(10.0), Expr::Number(20.0)])),
            index:  Box::new(Expr::Number(1.0)),
        }])
    );
}
```

- [ ] **Step 2: Run to confirm compile failure**

```bash
cargo test test_parse_list test_parse_empty test_parse_dict test_parse_index 2>&1
```

Expected: compile error — `Expr::List`, `Expr::Dict`, `Expr::Index` not found.

- [ ] **Step 3: Add new Expr variants and Value variants**

In `src/parser.rs`, add three variants to the `Expr` enum (after `Block`):

```rust
    List(Vec<Expr>),
    Dict(Vec<(String, Expr)>),
    Index { object: Box<Expr>, index: Box<Expr> },
```

In `src/interpreter.rs`, add two variants to the `Value` enum (after `Function`):

```rust
    List(Vec<Value>),
    Dict(Vec<(String, Value)>),
```

Update the `PartialEq` impl for `Value` — add these arms inside the `match (self, other)` block:

```rust
            (Value::List(a),  Value::List(b))  => a == b,
            (Value::Dict(a),  Value::Dict(b))  => a == b,
```

Update the `Debug` impl for `Value` — add these arms:

```rust
            Value::List(items) => write!(f, "List({:?})", items),
            Value::Dict(pairs) => write!(f, "Dict({:?})", pairs),
```

Update the `Display` impl for `Value` — add these arms:

```rust
            Value::List(items) => {
                let inner: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", inner.join(", "))
            }
            Value::Dict(pairs) => {
                let inner: Vec<String> = pairs.iter()
                    .map(|(k, v)| format!("{}:{}", k, v))
                    .collect();
                write!(f, "{{{}}}", inner.join(", "))
            }
```

- [ ] **Step 4: Extract is_truthy helper, update If arm, add eval cases for new Expr variants**

Adding `Value::List` and `Value::Dict` makes the existing truthiness `match` inside `Expr::If` non-exhaustive — extract it into a helper first to keep the code compilable.

In `src/interpreter.rs`, add `is_truthy` as a free function directly before `eval_binop`:

```rust
fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Number(n)   => *n != 0.0,
        Value::String(s)   => !s.is_empty(),
        Value::Nil         => false,
        Value::Function(_) => true,
        Value::List(items) => !items.is_empty(),
        Value::Dict(pairs) => !pairs.is_empty(),
    }
}
```

Replace the `Expr::If` arm in `eval_expr` with:

```rust
        Expr::If { cond, then, else_ } => {
            if is_truthy(&eval_expr(env, cond)) {
                eval_expr(env, then)
            } else {
                eval_expr(env, else_)
            }
        }
```

Then add these three new arms after `Expr::Each`:

```rust
        Expr::List(items) => {
            Value::List(items.iter().map(|e| eval_expr(env, e)).collect())
        }
        Expr::Dict(pairs) => {
            Value::Dict(
                pairs.iter()
                    .map(|(k, v)| (k.clone(), eval_expr(env, v)))
                    .collect(),
            )
        }
        Expr::Index { object, index } => {
            let obj_val = eval_expr(env, object);
            let idx_val = eval_expr(env, index);
            match (obj_val, idx_val) {
                (Value::List(items), Value::Number(n)) => {
                    items.get(n as usize).cloned().unwrap_or(Value::Nil)
                }
                (Value::Dict(pairs), Value::String(key)) => {
                    pairs.into_iter()
                        .find(|(k, _)| k == &key)
                        .map(|(_, v)| v)
                        .unwrap_or(Value::Nil)
                }
                _ => Value::Nil,
            }
        }
```

- [ ] **Step 5: Add parser methods — parse_list, parse_dict, apply_subscript**

In `src/parser.rs`, inside the `impl<'a> Parser<'a>` block, add these three methods (place them before `parse_primary`):

```rust
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
```

- [ ] **Step 6: Rewrite parse_primary to handle `[`, `{`, and subscript loop**

In `src/parser.rs`, replace the entire `parse_primary` method with:

```rust
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
```

- [ ] **Step 7: Update is_value_start to include `[` and `{`**

In `src/parser.rs`, replace `is_value_start`:

```rust
    fn is_value_start(&self) -> bool {
        matches!(
            self.peek(),
            Token::Number(_) | Token::Str(_) | Token::Ident(_)
                | Token::Sym('[') | Token::Sym('{')
        )
    }
```

- [ ] **Step 8: Update parse_call_args_paren to allow full expressions**

In `src/parser.rs`, replace `parse_call_args_paren` (currently uses `parse_add` — upgrade to `parse_pipe` so `fn(1 and 2)` works):

```rust
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
```

- [ ] **Step 9: Run parser tests — all new tests must pass**

```bash
cargo test test_parse_list test_parse_empty test_parse_dict test_parse_index 2>&1
```

Expected: 7 new tests pass. Also run `cargo test` — all 51 existing tests still pass.

- [ ] **Step 10: Write failing interpreter tests for collections**

Append to `tests/interpreter.rs`:

```rust
#[test]
fn test_list_indexing() {
    assert_eq!(execute("[10,20,30][1]"), Value::Number(20.0));
}

#[test]
fn test_list_assign_and_index() {
    assert_eq!(execute("lst=[1,2,3];lst[0]"), Value::Number(1.0));
}

#[test]
fn test_dict_literal_lookup() {
    assert_eq!(execute("{x:42}[\"x\"]"), Value::Number(42.0));
}

#[test]
fn test_dict_assign_and_lookup() {
    assert_eq!(
        execute("d={name:\"Alice\"};d[\"name\"]"),
        Value::String("Alice".into())
    );
}

#[test]
fn test_index_out_of_bounds() {
    assert_eq!(execute("[1,2][9]"), Value::Nil);
}

#[test]
fn test_dict_missing_key() {
    assert_eq!(execute("{a:1}[\"b\"]"), Value::Nil);
}

#[test]
fn test_list_in_fn_call() {
    assert_eq!(execute("fn first lst=>lst[0];first [7,8,9]"), Value::Number(7.0));
}
```

- [ ] **Step 11: Run interpreter tests — all new tests must pass**

```bash
cargo test test_list test_dict test_index 2>&1
```

Expected: 7 new tests pass. Run `cargo test` — all tests pass.

- [ ] **Step 12: Commit**

```bash
git add src/parser.rs src/interpreter.rs tests/parser.rs tests/interpreter.rs
git commit -m "feat: List/Dict/Index types with bracket syntax and subscript indexing"
```

---

### Task 3: Logical operators — and, or, not

**Files:**
- Modify: `src/parser.rs`
- Modify: `src/interpreter.rs`
- Modify: `tests/parser.rs`
- Modify: `tests/interpreter.rs`

- [ ] **Step 1: Write failing parser tests for logical operators**

Append to `tests/parser.rs`:

```rust
#[test]
fn test_parse_and() {
    assert_eq!(
        p("a and b"),
        Expr::Block(vec![Expr::And {
            left:  Box::new(Expr::Ident("a".into())),
            right: Box::new(Expr::Ident("b".into())),
        }])
    );
}

#[test]
fn test_parse_or() {
    assert_eq!(
        p("a or b"),
        Expr::Block(vec![Expr::Or {
            left:  Box::new(Expr::Ident("a".into())),
            right: Box::new(Expr::Ident("b".into())),
        }])
    );
}

#[test]
fn test_parse_not() {
    assert_eq!(
        p("not a"),
        Expr::Block(vec![Expr::Not(Box::new(Expr::Ident("a".into())))])
    );
}

#[test]
fn test_parse_or_binds_looser_than_and() {
    // a or b and c  →  a or (b and c)
    assert_eq!(
        p("a or b and c"),
        Expr::Block(vec![Expr::Or {
            left: Box::new(Expr::Ident("a".into())),
            right: Box::new(Expr::And {
                left:  Box::new(Expr::Ident("b".into())),
                right: Box::new(Expr::Ident("c".into())),
            }),
        }])
    );
}

#[test]
fn test_parse_not_binds_tighter_than_and() {
    // not a and b  →  (not a) and b
    assert_eq!(
        p("not a and b"),
        Expr::Block(vec![Expr::And {
            left:  Box::new(Expr::Not(Box::new(Expr::Ident("a".into())))),
            right: Box::new(Expr::Ident("b".into())),
        }])
    );
}
```

- [ ] **Step 2: Run to confirm compile failure**

```bash
cargo test test_parse_and test_parse_or test_parse_not test_parse_or_binds test_parse_not_binds 2>&1
```

Expected: compile error — `Expr::And`, `Expr::Or`, `Expr::Not` not found.

- [ ] **Step 3: Add And, Or, Not to the Expr enum**

In `src/parser.rs`, add three variants to the `Expr` enum (place them after `Index`):

```rust
    And { left: Box<Expr>, right: Box<Expr> },
    Or  { left: Box<Expr>, right: Box<Expr> },
    Not (Box<Expr>),
```

- [ ] **Step 4: Add eval cases for And, Or, Not**

`is_truthy` was already extracted in Task 2. Add these three arms to `eval_expr` after the `Expr::Index` arm:

```rust
        Expr::And { left, right } => {
            let l = eval_expr(env, left);
            if !is_truthy(&l) { return Value::Number(0.0); }
            let r = eval_expr(env, right);
            Value::Number(if is_truthy(&r) { 1.0 } else { 0.0 })
        }
        Expr::Or { left, right } => {
            let l = eval_expr(env, left);
            if is_truthy(&l) { return Value::Number(1.0); }
            let r = eval_expr(env, right);
            Value::Number(if is_truthy(&r) { 1.0 } else { 0.0 })
        }
        Expr::Not(inner) => {
            Value::Number(if is_truthy(&eval_expr(env, inner)) { 0.0 } else { 1.0 })
        }
```

- [ ] **Step 5: Add parse_or, parse_and, parse_not; update parse_pipe**

In `src/parser.rs`, replace the `parse_pipe` method with:

```rust
    fn parse_pipe(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_or()?;
        while self.peek() == &Token::Sym('|') {
            self.advance();
            let right = self.parse_or()?;
            left = Expr::Pipe { left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }
```

Add these three new methods after `parse_pipe`:

```rust
    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while self.peek() == &Token::Or {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::Or { left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_not()?;
        while self.peek() == &Token::And {
            self.advance();
            let right = self.parse_not()?;
            left = Expr::And { left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> Result<Expr, String> {
        if self.peek() == &Token::Not {
            self.advance();
            Ok(Expr::Not(Box::new(self.parse_not()?)))
        } else {
            self.parse_cmp()
        }
    }
```

- [ ] **Step 6: Run parser tests — all new tests must pass**

```bash
cargo test test_parse_and test_parse_or test_parse_not test_parse_or_binds test_parse_not_binds 2>&1
```

Expected: 5 new tests pass. Run `cargo test` — all existing tests still pass.

- [ ] **Step 7: Write failing interpreter tests for logical operators**

Append to `tests/interpreter.rs`:

```rust
#[test]
fn test_logical_and_true() {
    assert_eq!(execute("1>0 and 2>0"), Value::Number(1.0));
}

#[test]
fn test_logical_and_false() {
    assert_eq!(execute("1>0 and 0>1"), Value::Number(0.0));
}

#[test]
fn test_logical_or_true() {
    assert_eq!(execute("0>1 or 1>0"), Value::Number(1.0));
}

#[test]
fn test_logical_or_false() {
    assert_eq!(execute("0>1 or 0>1"), Value::Number(0.0));
}

#[test]
fn test_logical_not_true() {
    assert_eq!(execute("not 0>1"), Value::Number(1.0));
}

#[test]
fn test_logical_not_false() {
    assert_eq!(execute("not 1>0"), Value::Number(0.0));
}

#[test]
fn test_short_circuit_and() {
    // LHS is falsy → result is 0, RHS not evaluated
    assert_eq!(execute("0 and 1>0"), Value::Number(0.0));
}

#[test]
fn test_short_circuit_or() {
    // LHS is truthy → result is 1, RHS not evaluated
    assert_eq!(execute("1 or 0>1"), Value::Number(1.0));
}

#[test]
fn test_logical_precedence_not_and() {
    // not 0 and 1  →  (not 0) and 1  →  1 and 1  →  1
    assert_eq!(execute("not 0 and 1"), Value::Number(1.0));
}

#[test]
fn test_logical_precedence_or_and() {
    // 0 or 1 and 1  →  0 or (1 and 1)  →  0 or 1  →  1
    assert_eq!(execute("0 or 1 and 1"), Value::Number(1.0));
}

#[test]
fn test_line_continuation() {
    // backslash suppresses the newline; 1 + 2 parsed as single expression
    assert_eq!(execute("1\\\n+2"), Value::Number(3.0));
}

#[test]
fn test_logical_in_conditional() {
    // conditional using and
    assert_eq!(execute("x=5;?x>3 and x<10:1:0"), Value::Number(1.0));
}
```

- [ ] **Step 8: Run interpreter tests — all new tests must pass**

```bash
cargo test test_logical test_line_continuation 2>&1
```

Expected: 12 new tests pass.

- [ ] **Step 9: Run full test suite**

```bash
cargo test 2>&1
```

Expected: all tests pass (51 original + 7 collection parser + 7 collection interpreter + 5 logical parser + 12 logical interpreter = 82 total).

- [ ] **Step 10: Commit**

```bash
git add src/parser.rs src/interpreter.rs tests/parser.rs tests/interpreter.rs
git commit -m "feat: logical operators and/or/not with short-circuit evaluation"
```

---

**Plan complete and saved to `docs/superpowers/plans/2026-06-10-cdsl-collections-logic-multiline.md`.**

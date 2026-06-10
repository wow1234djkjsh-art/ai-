# Error System Design

**Date:** 2026-06-10
**Status:** Approved

## Problem

The interpreter has two layers of broken error reporting:

1. `Value::Error(String)` exists as an enum variant and propagates through most eval paths, but many operations silently return `Value::Nil` instead — undefined variables, type mismatches, division by zero, out-of-bounds index.
2. `execute()` ignores the return value entirely in `main.rs`, so even correctly-propagated `Value::Error` values are never shown to the user. Parse errors are swallowed as `Nil`.

Result: bugs pass silently, debugging is nearly impossible.

## Goals

- All runtime failures produce a `Value::Error`, never `Value::Nil`.
- Errors are first-class values: inspectable via `.type` and `.message` field access.
- Dot notation (`x.field`) works for both errors and dicts.
- `execute()` surfaces errors to stderr and exits with code 1.
- `try/catch` available as an optional error-handling construct (Phase 2).

## Non-Goals

- Source span / line numbers in errors.
- Error `kind` field beyond `"error"` (e.g. `"arity_mismatch"`) — future work.
- Stack traces.

---

## Phase 1: Core Error System

### 1. Lexer — new `.` token

Add `Token::Dot` to the lexer. The `.` character currently has no meaning; it will now be recognised as a postfix accessor token.

### 2. Parser / AST — FieldAccess node

```rust
Expr::FieldAccess { object: Box<Expr>, field: String }
```

Parsing rule: after any primary expression (ident, call, index, paren), a `.` followed by an identifier produces a `FieldAccess` node. This has the same precedence as `Index` (postfix), so chaining works naturally:

```
err.message              → FieldAccess(Ident("err"), "message")
list[0].name             → FieldAccess(Index(list, 0), "name")
get_user().type          → FieldAccess(Call("get_user", []), "type")
```

### 3. Interpreter — FieldAccess evaluation

```rust
Expr::FieldAccess { object, field } => {
    let obj = eval_expr(env, object);
    match (&obj, field.as_str()) {
        (Value::Error(msg), "message") => Value::String(msg.clone()),
        (Value::Error(_),   "type")    => Value::String("error".into()),
        (Value::Dict(pairs), key) => pairs.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
            .unwrap_or(Value::Nil),
        _ => Value::Error(format!("no field '{}' on {}", field, obj)),
    }
}
```

### 4. Interpreter — silent Nil → Error conversions

Every place that currently returns `Value::Nil` on a failure condition must return `Value::Error` instead:

| Location | Trigger | New error message |
|----------|---------|-------------------|
| `Expr::Ident` | variable not found | `"undefined variable: {name}"` |
| `eval_binop` catch-all | type mismatch | `"type error: '{op}' not supported for these types"` |
| `eval_binop` `/` | division by zero | `"division by zero"` |
| `Expr::Neg` catch-all | non-number operand | `"type error: unary '-' requires a number"` |
| `Expr::Index` list | negative or fractional index | `"invalid index: must be a non-negative integer"` |
| `Expr::Index` list | out of bounds | `"index out of bounds: {n}"` |
| `Expr::Index` dict | key not found | `Nil` (intentional — optional field pattern) |
| `Expr::Index` catch-all | wrong types | `"type error: cannot index {type} with {type}"` |
| `Expr::Each` catch-all | non-function passed | `"each requires a function"` |
| `Expr::Pipe` catch-all | non-call on right | `"pipe right-hand side must be a function call"` |

### 5. execute() / eval() — parse error surfacing

```rust
pub fn execute(src: &str) -> Value {
    match parse_src(src) {
        Ok(ast) => {
            let mut env = Environment::new();
            eval_expr(&mut env, &ast)
        }
        Err(e) => Value::Error(format!("parse error: {}", e)),
    }
}

pub fn eval(env: &Environment, src: &str) -> Value {
    match parse_src(src) {
        Ok(ast) => {
            let mut local = env.clone();
            eval_expr(&mut local, &ast)
        }
        Err(e) => Value::Error(format!("parse error: {}", e)),
    }
}
```

### 6. main.rs — error display and exit code

Script mode (`--run`):
```rust
let result = interpreter::execute(&src);
if let Value::Error(msg) = &result {
    eprintln!("Runtime Error: {}", msg);
    std::process::exit(1);
}
```

REPL mode: already prints non-Nil values; `Value::Error` will display as `<error: ...>` via the existing `Display` impl. No change needed — interactive use benefits from seeing the error value inline.

---

## Phase 2: try/catch

### Syntax

```
try
  x = risky_call()
  print(x)
catch err
  print(err.message)
end
```

### AST node

```rust
Expr::TryCatch {
    body: Box<Expr>,
    catch_var: String,
    handler: Box<Expr>,
}
```

### Evaluation

```rust
Expr::TryCatch { body, catch_var, handler } => {
    let result = eval_expr(env, body);
    if let Value::Error(_) = &result {
        env.define(catch_var.clone(), result);
        eval_expr(env, handler)
    } else {
        result
    }
}
```

If the body produces a `Value::Error`, propagation stops, the error is bound to `catch_var`, and the handler runs. If the body succeeds, its value is returned unchanged.

---

## Implementation Order

1. Lexer: add `Token::Dot`
2. Parser: add `Expr::FieldAccess`, parse postfix `.ident`
3. Interpreter: evaluate `FieldAccess`
4. Interpreter: convert all silent `Value::Nil` failure paths to `Value::Error`
5. `execute()` / `eval()`: surface parse errors
6. `main.rs`: print error to stderr, exit 1
7. Tests: cover each new error case
8. *(Phase 2)* Lexer/parser/interpreter: try/catch

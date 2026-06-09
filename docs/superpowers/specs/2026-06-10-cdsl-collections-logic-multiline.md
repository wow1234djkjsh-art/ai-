---
name: cdsl-collections-logic-multiline
description: Design spec for adding list/dict types, logical operators, and line continuation to C-DSL.
metadata:
  type: reference
---

# C-DSL Extension: Collections, Logical Operators, Line Continuation

## Overview

This spec extends Compact-DSL with three new capabilities:

1. **List and Dictionary types** with bracket-based indexing
2. **Logical operators** (`and`, `or`, `not`)
3. **Line continuation** via trailing `\`

All changes follow existing C-DSL conventions: failures return `Nil`, no new `Bool` type, results of logical ops are `Number(1.0)` / `Number(0.0)`.

---

## 1. Syntax

### List Literals & Indexing

```
lst = [1, 2, 3]
lst[0]          # → 1
lst[1+1]        # → 3  (expressions allowed as index)
lst[99]         # → Nil (out of bounds)
```

### Dictionary Literals & Lookup

```
d = {name:"Alice", age:30}
d["name"]       # → "Alice"
d["age"]        # → 30
d["missing"]    # → Nil
```

The `:` inside `{...}` is a key-value separator. Outside `{}`, `:` retains its existing roles (conditional branches and `each` body delimiter). No lexer state change required — the parser distinguishes context structurally.

Keys must be string literals or bare identifiers. Values are parsed at `parse_pipe` level (full expressions). The closing `}` or `,` terminates each value naturally since neither is consumed by any expression parser.

### Logical Operators

```
x = 5
x>3 and x<10   # → 1
x<0 or x>3     # → 1
not x>10       # → 1
```

Logical results follow existing truthiness rules:
- `Number(n)` where `n != 0.0` → truthy
- `String(s)` where `s` is non-empty → truthy
- `Nil` → falsy
- `Function` → truthy

Short-circuit evaluation: `and` stops at first falsy, `or` stops at first truthy.

### Line Continuation

```
result = add 1,2 \
         |double

fn big x =>     \
  ?x>0:x*2:0
```

A `\` immediately followed by `\n` is consumed by the lexer — the newline is not emitted as a `Sep` token. Any other use of `\` is ignored.

---

## 2. Operator Precedence

Lowest to highest:

| Level | Operators |
|-------|-----------|
| pipe  | `\|`      |
| or    | `or`      |
| and   | `and`     |
| not   | `not` (unary) |
| cmp   | `>`, `<`  |
| add   | `+`, `-`  |
| mul   | `*`, `/`  |
| unary | `-` (negate) |
| primary + subscript | literals, calls, `[idx]` |

Subscript `[idx]` is left-associative postfix: `lst[0][1]` is `(lst[0])[1]`.

---

## 3. Grammar Changes

### Parser Chain (after change)

```
parse_pipe → parse_or → parse_and → parse_not → parse_cmp
           → parse_add → parse_mul → parse_unary
           → parse_primary (atom) → subscript loop
```

### New parse functions

- `parse_or`: loops on `Token::Or`, calls `parse_and` for each operand
- `parse_and`: loops on `Token::And`, calls `parse_not` for each operand
- `parse_not`: if `Token::Not`, advance and wrap result of `parse_not` in `Expr::Not`; else delegate to `parse_cmp`
- Subscript loop (inside `parse_primary`): after parsing the atom, loop while `peek() == Sym('[')`, consuming `[expr]` and wrapping in `Expr::Index`

### New AST Nodes

```rust
List(Vec<Expr>)
Dict(Vec<(String, Expr)>)
Index { object: Box<Expr>, index: Box<Expr> }
And   { left: Box<Expr>, right: Box<Expr> }
Or    { left: Box<Expr>, right: Box<Expr> }
Not   (Box<Expr>)
```

---

## 4. Lexer Changes

### New tokens

```rust
Token::LBracket   // [
Token::RBracket   // ]
Token::LBrace     // {
Token::RBrace     // }
Token::And        // keyword: and
Token::Or         // keyword: or
Token::Not        // keyword: not
```

`[`, `]`, `{`, `}` are added to the character dispatch. `and`, `or`, `not` are added to the keyword match in the identifier branch.

### Line continuation

When `\` is encountered and the next character is `\n`, both are consumed without emitting a token. Otherwise `\` is silently skipped.

---

## 5. Interpreter Changes

### New Value variants

```rust
Value::List(Vec<Value>)
Value::Dict(Vec<(String, Value)>)  // Vec preserves insertion order
```

### eval_expr additions

```rust
Expr::List(items) => {
    Value::List(items.iter().map(|e| eval_expr(env, e)).collect())
}

Expr::Dict(pairs) => {
    Value::Dict(pairs.iter()
        .map(|(k, v)| (k.clone(), eval_expr(env, v)))
        .collect())
}

Expr::Index { object, index } => {
    match (eval_expr(env, object), eval_expr(env, index)) {
        (Value::List(items), Value::Number(n)) =>
            items.get(n as usize).cloned().unwrap_or(Value::Nil),
        (Value::Dict(pairs), Value::String(key)) =>
            pairs.iter().find(|(k,_)| k == &key)
                 .map(|(_, v)| v.clone()).unwrap_or(Value::Nil),
        _ => Value::Nil,
    }
}

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

`is_truthy` extracts the existing truthiness check from `eval_expr` for the `If` branch into a shared helper.

---

## 6. Error Handling

| Situation | Result |
|-----------|--------|
| `lst[99]` out of bounds | `Nil` |
| `dict["missing"]` | `Nil` |
| `list["key"]` type mismatch | `Nil` |
| `dict[0]` type mismatch | `Nil` |
| `not "str"` | `Number(1.0)` (non-empty string is truthy) |

---

## 7. Display

```rust
Value::List(items) => {
    let inner: Vec<String> = items.iter().map(|v| v.to_string()).collect();
    write!(f, "[{}]", inner.join(", "))
}
Value::Dict(pairs) => {
    let inner: Vec<String> = pairs.iter()
        .map(|(k, v)| format!("{}:{}", k, v)).collect();
    write!(f, "{{{}}}", inner.join(", "))
}
```

---

## 8. Testing Plan

### tests/parser.rs additions

| Test | Input | Expected AST |
|------|-------|--------------|
| `test_parse_list_literal` | `[1,2,3]` | `List([1,2,3])` |
| `test_parse_dict_literal` | `{a:"x"}` | `Dict([("a", Str("x"))])` |
| `test_parse_index_list` | `lst[0]` | `Index { lst, 0 }` |
| `test_parse_index_dict` | `d["k"]` | `Index { d, "k" }` |
| `test_parse_logical_and` | `a and b` | `And { a, b }` |
| `test_parse_logical_or` | `a or b` | `Or { a, b }` |
| `test_parse_not` | `not a` | `Not(a)` |
| `test_parse_line_continuation` | `1+\\\n2` | `BinOp(+, 1, 2)` |

### tests/interpreter.rs additions

| Test | Input | Expected |
|------|-------|----------|
| `test_list_indexing` | `[10,20,30][1]` | `Number(20)` |
| `test_dict_lookup` | `{x:42}["x"]` | `Number(42)` |
| `test_index_out_of_bounds` | `[1,2][9]` | `Nil` |
| `test_dict_missing_key` | `{a:1}["b"]` | `Nil` |
| `test_logical_and_true` | `1>0 and 2>0` | `Number(1)` |
| `test_logical_and_false` | `1>0 and 0>1` | `Number(0)` |
| `test_logical_or_true` | `0>1 or 1>0` | `Number(1)` |
| `test_logical_not_true` | `not 0>1` | `Number(1)` |
| `test_short_circuit_and` | LHS falsy → RHS not evaluated | `Number(0)` |
| `test_line_continuation` | multi-line via `\` | correct result |

---

## 9. Files Changed

| File | Change |
|------|--------|
| `src/lexer.rs` | Add `[`, `]`, `{`, `}` tokens; `and`/`or`/`not` keywords; `\` continuation |
| `src/parser.rs` | Add `List`, `Dict`, `Index`, `And`, `Or`, `Not` AST variants; new parse layers |
| `src/interpreter.rs` | Eval new nodes; extract `is_truthy`; update `Display` for new Value variants |
| `tests/parser.rs` | 8 new tests |
| `tests/interpreter.rs` | 10 new tests |

No changes to `src/builtins.rs`, `src/cache.rs`, `src/main.rs`, or `src/lib.rs`.

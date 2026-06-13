# C-DSL Language Upgrade Plan — 다른 언어 수준으로 업그레이드

**Goal:** Make C-DSL feature-comparable to mainstream scripting languages (Python, JS, Ruby).

**Repo:** `C:\Users\PC\Desktop\클로드\c-dsl`  
**Build:** `cargo build --release`, tests: `cargo test`

## Already Done (partial, code is currently broken)

- `lexer.rs`: Added `Token::Pow` (`**`), `Token::Return`, `Token::Break`, `Token::Continue`; `%` in Sym list; `**` two-char handling; `return`/`break`/`continue` keywords
- `parser.rs`: Added `Expr::Return(Option<Box<Expr>>)`, `Expr::Break`, `Expr::Continue` to enum; `parse_return()` method; `parse_power()` (right-associative `**`); `%` in `parse_mul`; `break`/`continue`/`return` in `parse_stmt`

**Current compile error:** `interpreter.rs` doesn't handle the 3 new `Expr` variants.

---

## Task 1 — Fix interpreter.rs: new Expr variants + signal system + operators + globals

**Files:** `src/interpreter.rs`

### 1a. Add 3 new Value variants

```rust
pub enum Value {
    // existing: Number, String, Function, Nil, List, Dict, Error
    Return(Box<Value>),
    Break,
    Continue,
}
```

Update everywhere Value is matched exhaustively:
- `impl PartialEq`: add `(Return(a),Return(b))=>a==b`, `(Break,Break)=>true`, `(Continue,Continue)=>true`
- `impl Debug`: add arms
- `impl Display`: `Return(v)=>write!(f,"{}",v)`, `Break=>write!(f,"<break>")`, `Continue=>write!(f,"<continue>")`
- `is_truthy`: `Value::Return(_)|Value::Break|Value::Continue => false`
- `type_name`: add arms returning `"return"`, `"break"`, `"continue"`
- `json_to_value`/`value_to_json` in builtins.rs: treat Break/Continue as Null

### 1b. Add `is_signal` helper

```rust
#[inline]
fn is_signal(v: &Value) -> bool {
    matches!(v, Value::Error(_) | Value::Return(_) | Value::Break | Value::Continue)
}
```

Replace ALL existing `if let Value::Error(_) = &x { return x; }` patterns with `if is_signal(&x) { return x; }` **except** in `TryCatch` body evaluation.

### 1c. Handle new Expr variants in eval_expr

```rust
Expr::Return(maybe_val) => {
    let val = match maybe_val {
        Some(e) => { let v = eval_expr(env, e); if is_signal(&v) { return v; } v }
        None => Value::Nil,
    };
    Value::Return(Box::new(val))
}
Expr::Break => Value::Break,
Expr::Continue => Value::Continue,
```

### 1d. Update While to catch Break/Continue

```rust
Expr::While { cond, body } => {
    let mut last = Value::Nil;
    loop {
        let cond_val = eval_expr(env, cond);
        if is_signal(&cond_val) { return cond_val; }
        if !is_truthy(&cond_val) { break; }
        last = eval_expr(env, body);
        match last {
            Value::Break    => { last = Value::Nil; break; }
            Value::Continue => { last = Value::Nil; continue; }
            ref v if is_signal(v) => return last,
            _ => {}
        }
    }
    last
}
```

### 1e. Update Function::call to catch Return

```rust
match eval_expr(&mut env, &self.body) {
    Value::Return(v)  => *v,
    Value::Break      => Value::Error("break outside of loop".into()),
    Value::Continue   => Value::Error("continue outside of loop".into()),
    other => other,
}
```

### 1f. Update TryCatch — only catch Error, propagate signals

```rust
Expr::TryCatch { body, catch_var, handler } => {
    let result = eval_expr(env, body);
    if let Value::Error(_) = &result {
        env.define(catch_var.clone(), result);
        eval_expr(env, handler)
    } else {
        result  // Return/Break/Continue propagate unchanged
    }
}
```

### 1g. Update Assign — propagate signals, keep soft Error behavior

```rust
Expr::Assign { name, value } => {
    let val = eval_expr(env, value);
    if matches!(&val, Value::Return(_) | Value::Break | Value::Continue) { return val; }
    env.define(name.clone(), val.clone());
    match val {
        Value::Error(_) => Value::Nil,
        other => other,
    }
}
```

### 1h. Add `**` and `%` to eval_binop

```rust
("**", Value::Number(l), Value::Number(r)) => Value::Number(l.powf(*r)),
("%",  Value::Number(l), Value::Number(r)) => {
    if *r == 0.0 { Value::Error("modulo by zero".into()) }
    else { Value::Number(l % r) }
}
```

### 1i. Negative indexing in Index handler

```rust
(Value::List(items), Value::Number(n)) => {
    let len = items.len() as f64;
    let idx = if *n < 0.0 { len + *n } else { *n };
    if idx < 0.0 || idx.fract() != 0.0 {
        return Value::Error(format!("invalid list index: {}", n));
    }
    items.get(idx as usize).cloned()
        .unwrap_or_else(|| Value::Error(format!("index out of bounds: {}", n as i64)))
}
```

### 1j. Expand setup_globals

```rust
fn setup_globals(env: &mut Environment) {
    env.define("true".into(),  Value::Number(1.0));
    env.define("false".into(), Value::Number(0.0));
    env.define("nil".into(),   Value::Nil);
    env.define("pi".into(),    Value::Number(std::f64::consts::PI));
    env.define("e".into(),     Value::Number(std::f64::consts::E));
    env.define("inf".into(),   Value::Number(f64::INFINITY));
    env.define("nan".into(),   Value::Number(f64::NAN));
}
```

### 1k. Handle Return/Break/Continue in parse_primary (parser.rs)

Add to `parse_primary` so they work inside expression positions (e.g., function bodies):

```rust
Token::Return => {
    self.advance();
    if matches!(self.peek(), Token::Sep | Token::Eof | Token::End | Token::Catch | Token::Sym(':')) {
        Ok(Expr::Return(None))
    } else {
        let e = self.parse_expr()?;
        Ok(Expr::Return(Some(Box::new(e))))
    }
}
Token::Break    => { self.advance(); Ok(Expr::Break) }
Token::Continue => { self.advance(); Ok(Expr::Continue) }
```

### Tests to add (in `interpreter.rs` tests module)

```rust
#[test] fn modulo_basic() { assert_eq!(run("10 % 3"), Value::Number(1.0)); }
#[test] fn modulo_zero()  { assert!(matches!(run("5 % 0"), Value::Error(_))); }
#[test] fn power_basic()  { assert_eq!(run("2 ** 10"), Value::Number(1024.0)); }
#[test] fn power_right_assoc() { assert_eq!(run("2 ** 3 ** 2"), Value::Number(512.0)); }
#[test] fn neg_index()    { assert_eq!(run("lst=[1,2,3]; lst[-1]"), Value::Number(3.0)); }
#[test] fn nil_global()   { assert_eq!(run("nil"), Value::Nil); }
#[test] fn pi_global()    { assert!(matches!(run("pi"), Value::Number(_))); }
#[test] fn break_while()  {
    assert_eq!(run("i=0; while i<10\ni=i+1\nif i==3: break\nend; i"), Value::Number(3.0));
    // simplified: actually test break stops the loop
    assert_eq!(run("i=0\nwhile i<5\n  i=i+1\n  ?i==3:break:nil\nend\ni"), Value::Number(3.0));
}
#[test] fn continue_while() {
    // continue skips rest of body, loop continues
    assert_eq!(run("s=0\ni=0\nwhile i<5\n  i=i+1\n  ?i==3:continue:nil\n  s=s+i\nend\ns"), Value::Number(12.0));
}
#[test] fn return_from_fn() {
    assert_eq!(run("fn f x => ?x>0: return x*2 : 0\nf 5"), Value::Number(10.0));
}
```

**Acceptance criteria:** `cargo test` passes all existing tests + new tests above. No compile errors.

---

## Task 2 — New builtins: math, string, list, dict, other

**Files:** `src/builtins.rs`, `src/interpreter.rs` (call_fn dispatch table only)

### Math builtins (add to builtins.rs)

```rust
pub fn builtin_sqrt(args)  -> sqrt(x)
pub fn builtin_pow(args)   -> x.powf(y)        // also usable as function
pub fn builtin_log(args)   -> x.ln() or log(x,base) if 2 args
pub fn builtin_log2(args)  -> x.log2()
pub fn builtin_log10(args) -> x.log10()
pub fn builtin_exp(args)   -> x.exp()
pub fn builtin_sin(args)   -> x.sin()
pub fn builtin_cos(args)   -> x.cos()
pub fn builtin_tan(args)   -> x.tan()
pub fn builtin_asin(args)  -> x.asin()
pub fn builtin_acos(args)  -> x.acos()
pub fn builtin_atan(args)  -> x.atan()
pub fn builtin_atan2(args) -> y.atan2(x)   // 2 args: atan2(y, x)
pub fn builtin_hypot(args) -> x.hypot(y)
pub fn builtin_clamp(args) -> x.clamp(lo, hi)  // 3 args
pub fn builtin_sign(args)  -> signum: -1/0/1
pub fn builtin_random(args) -> random f64 in [0,1) using thread-local xorshift64 PRNG
pub fn builtin_rand_int(args) -> rand_int(n): [0,n) or rand_int(lo,hi): [lo,hi)
```

PRNG implementation (no external crate needed):
```rust
use std::cell::Cell;
thread_local! {
    static RNG: Cell<u64> = Cell::new(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(6364136223846793005)
    );
}
fn xorshift64() -> u64 {
    RNG.with(|s| { let mut x = s.get(); x^=x<<13; x^=x>>7; x^=x<<17; s.set(x); x })
}
```

### String builtins

```rust
pub fn builtin_replace(args)      -> replace(s, old, new): s.replace(old, new)
pub fn builtin_starts_with(args)  -> 1/0
pub fn builtin_ends_with(args)    -> 1/0
pub fn builtin_index_of(args)     -> index_of(s_or_list, val): index (0-based) or -1
pub fn builtin_repeat(args)       -> repeat(s, n): s.repeat(n as usize)
pub fn builtin_char_at(args)      -> char_at(s, i): single char string at index i (supports negative)
pub fn builtin_chars(args)        -> chars(s): list of single-char strings
pub fn builtin_format(args)       -> format(template, ...): "{}" replaced with args in order
```

`index_of` works for both strings (substring search) and lists (element search).

### List builtins

```rust
pub fn builtin_reverse(args)    -> reverse list
pub fn builtin_unique(args)     -> remove duplicates (preserve order, keep first)
pub fn builtin_zip(args)        -> zip(a, b): [[a0,b0],[a1,b1],...] (stops at shorter)
pub fn builtin_enumerate(args)  -> enumerate(lst): [[0,v0],[1,v1],...]
pub fn builtin_any(args)        -> any(lst, fn): 1 if any item passes predicate
pub fn builtin_all(args)        -> all(lst, fn): 1 if all items pass predicate
pub fn builtin_sum(args)        -> sum(lst): sum of numbers
pub fn builtin_find_where(args) -> find_where(lst, fn): first item passing predicate or nil
pub fn builtin_flat_map(args)   -> flat_map(lst, fn): map then flatten one level
pub fn builtin_take(args)       -> take(lst, n): first n items
pub fn builtin_skip(args)       -> skip(lst, n): drop first n items
pub fn builtin_count(args)      -> count(lst, fn): count items passing predicate
pub fn builtin_product(args)    -> product(lst): product of numbers
```

`index_of` for lists: find index of first matching element (value equality), return -1 if not found.

### Dict builtins

```rust
pub fn builtin_get(args)   -> get(dict, key, default): lookup with fallback
pub fn builtin_del(args)   -> del(dict, key): returns new dict without that key
pub fn builtin_merge(args) -> merge(d1, d2): d2 keys override d1
pub fn builtin_has(args)   -> has(dict, key): 1/0 (alias: same as contains for dicts)
```

### Other builtins

```rust
pub fn builtin_make_error(args) -> error(msg): creates Value::Error(msg)
pub fn builtin_is_error(args)   -> is_error(v): 1 if Error, else 0
pub fn builtin_ok(args)         -> ok(v, default): returns v unless Error, then default
```

### call_fn dispatch additions (in interpreter.rs)

Add to the match in `call_fn`:
```
"sqrt" => ..., "pow" => ..., "log" => ..., "log2" => ..., "log10" => ...,
"exp" => ..., "sin" => ..., "cos" => ..., "tan" => ...,
"asin" => ..., "acos" => ..., "atan" => ..., "atan2" => ...,
"hypot" => ..., "clamp" => ..., "sign" => ...,
"random" => ..., "rand_int" => ...,
"replace" => ..., "starts_with" => ..., "ends_with" => ...,
"index_of" => ..., "repeat" => ..., "char_at" => ..., "chars" => ..., "format" => ...,
"reverse" => ..., "unique" => ..., "zip" => ..., "enumerate" => ...,
"any" => ..., "all" => ..., "sum" => ..., "find_where" => ...,
"flat_map" => ..., "take" => ..., "skip" => ..., "count" => ..., "product" => ...,
"get" => ..., "del" => ..., "merge" => ..., "has" => ...,
"error" => ..., "is_error" => ..., "ok" => ...,
```

Also update `value_to_json` and `json_to_value` in builtins.rs to handle `Value::Return/Break/Continue` (treat as Null).

### Tests (add to builtins.rs or a new tests/builtins.rs integration test file)

```rust
// Math
assert_eq!(run("sqrt 4"),   Value::Number(2.0));
assert_eq!(run("pow 2,10"), Value::Number(1024.0));
assert_eq!(run("floor(log10(100))"), Value::Number(2.0));
assert!(matches!(run("sin pi"), Value::Number(_)));

// String
assert_eq!(run(r#"replace "hello world" "world" "Rust""#), Value::String("hello Rust".into()));
assert_eq!(run(r#"starts_with "hello" "he""#), Value::Number(1.0));
assert_eq!(run(r#"index_of "hello" "ll""#), Value::Number(2.0));
assert_eq!(run(r#"repeat "ab" 3"#), Value::String("ababab".into()));
assert_eq!(run(r#"len chars "hello""#), Value::Number(5.0));

// List
assert_eq!(run("reverse [1,2,3]"), Value::List(vec![Value::Number(3.0),Value::Number(2.0),Value::Number(1.0)]));
assert_eq!(run("unique [1,2,1,3,2]"), Value::List(vec![Value::Number(1.0),Value::Number(2.0),Value::Number(3.0)]));
assert_eq!(run("sum [1,2,3,4,5]"), Value::Number(15.0));
assert_eq!(run("any [1,2,3] fn x=>x>2"), Value::Number(1.0));
assert_eq!(run("all [1,2,3] fn x=>x>0"), Value::Number(1.0));
assert_eq!(run("take [1,2,3,4,5] 3"), Value::List(vec![Value::Number(1.0),Value::Number(2.0),Value::Number(3.0)]));

// Dict
assert_eq!(run(r#"get {"a":1} "b" 99"#), Value::Number(99.0));
assert_eq!(run(r#"has {"a":1} "a""#), Value::Number(1.0));
assert_eq!(run(r#"keys del {"a":1,"b":2} "a""#), Value::List(vec![Value::String("b".into())]));

// Other
assert!(matches!(run("error \"oops\""), Value::Error(_)));
assert_eq!(run("is_error error(\"x\")"), Value::Number(1.0));
assert_eq!(run(r#"ok error("x") 42"#), Value::Number(42.0));
```

**Acceptance criteria:** `cargo test` passes all existing + new tests. All builtins documented in `docs/syntax.md`.

---

## Task 3 — Update syntax.md

**File:** `docs/syntax.md`

Update to document all newly added features:
- `%` and `**` operators
- `return`, `break`, `continue` statements
- `nil`, `pi`, `e`, `inf`, `nan` globals
- Negative indexing `lst[-1]`
- All new builtins from Task 2 (in appropriate tables)

Keep the file clear and accurate. Reference actual implementation.

**Acceptance criteria:** syntax.md accurately documents every language feature that exists in the interpreter.

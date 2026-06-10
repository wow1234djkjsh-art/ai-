# Compact-DSL (C-DSL)

A minimal, token-efficient scripting language designed for AI integration.

## Quick Start

### 1. Install Rust

If you don't have Rust installed, get it from [rustup.rs](https://rustup.rs) and follow the installer.

### 2. Clone and build

```bash
git clone https://github.com/wow1234djkjsh-art/ai-.git
cd ai-
cargo build --release
```

First build takes 1–2 minutes while dependencies download.

### 3. Run

Launch the interactive REPL:

```bash
# Linux / macOS
./target/release/c-dsl

# Windows
.\target\release\c-dsl.exe
```

Run a script file:

```bash
# Linux / macOS
./target/release/c-dsl --run script.cdsl

# Windows
.\target\release\c-dsl.exe --run script.cdsl
```

---

## Language Reference

### Variables

Assign with `=`. Assignments persist across lines in the REPL and in multi-statement scripts.

```
x = 5
y = x * 2
```

### Arithmetic

Standard operators: `+`, `-`, `*`, `/`, `>`, `<`. String concatenation uses `+`.

```
1 + 2
10 - 3
4 * 5
20 / 4
"hello" + " world"
```

### Logical Operators

`and`, `or`, `not` — short-circuit evaluation. Return `1` (true) or `0` (false).

```
1 > 0 and 2 > 0    // 1
0 > 1 or 2 > 0     // 1
not 0               // 1
```

### Functions

Define with `fn <name> <params> => <body>`. Parameters are comma-separated.

```
fn add a, b => a + b
fn square x => x * x
```

Anonymous (lambda) functions:

```
fn x => x * 2
```

### Function Calls

Call with parentheses or with a space before arguments.

```
add(1, 2)
add 1, 2
square 7
```

### Conditionals (ternary)

`? <cond> : <then> : <else>`

```
x = 5
? x > 0 : x : 0
? x > 3 : x * 2 : 0
```

### Pipe Operator

`<expr> | <fn>` — passes the left value as the first argument to the right function.

```
fn double x => x * 2
3 | double
3 | double | double
add 1, 2 | double
```

### Each Loop

`each <item1>, <item2>, ... : <fn>` — applies a function to every item, returns the last result.

```
each 1, 2, 3 : fn x => x * 2
fn triple x => x * 3
each 10, 20, 30 : triple
```

### Lists

Ordered collections. Index with `[n]` (zero-based).

```
lst = [1, 2, 3]
lst[0]            // 1
lst[2]            // 3
```

Out-of-bounds access returns a runtime error.

### Dicts

Key-value collections. Keys are strings (bare or quoted). Access with `["key"]` or dot notation.

```
user = {name: "alice", age: 30}
user["name"]      // "alice"
user.name         // "alice"  (dot notation)
user.age          // 30
```

Missing keys return `nil`.

### Dot Notation (Field Access)

`.field` works on both dicts and error values.

```
d = {x: 10, y: 20}
d.x               // 10

e = unknown_fn()
e.type            // "error"
e.message         // "unknown function: unknown_fn"
```

### Recursive Functions

Functions can call themselves by name.

```
fn fact n => ? n > 0 : n * fact n-1 : 1
fact 5            // 120
```

### Multi-statement Scripts

Statements are separated by newlines or `;`.

```
fn add a, b => a + b
fn double x => x * 2
result = add 3, 4 | double
result            // 14
```

---

## Error Handling

### Errors as Values

Runtime errors are first-class values. A failed call returns an error value instead of crashing.

```
e = unknown_fn()
e.type            // "error"
e.message         // "unknown function: unknown_fn"
```

Check and branch on errors:

```
result = risky_fn()
? result.type == "error" : result.message : result
```

### try/catch/end

Catch errors from standalone expressions (expressions not assigned to a variable).

```
try
  unknown_fn()
catch err
  print(err.message)
end
```

The catch block runs only when the try body produces an error. If no error, the try body's result is returned.

```
try
  42
catch err
  0
end
// → 42
```

**Note:** `x = bad_fn()` stores the error in `x` and continues execution — it does NOT trigger catch. Only standalone expressions that propagate errors trigger catch.

### Error Messages

| Situation | Message |
|-----------|---------|
| Undefined variable | `undefined variable: foo` |
| Unknown function | `unknown function: pritn` |
| Wrong argument count | `arity mismatch: fn expects 2 args, got 1` |
| Type error | `type error: '+' not supported for these types` |
| Division by zero | `division by zero` |
| Index out of bounds | `index out of bounds: 5` |
| Invalid index | `invalid index: must be a non-negative integer` |
| Parse error | `parse error: ...` |

In script mode (`--run`), an unhandled error prints `Runtime Error: <message>` to stderr and exits with code 1.

---

## Built-ins

### `print`

Print a value to stdout and return it unchanged.

```
print 42
print "hello"
add 1, 2 | print
```

### `eval`

Evaluate a C-DSL expression string in the current scope.

```
eval "2 + 3"
x = 10
eval "x * 3"     // 30
```

### `model`

Call a language model with optional response caching.

```
model "model-id" "prompt"
model "model-id" "prompt" "code"
model "model-id" "prompt" "code" "true"
```

| Position | Value | Description |
|----------|-------|-------------|
| 1 | `"model-id"` | Model identifier |
| 2 | `"prompt"` | Input prompt |
| 3 (optional) | `"code"` | Return a C-DSL expression instead of plain text |
| 4 (optional) | `"true"` | Bypass cache (force recompute) |

Responses are cached to `~/.c-dsl/cache/` by a SHA-256 key derived from model ID, prompt, and format.

Combine with `eval`:

```
result = eval(model "codegen-model" "write an expression" "code")
```

---

## Running Tests

```bash
cargo test
```

# Compact-DSL (C-DSL)

A minimal, token-efficient scripting language designed for AI integration.

## Quick Start

```bash
cargo build --release
```

Development (no build step needed):

```bash
cargo run -- --run hello.cdsl
```

Run a script file:

```bash
# Linux / macOS
./target/release/c-dsl --run script.cdsl

# Windows
target\release\c-dsl.exe --run script.cdsl
```

Launch the interactive REPL (no arguments):

```bash
./target/release/c-dsl
```

## Language Reference

### Variables

Assign with `=`. Assignments persist across lines in the REPL and in multi-statement scripts.

```
x = 5
y = x * 2
```

### Arithmetic

Standard operators: `+`, `-`, `*`, `/`, `>`, `<`.  String concatenation uses `+`.

```
1 + 2
10 - 3
4 * 5
20 / 4
"hello" + " world"
```

### Functions

Define with `fn <name> <params> => <body>`.  Parameters are comma-separated.

```
fn add a,b => a+b
fn square x => x*x
```

### Function Calls

Call with or without parentheses.  Arguments are comma-separated.

```
add 1,2
add(1,2)
square 7
```

### Conditionals (ternary)

`?<cond>:<then>:<else>`

```
x = 5
?x>0:x:0
?x>3:x*2:0
```

### Pipe Operator

`<expr> | <fn>` — passes the left value as the first argument to the right function.

```
fn double x => x*2
3 | double
3 | double | double
add 1,2 | double
```

### Each Loop

`each <item1>,<item2>,...:<fn>` — applies the function to every item and returns the last result.

```
each 1,2,3 : fn x => x*2
fn triple x => x*3
each 10,20,30 : triple
```

### Recursive Functions

Functions can call themselves by name.

```
fn fact n => ?n>0:n*fact n-1:1
fact 5
```

### Multi-statement Scripts

Statements are separated by newlines or `;`.

```
fn add a,b => a+b
fn double x => x*2
result = add 3,4 | double
result
```

## Built-ins

### `print`

Print a value to stdout and return it unchanged.

```
print 42
print "hello"
add 1,2 | print
```

### `eval`

Evaluate a C-DSL expression string in the current scope.

```
eval "2 + 3"
eval "42"
x = 10
eval "x * 3"
```

Returns `Nil` if the argument is not a string.

### `model`

Call a language model with optional response caching.

```
model "model-id" "prompt"
model "model-id" "prompt" "code"
model "model-id" "prompt" "code" "true"
```

Arguments:

| Position | Value | Description |
|----------|-------|-------------|
| 1 | `"model-id"` | Identifier for the model |
| 2 | `"prompt"` | Input prompt |
| 3 (optional) | `"code"` | Return a C-DSL expression instead of plain text |
| 4 (optional) | `"true"` | Bypass the response cache (force recompute) |

Responses are cached to `~/.c-dsl/cache/` using a SHA-256 key derived from the model ID, prompt, and format.

### Combining `model` and `eval`

```
result = eval(model "codegen-model" "write an expression" "code")
```

## Running Tests

```bash
cargo test
```

All 51 tests cover the lexer, parser, interpreter, builtins, caching, and end-to-end pipelines.

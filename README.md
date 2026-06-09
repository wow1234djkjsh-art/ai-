# Compact-DSL (C-DSL)

A minimal, token-efficient scripting language designed for AI integration.

## Quick Start

```bash
cargo build --release
./target/release/c-dsl --run script.cdsl
```

## Language Basics

Expressions are evaluated directly. Variables and functions coming in a future release.

```
42
"hello"
1 + 2
10 - 3
4 * 5
20 / 4
```

## Built-in Primitives

### `model`

Call a language model with optional caching:

```
model "model-id" "prompt"
model "model-id" "prompt" "code"
model "model-id" "prompt" "code" "true"
```

Arguments:
- `model-id` — identifier for the model (string)
- `prompt` — input prompt (string)
- `format` (optional) — `"code"` returns a C-DSL expression; default returns text
- `force` (optional) — `"true"` bypasses the response cache

Results are cached to `~/.c-dsl/cache/` by default.

### `eval`

Evaluate a C-DSL expression string in the current scope:

```
eval "2 + 3"
eval "42"
```

Returns the result of the expression, or `Nil` if the input is not a string.

## Combining `model` and `eval`

```
# (future syntax) — run AI-generated code at runtime
result = eval(model "codegen-model" "write an expression" "code")
```

## Running Tests

```bash
cargo test
```

## Status

v0.1.0 — core interpreter, model primitive with caching, eval primitive.
Lexer and parser are implemented and will be integrated in a future release.

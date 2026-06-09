# C-DSL Full Feature Design

**Date:** 2026-06-10  
**Status:** Approved  
**Scope:** Language features only — `model` stays as stub, real API integration is out of scope.

## Purpose

C-DSL is an AI-native scripting language designed to minimize token consumption in AI-to-AI communication. It is not intended to be human-readable. The primary design principle is: express general logic and AI orchestration in as few BPE tokens as possible.

Short English keywords (`fn`, `if`, `do`) are kept because they are typically 1 BPE token each, which is equivalent to or better than single symbols. Whitespace is optional. Statements are separated by `;` or newlines.

## Syntax

### Literals
```
42
3.14
"hello"
```

### Variable Assignment
```
x=5
y="hello";z=x+3
```

### Function Definition
Parameters are comma-separated. No spaces required.
```
fn add a,b=>a+b
fn greet name=>model"gpt","say hi to "+name
```

### Function Call
Both forms are accepted.
```
add 1,2
add(1,2)
```

### Conditionals
`?cond:then:else` — all three parts required.
```
?x>0:x:-x
?x>10:"big":"small"
```

### Pipe
The result of the left expression is passed as the first argument to the right.
```
add 1,2|print
model"gpt","prompt"|eval
3|double|double
```

### Each (iteration)
Apply a function to each item in a list.
```
each 1,2,3:fn x=>x*2
```

### Statement Separator
`;` or newline.
```
x=3;y=4;add x,y|print
```

### Built-in Functions
```
print x                                    # output a value
eval"2+3"                                  # evaluate a C-DSL string
model"model-id","prompt"                   # call AI model (stub)
model"model-id","prompt","code"            # request C-DSL expression back
model"model-id","prompt","code","true"     # bypass cache
```

### Full Example
```
fn double x=>x*2
x=5;?x>3:x|double:0|print
```
Reads as: define `double`, assign `x=5`, if `x>3` pipe `x` through `double` else `0`, then print.

## Architecture

```
source string
    │
    ▼
Lexer          (src/lexer.rs)    — full rewrite, character-by-character
    │
    ▼
Parser         (src/parser.rs)   — full rewrite, recursive descent
    │
    ▼
Interpreter    (src/interpreter.rs) — connect execute() to lexer→parser→eval
    │
    ├── Builtins (src/builtins.rs)  — add print; model stays stub
    └── Cache    (src/cache.rs)     — no changes
```

## AST Node Types

```rust
enum Expr {
    Number(f64),
    Str(String),
    Ident(String),
    Assign   { name: String, value: Box<Expr> },
    BinOp    { op: char, left: Box<Expr>, right: Box<Expr> },
    FnDef    { name: String, params: Vec<String>, body: Box<Expr> },
    Call     { name: String, args: Vec<Expr> },
    If       { cond: Box<Expr>, then: Box<Expr>, else_: Box<Expr> },
    Pipe     { left: Box<Expr>, right: Box<Expr> },
    Each     { items: Vec<Expr>, func: Box<Expr> },
    Block    (Vec<Expr>),
}
```

## Component Responsibilities

### Lexer
- Character-by-character scanning (not whitespace-split)
- Recognizes: identifiers, numbers, strings (`"..."`), operators (`+`,`-`,`*`,`/`,`>`,`<`,`=`,`?`,`:`,`|`,`,`,`(`,`)`), keywords (`fn`, `each`), `;` and newline as statement separators
- Whitespace is skipped (not significant)
- Produces a flat `Vec<Token>`

### Parser
- Recursive descent
- Entry point: `parse(tokens) -> Expr` returns a `Block` of statements
- Operator precedence (low to high): pipe `|` → comparison `>`,`<` → additive `+`,`-` → multiplicative `*`,`/` → unary → primary
- `?cond:then:else` parses all three sub-expressions
- `fn name p1,p2=>body` — body is a single expression

### Interpreter
- `execute(src)` calls `lex(src)` → `parse(tokens)` → `eval_expr(env, ast)`
- `eval_expr` walks the AST recursively
- `Assign` stores into mutable `Environment`
- `FnDef` stores a `Function` value in the environment
- `Call` looks up the name, matches params to args, evaluates body in a child scope
- `Pipe` evaluates left, then calls right with left's result as first argument
- `Each` evaluates each item, calls the function for each, returns the last result
- `Block` evaluates each statement in order, returns the last value

### Builtins
| Name | Signature | Behavior |
|------|-----------|----------|
| `print` | `print value` | Prints value to stdout, returns the value |
| `eval` | `eval "expr"` | Evaluates a C-DSL string in current scope |
| `model` | `model "id","prompt"[,"code"[,"true"]]` | Stub: returns prompt or prompt.len() |

## Testing Strategy

| File | Covers |
|------|--------|
| `tests/lexer.rs` | Tokenization: no-whitespace input, all token types, edge cases |
| `tests/parser.rs` | AST construction: one test per node type |
| `tests/interpreter.rs` | Eval results: variables, functions, conditionals, pipes |
| `tests/integration.rs` | End-to-end: source string → final `Value` |

Key integration tests:
```rust
run("x=5;?x>3:x*2:0") == Value::Number(10.0)
run("fn add a,b=>a+b;add 3,4") == Value::Number(7.0)
run("fn double x=>x*2;3|double|double") == Value::Number(12.0)
run("each 1,2,3:fn x=>x*2") == Value::Number(6.0)  // last result
```

## Out of Scope

- Real LLM API integration for `model`
- String interpolation
- Lists/arrays as first-class values
- Loops other than `each`
- Error recovery (panics on invalid input are acceptable for now)

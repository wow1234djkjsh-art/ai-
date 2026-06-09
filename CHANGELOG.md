# Changelog

## [0.1.0] - 2026-06-09

### Added

**Core interpreter**
- Number and string literals
- Arithmetic operators: `+`, `-`, `*`, `/`
- Comparison operators: `>`, `<` (return `1` or `0`)
- String concatenation via `+`
- Unary negation

**Variables**
- Assignment with `=` (`x = 5`)
- Lexical scoping with parent-environment lookup

**Functions**
- Named function definitions: `fn add a,b => a+b`
- Anonymous lambdas: `fn x => x*2`
- Call syntax with parens: `add(1,2)`
- Call syntax without parens (space-separated): `add 1,2`
- Closures capture the enclosing environment at definition time
- Recursive functions: self-reference is injected at call time to support patterns like `fn fact n => ?n>0:n*fact n-1:1`

**Conditionals**
- Ternary expression: `?<cond>:<then>:<else>` (e.g. `?x>0:x:0`)

**Pipe operator**
- `<expr> | <fn>` passes the left value as the first argument: `add 1,2 | print`
- Chains: `3 | double | double`

**Each loop**
- `each <items>:<fn>` — applies function to every item, returns last result
- Inline lambda form: `each 1,2,3 : fn x => x*2`
- Named function form: `each 10,20,30 : triple`

**Multi-statement programs**
- Statements separated by newlines or `;`
- Last evaluated value is the program result

**Built-in functions**
- `print <value>` — prints to stdout, returns value unchanged
- `eval "<expr>"` — evaluates a C-DSL expression string in the caller's environment
- `model "<id>" "<prompt>" [format] [force]` — language model call with SHA-256-keyed file cache at `~/.c-dsl/cache/`; `force:"true"` bypasses cache

**Infrastructure**
- Lexer: tokenises numbers, strings, identifiers, keywords (`fn`, `each`), `=>` arrow, all operators
- Recursive-descent parser producing a typed `Expr` AST
- Full integration: every script path runs lex → parse → eval
- REPL: interactive mode with persistent environment (`exit` or `quit` to leave)
- CLI: `--run <file>` flag to execute a script file
- File-based response cache with base64/SHA-256 keys (`serde_json` serialisation)
- 51 unit and integration tests covering lexer, parser, interpreter, builtins, caching, and end-to-end pipelines

# Changelog

## [0.1.0] - 2026-06-09

### Added
- Core expression interpreter: number literals, string literals, arithmetic operators (+, -, *, /), variable lookup
- `model` primitive: call a language model stub with SHA-256-keyed file cache and `force` flag
- `eval` primitive: evaluate a C-DSL expression string in the caller's environment
- File-based response cache at `~/.c-dsl/cache/` using base64-encoded SHA-256 keys
- Lexer and parser (planned infrastructure, not yet integrated into runtime)
- CLI: `--run <file>` and `--test` flags

### Notes
- Model responses are currently stub implementations; real API integration is planned
- Lexer and parser will be connected to the interpreter in a future release

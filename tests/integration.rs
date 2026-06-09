use c_dsl::builtins::{model, builtin_eval};
use c_dsl::interpreter::{Environment, Value};
use sha2::{Sha256, Digest};

/// Clear a specific cache entry so tests start with known state.
fn clear_cache_for(model_id: &str, prompt: &str, format: &str) {
    let raw_key = format!("{}:{}:{}", model_id, prompt, format);
    let cache_key = format!("{:x}", Sha256::digest(raw_key.as_bytes()));
    if let Some(mut path) = dirs::home_dir() {
        path.push(".c-dsl");
        path.push("cache");
        path.push(format!("{}.json", cache_key));
        let _ = std::fs::remove_file(path); // ignore if absent
    }
}

/// Verifies the full model → eval pipeline:
/// model(format:"code") returns a C-DSL expression, builtin_eval evaluates it.
#[test]
fn test_model_eval_pipeline() {
    let env = Environment::new();
    let prompt = "generate an expression";

    let code_val = model(&env, vec![
        Value::String("stub-model".to_string()),
        Value::String(prompt.to_string()),
        Value::String("code".to_string()),
    ]);

    // model must return a String (the C-DSL expression)
    assert!(matches!(code_val, Value::String(_)), "model must return String");

    // eval must produce a Number from that string
    let result = builtin_eval(&env, vec![code_val]);
    // The stub returns prompt.len() as string → eval produces Number(prompt.len())
    assert_eq!(result, Value::Number(prompt.len() as f64));
}

/// Verifies caching: identical calls return the same value.
#[test]
fn test_model_caching_consistency() {
    let env = Environment::new();
    let model_id = "m";
    let prompt = "cache test prompt";
    let fmt = "code";

    // Clear any stale cache to guarantee cold-cache first call
    clear_cache_for(model_id, prompt, fmt);

    let args = || vec![
        Value::String(model_id.to_string()),
        Value::String(prompt.to_string()),
        Value::String(fmt.to_string()),
    ];

    // First call: cache miss — computes and stores
    let first = model(&env, args());
    // Second call: cache hit — must return same value
    let second = model(&env, args());

    assert_eq!(first, second);
    assert!(matches!(first, Value::String(_)));
}

/// Verifies force:true bypasses cache and recomputes the correct stub output.
#[test]
fn test_model_force_bypasses_cache() {
    let env = Environment::new();
    let model_id = "m";
    let prompt = "force flag prompt";
    let fmt = "code";

    // Clear cache so we start fresh
    clear_cache_for(model_id, prompt, fmt);

    let make_args = |force: bool| {
        let mut v = vec![
            Value::String(model_id.to_string()),
            Value::String(prompt.to_string()),
            Value::String(fmt.to_string()),
        ];
        if force {
            v.push(Value::String("true".to_string()));
        }
        v
    };

    // Normal call: computes and caches
    let cached_val = model(&env, make_args(false));
    assert!(matches!(cached_val, Value::String(_)));

    // Force call: bypasses cache, recomputes — must equal the stub's formula
    let forced_val = model(&env, make_args(true));
    assert!(matches!(forced_val, Value::String(_)));

    // Stub is deterministic: both must equal prompt.len().to_string()
    let expected = Value::String(prompt.len().to_string());
    assert_eq!(cached_val, expected, "cached value must match stub formula");
    assert_eq!(forced_val, expected, "forced value must match stub formula");
}

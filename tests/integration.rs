use c_dsl::builtins::{builtin_eval, model};
use c_dsl::cache::set_cached;
use c_dsl::interpreter::{execute, Environment, Value};
use sha2::{Digest, Sha256};

fn cache_key_for(model_id: &str, prompt: &str, format: &str) -> String {
    let raw = format!("{}:{}:{}", model_id, prompt, format);
    format!("{:x}", Sha256::digest(raw.as_bytes()))
}

/// model returns cached value when cache is pre-populated; eval can run on it.
#[test]
fn test_model_eval_pipeline() {
    let env = Environment::new();
    let model_id = "eval-test-model";
    let prompt = "generate expression";
    let fmt = "code";

    // Pre-populate cache with a valid C-DSL expression
    set_cached(&cache_key_for(model_id, prompt, fmt), "2 + 3");

    let code_val = model(
        &env,
        vec![
            Value::String(model_id.to_string()),
            Value::String(prompt.to_string()),
            Value::String(fmt.to_string()),
        ],
    );
    assert_eq!(
        code_val,
        Value::String("2 + 3".to_string()),
        "model must return cached expression"
    );

    let result = builtin_eval(&env, vec![code_val]);
    assert_eq!(result, Value::Number(5.0), "eval of '2 + 3' must be 5");
}

/// Identical calls return same cached value (cache hit).
#[test]
fn test_model_caching_consistency() {
    let env = Environment::new();
    let model_id = "cache-consistency-model";
    let prompt = "unique cache test XYZ789";
    let fmt = "";

    set_cached(&cache_key_for(model_id, prompt, fmt), "hello from cache");

    let args = || vec![
        Value::String(model_id.to_string()),
        Value::String(prompt.to_string()),
    ];

    let first = model(&env, args());
    let second = model(&env, args());

    assert_eq!(first, second, "cached calls must be equal");
    assert_eq!(first, Value::String("hello from cache".to_string()));
}

/// force:true bypasses cache — cached call returns cached value, force call does not.
#[test]
fn test_model_force_bypasses_cache() {
    let env = Environment::new();
    let model_id = "force-bypass-model";
    let prompt = "force flag test prompt";
    let fmt = "";

    // Pre-cache a known value
    set_cached(&cache_key_for(model_id, prompt, fmt), "cached value");

    // Non-force: hits cache
    let cached_result = model(
        &env,
        vec![
            Value::String(model_id.to_string()),
            Value::String(prompt.to_string()),
        ],
    );
    assert_eq!(
        cached_result,
        Value::String("cached value".to_string()),
        "non-force must return cached value"
    );

    // Force: bypasses cache — returns error (no valid model) or real API response
    let forced_result = model(
        &env,
        vec![
            Value::String(model_id.to_string()),
            Value::String(prompt.to_string()),
            Value::String("".to_string()),
            Value::String("true".to_string()),
        ],
    );
    // Must NOT return the pre-cached value
    assert_ne!(
        forced_result,
        Value::String("cached value".to_string()),
        "force must bypass cache"
    );
}

#[test]
fn test_e2e_variable_conditional() {
    assert_eq!(execute("x=5;?x>3:x*2:0"), Value::Number(10.0));
}

#[test]
fn test_e2e_fn_def_and_call() {
    assert_eq!(execute("fn add a,b=>a+b;add 3,4"), Value::Number(7.0));
}

#[test]
fn test_e2e_pipe_chain() {
    assert_eq!(
        execute("fn double x=>x*2;3|double|double"),
        Value::Number(12.0)
    );
}

#[test]
fn test_e2e_each() {
    assert_eq!(execute("each 1,2,3:fn x=>x*2"), Value::Number(6.0));
}

use c_dsl::builtins::{model, builtin_eval};
use c_dsl::interpreter::{Environment, Value};

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
    let args = || vec![
        Value::String("m".to_string()),
        Value::String("cache test prompt".to_string()),
        Value::String("code".to_string()),
    ];

    let first = model(&env, args());
    let second = model(&env, args());

    // Both calls must return equal Values (second from cache)
    assert_eq!(first, second);
    assert!(matches!(first, Value::String(_)));
}

/// Verifies force:true bypasses cache but returns same deterministic value.
#[test]
fn test_model_force_flag() {
    let env = Environment::new();
    let base_args = vec![
        Value::String("m".to_string()),
        Value::String("force flag prompt".to_string()),
        Value::String("code".to_string()),
    ];
    let mut force_args = base_args.clone();
    force_args.push(Value::String("true".to_string()));

    let normal = model(&env, base_args);
    let forced = model(&env, force_args);

    // Both must be String values
    assert!(matches!(normal, Value::String(_)));
    assert!(matches!(forced, Value::String(_)));
    // Stub is deterministic → force and normal return same value
    assert_eq!(normal, forced);
}

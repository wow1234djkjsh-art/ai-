// Built-in functions for the DSL interpreter

use crate::interpreter::Value;
use crate::cache::{get_cached, set_cached};

/// Call the model primitive.
/// Signature: model "<model-id>" "<prompt>" [format:"code"] [force:"true"|"false"]
pub fn model(_env: &crate::interpreter::Environment, args: Vec<Value>) -> Value {
    if args.is_empty() {
        return Value::Nil;
    }
    let model_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Value::Nil,
    };
    let prompt = if args.len() > 1 {
        match &args[1] {
            Value::String(s) => s.clone(),
            _ => return Value::Nil,
        }
    } else {
        return Value::Nil;
    };
    let format = if args.len() > 2 {
        match &args[2] {
            Value::String(s) if s == "code" => "code",
            _ => "",
        }
    } else {
        ""
    };
    let force = if args.len() > 3 {
        match &args[3] {
            Value::String(s) if s == "true" => true,
            _ => false,
        }
    } else {
        false
    };

    let cache_key = format!("{}:{}:{}:{}", model_id, prompt, format, force);
    let cached = get_cached(&cache_key);
    if cached.is_some() && !force {
        return Value::String(cached.unwrap());
    }

    let response = if format == "code" {
        format!("// Generated code for {}: {}", model_id, prompt)
    } else {
        prompt.to_string()
    };

    set_cached(&cache_key, &response);
    Value::String(response)
}

// Built-in functions for the DSL interpreter

use sha2::{Sha256, Digest};
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

    let raw_key = format!("{}:{}:{}", model_id, prompt, format);
    let cache_key = format!("{:x}", Sha256::digest(raw_key.as_bytes()));
    let cached = get_cached(&cache_key);
    if !force {
        if let Some(hit) = cached {
            return Value::String(hit);
        }
    }

    let response = if format == "code" {
        format!("// Generated code for {}: {}", model_id, prompt)
    } else {
        // Stub: echo prompt until real API integration is added
        prompt.to_string()
    };

    set_cached(&cache_key, &response);
    Value::String(response)
}

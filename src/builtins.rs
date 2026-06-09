// Built-in functions for the DSL interpreter

use crate::cache::{get_cached, set_cached};
use crate::interpreter::Value;
use sha2::{Digest, Sha256};

#[allow(dead_code)]
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
        matches!(&args[3], Value::String(s) if s == "true")
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
        // Stub: return prompt length as a number expression (placeholder for real codegen)
        prompt.len().to_string()
    } else {
        // Stub: echo prompt until real API integration is added
        prompt.to_string()
    };

    set_cached(&cache_key, &response);
    Value::String(response)
}

#[allow(dead_code)]
/// Evaluate a single C-DSL expression string in the caller's environment.
/// Signature: eval "<expression-string>"
pub fn builtin_eval(env: &crate::interpreter::Environment, args: Vec<Value>) -> Value {
    let code = match args.into_iter().next() {
        Some(Value::String(s)) => s,
        _ => return Value::Nil,
    };
    crate::interpreter::eval(env, &code)
}

/// Print the first argument to stdout and return it unchanged.
pub fn builtin_print(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(v) => {
            match &v {
                Value::Number(n) => println!("{}", n),
                Value::String(s) => println!("{}", s),
                Value::Nil => println!("nil"),
                Value::Function(_) => println!("<fn>"),
            }
            v
        }
        None => Value::Nil,
    }
}

use crate::cache::{get_cached, set_cached};
use crate::interpreter::{Environment, Value};
use sha2::{Digest, Sha256};

// ── PRNG (xorshift64, no external crate) ──────────────────────────────────
use std::cell::Cell;
thread_local! {
    static RNG: Cell<u64> = Cell::new(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(6364136223846793005)
    );
}
fn xorshift64() -> u64 {
    RNG.with(|s| {
        let mut x = s.get();
        x ^= x << 13; x ^= x >> 7; x ^= x << 17;
        s.set(x); x
    })
}

fn json_to_value(j: serde_json::Value) -> Value {
    match j {
        serde_json::Value::Null => Value::Nil,
        serde_json::Value::Bool(b) => Value::Number(if b { 1.0 } else { 0.0 }),
        serde_json::Value::Number(n) => Value::Number(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => Value::List(arr.into_iter().map(json_to_value).collect()),
        serde_json::Value::Object(obj) => {
            Value::Dict(obj.into_iter().map(|(k, v)| (k, json_to_value(v))).collect())
        }
    }
}

fn value_to_json(v: Value) -> serde_json::Value {
    match v {
        Value::Nil => serde_json::Value::Null,
        Value::Number(n) => {
            if n.is_finite() {
                serde_json::json!(n)
            } else {
                serde_json::Value::Null
            }
        }
        Value::String(s) => serde_json::Value::String(s),
        Value::List(items) => {
            serde_json::Value::Array(items.into_iter().map(value_to_json).collect())
        }
        Value::Dict(pairs) => serde_json::Value::Object(
            pairs.into_iter().map(|(k, v)| (k, value_to_json(v))).collect(),
        ),
        Value::Function(_) => serde_json::Value::String("<fn>".into()),
        Value::Error(e) => serde_json::Value::String(format!("<error: {}>", e)),
        Value::Return(_) | Value::Break | Value::Continue => serde_json::Value::Null,
    }
}

// ── model ──────────────────────────────────────────────────────────────────

pub fn model(_env: &Environment, args: Vec<Value>) -> Value {
    let model_id = match args.get(0) {
        Some(Value::String(s)) => s.clone(),
        _ => return Value::Error("model: first arg must be model-id string".into()),
    };
    let prompt = match args.get(1) {
        Some(Value::String(s)) => s.clone(),
        _ => return Value::Error("model: second arg must be prompt string".into()),
    };
    let format = match args.get(2) {
        Some(Value::String(s)) if s == "code" => "code",
        _ => "",
    };
    let force = matches!(args.get(3), Some(Value::String(s)) if s == "true");

    let raw_key = format!("{}:{}:{}", model_id, prompt, format);
    let cache_key = format!("{:x}", Sha256::digest(raw_key.as_bytes()));

    if !force {
        if let Some(hit) = get_cached(&cache_key) {
            return Value::String(hit);
        }
    }

    let api_key = match std::env::var("ANTHROPIC_API_KEY") {
        Ok(k) if !k.is_empty() => k,
        _ => return Value::Error("ANTHROPIC_API_KEY environment variable not set".into()),
    };

    let body = serde_json::json!({
        "model": model_id,
        "max_tokens": 4096,
        "messages": [{"role": "user", "content": prompt}]
    });

    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
    {
        Ok(c) => c,
        Err(e) => return Value::Error(format!("model: client build failed: {}", e)),
    };
    let response = match client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
    {
        Ok(r) => r,
        Err(e) => return Value::Error(format!("model: request failed: {}", e)),
    };

    let status = response.status();
    let text = match response.text() {
        Ok(t) => t,
        Err(e) => return Value::Error(format!("model: failed to read response: {}", e)),
    };

    if !status.is_success() {
        return Value::Error(format!("model: API error {}: {}", status, text));
    }

    let json: serde_json::Value = match serde_json::from_str(&text) {
        Ok(j) => j,
        Err(e) => return Value::Error(format!("model: invalid response JSON: {}", e)),
    };

    let content = match json["content"][0]["text"].as_str() {
        Some(s) => s.to_string(),
        None => return Value::Error("model: unexpected API response shape".into()),
    };

    set_cached(&cache_key, &content);
    Value::String(content)
}

// ── core ───────────────────────────────────────────────────────────────────

pub fn builtin_eval(env: &Environment, args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::String(s)) => crate::interpreter::eval(env, &s),
        _ => Value::Nil,
    }
}

pub fn builtin_print(args: Vec<Value>) -> Value {
    if args.is_empty() {
        println!();
        return Value::Nil;
    }
    let parts: Vec<String> = args.iter().map(|v| v.to_string()).collect();
    println!("{}", parts.join(" "));
    args.into_iter().last().unwrap_or(Value::Nil)
}

// ── environment ────────────────────────────────────────────────────────────

pub fn builtin_env(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::String(name)) => match std::env::var(&name) {
            Ok(val) => Value::String(val),
            Err(_) => Value::Nil,
        },
        _ => Value::Error("env: expected string argument".into()),
    }
}

// ── collections ────────────────────────────────────────────────────────────

pub fn builtin_len(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::List(items)) => Value::Number(items.len() as f64),
        Some(Value::String(s)) => Value::Number(s.chars().count() as f64),
        Some(Value::Dict(pairs)) => Value::Number(pairs.len() as f64),
        _ => Value::Error("len: expected list, string, or dict".into()),
    }
}

pub fn builtin_keys(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Dict(pairs)) => {
            Value::List(pairs.into_iter().map(|(k, _)| Value::String(k)).collect())
        }
        _ => Value::Error("keys: expected dict".into()),
    }
}

pub fn builtin_values(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Dict(pairs)) => Value::List(pairs.into_iter().map(|(_, v)| v).collect()),
        _ => Value::Error("values: expected dict".into()),
    }
}

pub fn builtin_push(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::List(mut items)), Some(val)) => {
            items.push(val);
            Value::List(items)
        }
        _ => Value::Error("push: expected (list, value)".into()),
    }
}

pub fn builtin_range(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    let (start, end) = match (iter.next(), iter.next()) {
        (Some(Value::Number(n)), None) => (0.0_f64, n),
        (Some(Value::Number(s)), Some(Value::Number(e))) => (s, e),
        _ => return Value::Error("range: expected range(n) or range(start, end)".into()),
    };
    if start.fract() != 0.0 || end.fract() != 0.0 || end < start {
        return Value::Error("range: args must be integers with end >= start".into());
    }
    Value::List(
        (start as i64..end as i64)
            .map(|i| Value::Number(i as f64))
            .collect(),
    )
}

pub fn builtin_contains(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::List(items)), Some(val)) => {
            Value::Number(if items.iter().any(|i| i == &val) { 1.0 } else { 0.0 })
        }
        (Some(Value::String(s)), Some(Value::String(sub))) => {
            Value::Number(if s.contains(sub.as_str()) { 1.0 } else { 0.0 })
        }
        (Some(Value::Dict(pairs)), Some(Value::String(key))) => {
            Value::Number(if pairs.iter().any(|(k, _)| k == &key) { 1.0 } else { 0.0 })
        }
        _ => Value::Error("contains: expected (collection, value)".into()),
    }
}

pub fn builtin_slice(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    let collection = iter.next();
    let start = match iter.next() {
        Some(Value::Number(n)) if n >= 0.0 => n as usize,
        _ => return Value::Error("slice: expected (collection, start[, end])".into()),
    };
    let end_opt = match iter.next() {
        Some(Value::Number(n)) if n >= 0.0 => Some(n as usize),
        None => None,
        _ => return Value::Error("slice: end must be a non-negative number".into()),
    };
    match collection {
        Some(Value::List(items)) => {
            let len = items.len();
            let end = end_opt.unwrap_or(len).min(len);
            Value::List(items[start.min(len)..end].to_vec())
        }
        Some(Value::String(s)) => {
            let chars: Vec<char> = s.chars().collect();
            let len = chars.len();
            let end = end_opt.unwrap_or(len).min(len);
            Value::String(chars[start.min(len)..end].iter().collect())
        }
        _ => Value::Error("slice: expected list or string".into()),
    }
}

pub fn builtin_sort(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::List(mut items)) => {
            items.sort_by(|a, b| match (a, b) {
                (Value::Number(x), Value::Number(y)) => {
                    x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                }
                (Value::String(x), Value::String(y)) => x.cmp(y),
                _ => std::cmp::Ordering::Equal,
            });
            Value::List(items)
        }
        _ => Value::Error("sort: expected list".into()),
    }
}

// ── higher-order ───────────────────────────────────────────────────────────

pub fn builtin_map(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::List(items)), Some(Value::Function(f))) => {
            let mut result = Vec::new();
            for item in items {
                let v = f.call(vec![item]);
                if crate::interpreter::is_signal(&v) {
                    return v;
                }
                result.push(v);
            }
            Value::List(result)
        }
        _ => Value::Error("map: expected (list, function)".into()),
    }
}

pub fn builtin_filter(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::List(items)), Some(Value::Function(f))) => {
            let mut result = Vec::new();
            for item in items {
                let v = f.call(vec![item.clone()]);
                if crate::interpreter::is_signal(&v) {
                    return v;
                }
                if crate::interpreter::is_truthy(&v) {
                    result.push(item);
                }
            }
            Value::List(result)
        }
        _ => Value::Error("filter: expected (list, function)".into()),
    }
}

pub fn builtin_reduce(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next(), iter.next()) {
        (Some(Value::List(items)), Some(Value::Function(f)), Some(initial)) => {
            let mut acc = initial;
            for item in items {
                let v = f.call(vec![acc, item]);
                if crate::interpreter::is_signal(&v) {
                    return v;
                }
                acc = v;
            }
            acc
        }
        _ => Value::Error("reduce: expected (list, function, initial)".into()),
    }
}

// ── strings ────────────────────────────────────────────────────────────────

pub fn builtin_split(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::String(s)), Some(Value::String(sep))) => {
            Value::List(
                s.split(sep.as_str())
                    .map(|p| Value::String(p.to_string()))
                    .collect(),
            )
        }
        (Some(Value::String(s)), None) => Value::List(
            s.split_whitespace()
                .map(|p| Value::String(p.to_string()))
                .collect(),
        ),
        _ => Value::Error("split: expected (string) or (string, separator)".into()),
    }
}

pub fn builtin_join(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::List(items)), Some(Value::String(sep))) => {
            let parts: Vec<String> = items.iter().map(|v| v.to_string()).collect();
            Value::String(parts.join(&sep))
        }
        (Some(Value::List(items)), None) => {
            let parts: Vec<String> = items.iter().map(|v| v.to_string()).collect();
            Value::String(parts.join(""))
        }
        _ => Value::Error("join: expected (list) or (list, separator)".into()),
    }
}

pub fn builtin_upper(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::String(s)) => Value::String(s.to_uppercase()),
        _ => Value::Error("upper: expected string".into()),
    }
}

pub fn builtin_lower(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::String(s)) => Value::String(s.to_lowercase()),
        _ => Value::Error("lower: expected string".into()),
    }
}

pub fn builtin_trim(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::String(s)) => Value::String(s.trim().to_string()),
        _ => Value::Error("trim: expected string".into()),
    }
}

// ── type conversion ────────────────────────────────────────────────────────

pub fn builtin_to_str(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(v) => Value::String(v.to_string()),
        None => Value::Error("str: expected one argument".into()),
    }
}

pub fn builtin_to_num(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Number(n)) => Value::Number(n),
        Some(Value::String(s)) => s
            .parse::<f64>()
            .map(Value::Number)
            .unwrap_or_else(|_| Value::Error(format!("num: cannot parse {:?}", s))),
        _ => Value::Error("num: expected string or number".into()),
    }
}

pub fn builtin_type_of(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Number(_)) => Value::String("number".into()),
        Some(Value::String(_)) => Value::String("string".into()),
        Some(Value::List(_)) => Value::String("list".into()),
        Some(Value::Dict(_)) => Value::String("dict".into()),
        Some(Value::Function(_)) => Value::String("function".into()),
        Some(Value::Nil) => Value::String("nil".into()),
        Some(Value::Error(_)) => Value::String("error".into()),
        Some(Value::Return(_)) => Value::String("return".into()),
        Some(Value::Break) => Value::String("break".into()),
        Some(Value::Continue) => Value::String("continue".into()),
        None => Value::Error("type: expected one argument".into()),
    }
}

// ── math ───────────────────────────────────────────────────────────────────

pub fn builtin_floor(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Number(n)) => Value::Number(n.floor()),
        _ => Value::Error("floor: expected number".into()),
    }
}

pub fn builtin_ceil(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Number(n)) => Value::Number(n.ceil()),
        _ => Value::Error("ceil: expected number".into()),
    }
}

pub fn builtin_round(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Number(n)) => Value::Number(n.round()),
        _ => Value::Error("round: expected number".into()),
    }
}

pub fn builtin_abs(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Number(n)) => Value::Number(n.abs()),
        _ => Value::Error("abs: expected number".into()),
    }
}

pub fn builtin_min(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::Number(a)), Some(Value::Number(b))) => Value::Number(a.min(b)),
        _ => Value::Error("min: expected two numbers".into()),
    }
}

pub fn builtin_max(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::Number(a)), Some(Value::Number(b))) => Value::Number(a.max(b)),
        _ => Value::Error("max: expected two numbers".into()),
    }
}

// ── HTTP / JSON ─────────────────────────────────────────────────────────────

pub fn builtin_http_get(args: Vec<Value>) -> Value {
    let url = match args.into_iter().next() {
        Some(Value::String(s)) => s,
        _ => return Value::Error("http_get: expected url string".into()),
    };
    let client = reqwest::blocking::Client::new();
    match client.get(&url).send().and_then(|r| r.text()) {
        Ok(t) => Value::String(t),
        Err(e) => Value::Error(format!("http_get failed: {}", e)),
    }
}

pub fn builtin_http_post(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    let url = match iter.next() {
        Some(Value::String(s)) => s,
        _ => return Value::Error("http_post: expected (url, body)".into()),
    };
    let body = match iter.next() {
        Some(v) => v.to_string(),
        None => return Value::Error("http_post: expected body as second arg".into()),
    };
    let client = reqwest::blocking::Client::new();
    match client
        .post(&url)
        .header("content-type", "application/json")
        .body(body)
        .send()
        .and_then(|r| r.text())
    {
        Ok(t) => Value::String(t),
        Err(e) => Value::Error(format!("http_post failed: {}", e)),
    }
}

pub fn builtin_json_parse(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::String(s)) => match serde_json::from_str::<serde_json::Value>(&s) {
            Ok(j) => json_to_value(j),
            Err(e) => Value::Error(format!("json_parse error: {}", e)),
        },
        _ => Value::Error("json_parse: expected string".into()),
    }
}

pub fn builtin_json_str(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(v) => match serde_json::to_string(&value_to_json(v)) {
            Ok(s) => Value::String(s),
            Err(e) => Value::Error(format!("json_str error: {}", e)),
        },
        None => Value::Error("json_str: expected one argument".into()),
    }
}

// ── file I/O ───────────────────────────────────────────────────────────────

pub fn builtin_read_file(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::String(path)) => match std::fs::read_to_string(&path) {
            Ok(content) => Value::String(content),
            Err(e) => Value::Error(format!("read_file: {}", e)),
        },
        _ => Value::Error("read_file: expected path string".into()),
    }
}

pub fn builtin_write_file(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::String(path)), Some(Value::String(content))) => {
            match std::fs::write(&path, &content) {
                Ok(_) => Value::Nil,
                Err(e) => Value::Error(format!("write_file: {}", e)),
            }
        }
        _ => Value::Error("write_file: expected (path, content)".into()),
    }
}

pub fn builtin_append_file(args: Vec<Value>) -> Value {
    use std::io::Write;
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::String(path)), Some(Value::String(content))) => {
            match std::fs::OpenOptions::new().append(true).create(true).open(&path) {
                Ok(mut f) => match f.write_all(content.as_bytes()) {
                    Ok(_) => Value::Nil,
                    Err(e) => Value::Error(format!("append_file: {}", e)),
                },
                Err(e) => Value::Error(format!("append_file: {}", e)),
            }
        }
        _ => Value::Error("append_file: expected (path, content)".into()),
    }
}

// ── process ────────────────────────────────────────────────────────────────

pub fn builtin_exit(args: Vec<Value>) -> Value {
    let code = match args.into_iter().next() {
        Some(Value::Number(n)) => n as i32,
        None => 0,
        _ => 1,
    };
    std::process::exit(code);
}

pub fn builtin_sleep(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Number(ms)) if ms >= 0.0 => {
            std::thread::sleep(std::time::Duration::from_millis(ms as u64));
            Value::Nil
        }
        _ => Value::Error("sleep: expected non-negative number (milliseconds)".into()),
    }
}

// ── nil / type checks ──────────────────────────────────────────────────────

pub fn builtin_is_nil(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Nil) => Value::Number(1.0),
        Some(_) => Value::Number(0.0),
        None => Value::Error("is_nil: expected one argument".into()),
    }
}

// ── list extras ────────────────────────────────────────────────────────────

pub fn builtin_concat(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::List(mut a)), Some(Value::List(b))) => {
            a.extend(b);
            Value::List(a)
        }
        _ => Value::Error("concat: expected (list, list)".into()),
    }
}

pub fn builtin_flat(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::List(items)) => {
            let mut out = Vec::new();
            for item in items {
                match item {
                    Value::List(inner) => out.extend(inner),
                    other => out.push(other),
                }
            }
            Value::List(out)
        }
        _ => Value::Error("flat: expected list".into()),
    }
}

pub fn builtin_first(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::List(items)) => items.into_iter().next().unwrap_or(Value::Nil),
        Some(Value::String(s)) => s.chars().next().map(|c| Value::String(c.to_string())).unwrap_or(Value::Nil),
        _ => Value::Error("first: expected list or string".into()),
    }
}

pub fn builtin_last(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::List(items)) => items.into_iter().last().unwrap_or(Value::Nil),
        Some(Value::String(s)) => s.chars().last().map(|c| Value::String(c.to_string())).unwrap_or(Value::Nil),
        _ => Value::Error("last: expected list or string".into()),
    }
}

pub fn builtin_pop(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::List(mut items)) => {
            items.pop();
            Value::List(items)
        }
        _ => Value::Error("pop: expected list".into()),
    }
}

pub fn builtin_set(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next(), iter.next()) {
        (Some(Value::Dict(mut pairs)), Some(Value::String(key)), Some(val)) => {
            if let Some(entry) = pairs.iter_mut().find(|(k, _)| k == &key) {
                entry.1 = val;
            } else {
                pairs.push((key, val));
            }
            Value::Dict(pairs)
        }
        _ => Value::Error("set: expected (dict, key, value)".into()),
    }
}

// ── stdin I/O ──────────────────────────────────────────────────────────────

pub fn builtin_input(args: Vec<Value>) -> Value {
    use std::io::{self, Write};
    let prompt = match args.into_iter().next() {
        Some(Value::String(s)) => s,
        None => String::new(),
        _ => return Value::Error("input: expected optional string prompt".into()),
    };
    if !prompt.is_empty() {
        print!("{}", prompt);
        io::stdout().flush().ok();
    }
    let mut line = String::new();
    match io::stdin().read_line(&mut line) {
        Ok(_) => Value::String(line.trim_end_matches('\n').trim_end_matches('\r').to_string()),
        Err(e) => Value::Error(format!("input error: {}", e)),
    }
}

// ── math (extended) ────────────────────────────────────────────────────────

pub fn builtin_sqrt(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Number(n)) => Value::Number(n.sqrt()),
        _ => Value::Error("sqrt: expected number".into()),
    }
}

pub fn builtin_cbrt(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Number(n)) => Value::Number(n.cbrt()),
        _ => Value::Error("cbrt: expected number".into()),
    }
}

pub fn builtin_pow(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::Number(x)), Some(Value::Number(y))) => Value::Number(x.powf(y)),
        _ => Value::Error("pow: expected (number, number)".into()),
    }
}

pub fn builtin_log(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::Number(x)), None) => Value::Number(x.ln()),
        (Some(Value::Number(x)), Some(Value::Number(base))) => Value::Number(x.log(base)),
        _ => Value::Error("log: expected number or (number, base)".into()),
    }
}

pub fn builtin_log2(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Number(n)) => Value::Number(n.log2()),
        _ => Value::Error("log2: expected number".into()),
    }
}

pub fn builtin_log10(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Number(n)) => Value::Number(n.log10()),
        _ => Value::Error("log10: expected number".into()),
    }
}

pub fn builtin_exp(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Number(n)) => Value::Number(n.exp()),
        _ => Value::Error("exp: expected number".into()),
    }
}

pub fn builtin_sin(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Number(n)) => Value::Number(n.sin()),
        _ => Value::Error("sin: expected number".into()),
    }
}

pub fn builtin_cos(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Number(n)) => Value::Number(n.cos()),
        _ => Value::Error("cos: expected number".into()),
    }
}

pub fn builtin_tan(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Number(n)) => Value::Number(n.tan()),
        _ => Value::Error("tan: expected number".into()),
    }
}

pub fn builtin_asin(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Number(n)) => Value::Number(n.asin()),
        _ => Value::Error("asin: expected number".into()),
    }
}

pub fn builtin_acos(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Number(n)) => Value::Number(n.acos()),
        _ => Value::Error("acos: expected number".into()),
    }
}

pub fn builtin_atan(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Number(n)) => Value::Number(n.atan()),
        _ => Value::Error("atan: expected number".into()),
    }
}

pub fn builtin_atan2(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::Number(y)), Some(Value::Number(x))) => Value::Number(y.atan2(x)),
        _ => Value::Error("atan2: expected (y, x) numbers".into()),
    }
}

pub fn builtin_hypot(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::Number(x)), Some(Value::Number(y))) => Value::Number(x.hypot(y)),
        _ => Value::Error("hypot: expected (x, y) numbers".into()),
    }
}

pub fn builtin_clamp(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next(), iter.next()) {
        (Some(Value::Number(x)), Some(Value::Number(lo)), Some(Value::Number(hi))) => {
            if lo > hi {
                return Value::Error(format!("clamp: lo ({}) must be <= hi ({})", lo, hi));
            }
            let clamped = if x < lo { lo } else if x > hi { hi } else { x };
            Value::Number(clamped)
        }
        _ => Value::Error("clamp: expected (x, lo, hi) numbers".into()),
    }
}

pub fn builtin_sign(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Number(n)) => Value::Number(if n > 0.0 { 1.0 } else if n < 0.0 { -1.0 } else { 0.0 }),
        _ => Value::Error("sign: expected number".into()),
    }
}

pub fn builtin_random(args: Vec<Value>) -> Value {
    let _ = args;
    let bits = xorshift64();
    Value::Number((bits >> 11) as f64 * (1.0 / (1u64 << 53) as f64))
}

pub fn builtin_rand_int(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::Number(n)), None) => {
            let n = n as u64;
            if n == 0 { return Value::Error("rand_int: n must be > 0".into()); }
            Value::Number((xorshift64() % n) as f64)
        }
        (Some(Value::Number(lo)), Some(Value::Number(hi))) => {
            let lo = lo as i64;
            let hi = hi as i64;
            if hi <= lo { return Value::Error("rand_int: hi must be > lo".into()); }
            let range = (hi as i128 - lo as i128) as u64;
            Value::Number(lo as f64 + (xorshift64() % range) as f64)
        }
        _ => Value::Error("rand_int: expected rand_int(n) or rand_int(lo, hi)".into()),
    }
}

// ── strings (extended) ─────────────────────────────────────────────────────

pub fn builtin_replace(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next(), iter.next()) {
        (Some(Value::String(s)), Some(Value::String(old)), Some(Value::String(new))) => {
            Value::String(s.replace(old.as_str(), new.as_str()))
        }
        _ => Value::Error("replace: expected (string, old, new)".into()),
    }
}

pub fn builtin_starts_with(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::String(s)), Some(Value::String(prefix))) => {
            Value::Number(if s.starts_with(prefix.as_str()) { 1.0 } else { 0.0 })
        }
        _ => Value::Error("starts_with: expected (string, prefix)".into()),
    }
}

pub fn builtin_ends_with(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::String(s)), Some(Value::String(suffix))) => {
            Value::Number(if s.ends_with(suffix.as_str()) { 1.0 } else { 0.0 })
        }
        _ => Value::Error("ends_with: expected (string, suffix)".into()),
    }
}

/// index_of: works for both strings (substring search) and lists (equality search).
pub fn builtin_index_of(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::String(s)), Some(Value::String(sub))) => {
            match s.find(sub.as_str()) {
                Some(byte_idx) => Value::Number(s[..byte_idx].chars().count() as f64),
                None => Value::Number(-1.0),
            }
        }
        (Some(Value::List(items)), Some(val)) => {
            match items.iter().position(|i| i == &val) {
                Some(idx) => Value::Number(idx as f64),
                None => Value::Number(-1.0),
            }
        }
        _ => Value::Error("index_of: expected (string, substring) or (list, value)".into()),
    }
}

pub fn builtin_repeat(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::String(s)), Some(Value::Number(n))) => {
            if n < 0.0 { return Value::Error("repeat: n must be non-negative".into()); }
            if n > 1_000_000.0 { return Value::Error("repeat: count too large".into()); }
            Value::String(s.repeat(n as usize))
        }
        (Some(Value::List(items)), Some(Value::Number(n))) => {
            if n < 0.0 { return Value::Error("repeat: n must be non-negative".into()); }
            if n > 1_000_000.0 { return Value::Error("repeat: count too large".into()); }
            let n = n as usize;
            let mut result = Vec::with_capacity(items.len() * n);
            for _ in 0..n {
                result.extend(items.iter().cloned());
            }
            Value::List(result)
        }
        _ => Value::Error("repeat: expected (string/list, n)".into()),
    }
}

pub fn builtin_char_at(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::String(s)), Some(Value::Number(n))) => {
            let chars: Vec<char> = s.chars().collect();
            let len = chars.len() as i64;
            let idx = if n < 0.0 { len + n as i64 } else { n as i64 };
            if idx < 0 || idx >= len {
                Value::Error(format!("char_at: index {} out of bounds", n as i64))
            } else {
                Value::String(chars[idx as usize].to_string())
            }
        }
        _ => Value::Error("char_at: expected (string, index)".into()),
    }
}

pub fn builtin_chars(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::String(s)) => {
            Value::List(s.chars().map(|c| Value::String(c.to_string())).collect())
        }
        _ => Value::Error("chars: expected string".into()),
    }
}

/// format(template, arg1, arg2, ...) — replaces each "{}" in order.
pub fn builtin_format(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    let template = match iter.next() {
        Some(Value::String(s)) => s,
        _ => return Value::Error("format: first arg must be a template string".into()),
    };
    let rest: Vec<Value> = iter.collect();
    let mut result = String::new();
    let mut arg_iter = rest.into_iter();
    let mut parts = template.split("{}");
    if let Some(first) = parts.next() {
        result.push_str(first);
    }
    for part in parts {
        match arg_iter.next() {
            Some(v) => result.push_str(&v.to_string()),
            None => result.push_str("{}"),
        }
        result.push_str(part);
    }
    Value::String(result)
}

// ── list (extended) ────────────────────────────────────────────────────────

pub fn builtin_reverse(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::List(mut items)) => { items.reverse(); Value::List(items) }
        _ => Value::Error("reverse: expected list".into()),
    }
}

pub fn builtin_unique(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::List(items)) => {
            let mut seen: Vec<Value> = Vec::new();
            let mut result = Vec::new();
            for item in items {
                if !seen.iter().any(|s| s == &item) {
                    seen.push(item.clone());
                    result.push(item);
                }
            }
            Value::List(result)
        }
        _ => Value::Error("unique: expected list".into()),
    }
}

pub fn builtin_zip(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::List(a)), Some(Value::List(b))) => {
            Value::List(
                a.into_iter().zip(b.into_iter())
                    .map(|(x, y)| Value::List(vec![x, y]))
                    .collect()
            )
        }
        _ => Value::Error("zip: expected (list, list)".into()),
    }
}

pub fn builtin_enumerate(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::List(items)) => {
            Value::List(
                items.into_iter().enumerate()
                    .map(|(i, v)| Value::List(vec![Value::Number(i as f64), v]))
                    .collect()
            )
        }
        _ => Value::Error("enumerate: expected list".into()),
    }
}

pub fn builtin_any(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::List(items)), Some(Value::Function(f))) => {
            for item in items {
                let v = f.call(vec![item]);
                if crate::interpreter::is_signal(&v) { return v; }
                if crate::interpreter::is_truthy(&v) { return Value::Number(1.0); }
            }
            Value::Number(0.0)
        }
        _ => Value::Error("any: expected (list, function)".into()),
    }
}

pub fn builtin_all(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::List(items)), Some(Value::Function(f))) => {
            for item in items {
                let v = f.call(vec![item]);
                if crate::interpreter::is_signal(&v) { return v; }
                if !crate::interpreter::is_truthy(&v) { return Value::Number(0.0); }
            }
            Value::Number(1.0)
        }
        _ => Value::Error("all: expected (list, function)".into()),
    }
}

pub fn builtin_sum(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::List(items)) => {
            let mut total = 0.0f64;
            for item in items {
                match item {
                    Value::Number(n) => total += n,
                    _ => return Value::Error("sum: list must contain only numbers".into()),
                }
            }
            Value::Number(total)
        }
        _ => Value::Error("sum: expected list".into()),
    }
}

pub fn builtin_product(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::List(items)) => {
            let mut total = 1.0f64;
            for item in items {
                match item {
                    Value::Number(n) => total *= n,
                    _ => return Value::Error("product: list must contain only numbers".into()),
                }
            }
            Value::Number(total)
        }
        _ => Value::Error("product: expected list".into()),
    }
}

pub fn builtin_find_where(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::List(items)), Some(Value::Function(f))) => {
            for item in items {
                let v = f.call(vec![item.clone()]);
                if crate::interpreter::is_signal(&v) { return v; }
                if crate::interpreter::is_truthy(&v) { return item; }
            }
            Value::Nil
        }
        _ => Value::Error("find_where: expected (list, function)".into()),
    }
}

pub fn builtin_flat_map(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::List(items)), Some(Value::Function(f))) => {
            let mut result = Vec::new();
            for item in items {
                let v = f.call(vec![item]);
                if crate::interpreter::is_signal(&v) { return v; }
                match v {
                    Value::List(inner) => result.extend(inner),
                    other => result.push(other),
                }
            }
            Value::List(result)
        }
        _ => Value::Error("flat_map: expected (list, function)".into()),
    }
}

pub fn builtin_take(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::List(items)), Some(Value::Number(n))) => {
            if n < 0.0 { return Value::Error("take: n must be non-negative".into()); }
            Value::List(items.into_iter().take(n as usize).collect())
        }
        _ => Value::Error("take: expected (list, n)".into()),
    }
}

pub fn builtin_skip(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::List(items)), Some(Value::Number(n))) => {
            if n < 0.0 { return Value::Error("skip: n must be non-negative".into()); }
            Value::List(items.into_iter().skip(n as usize).collect())
        }
        _ => Value::Error("skip: expected (list, n)".into()),
    }
}

pub fn builtin_count(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::List(items)), Some(Value::Function(f))) => {
            let mut count = 0usize;
            for item in items {
                let v = f.call(vec![item]);
                if crate::interpreter::is_signal(&v) { return v; }
                if crate::interpreter::is_truthy(&v) { count += 1; }
            }
            Value::Number(count as f64)
        }
        _ => Value::Error("count: expected (list, function)".into()),
    }
}

pub fn builtin_group_by(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::List(items)), Some(Value::Function(f))) => {
            let mut groups: Vec<(String, Value)> = Vec::new();
            for item in items {
                let key_val = f.call(vec![item.clone()]);
                if crate::interpreter::is_signal(&key_val) { return key_val; }
                let key = key_val.to_string();
                if let Some(entry) = groups.iter_mut().find(|(k, _)| k == &key) {
                    if let Value::List(ref mut list) = entry.1 {
                        list.push(item);
                    }
                } else {
                    groups.push((key, Value::List(vec![item])));
                }
            }
            Value::Dict(groups)
        }
        _ => Value::Error("group_by: expected (list, function)".into()),
    }
}

// ── dict (extended) ────────────────────────────────────────────────────────

pub fn builtin_get(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next(), iter.next()) {
        (Some(Value::Dict(pairs)), Some(Value::String(key)), Some(default)) => {
            pairs.into_iter()
                .find(|(k, _)| k == &key)
                .map(|(_, v)| v)
                .unwrap_or(default)
        }
        _ => Value::Error("get: expected (dict, key, default)".into()),
    }
}

pub fn builtin_del(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::Dict(pairs)), Some(Value::String(key))) => {
            Value::Dict(pairs.into_iter().filter(|(k, _)| k != &key).collect())
        }
        _ => Value::Error("del: expected (dict, key)".into()),
    }
}

pub fn builtin_merge(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::Dict(mut d1)), Some(Value::Dict(d2))) => {
            for (k, v) in d2 {
                if let Some(entry) = d1.iter_mut().find(|(ek, _)| ek == &k) {
                    entry.1 = v;
                } else {
                    d1.push((k, v));
                }
            }
            Value::Dict(d1)
        }
        _ => Value::Error("merge: expected (dict, dict)".into()),
    }
}

pub fn builtin_has(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::Dict(pairs)), Some(Value::String(key))) => {
            Value::Number(if pairs.iter().any(|(k, _)| k == &key) { 1.0 } else { 0.0 })
        }
        _ => Value::Error("has: expected (dict, key)".into()),
    }
}

// ── error helpers ──────────────────────────────────────────────────────────

pub fn builtin_make_error(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::String(s)) => Value::Error(s),
        Some(v) => Value::Error(v.to_string()),
        None => Value::Error("error: expected message".into()),
    }
}

pub fn builtin_is_error(args: Vec<Value>) -> Value {
    match args.into_iter().next() {
        Some(Value::Error(_)) => Value::Number(1.0),
        Some(_) => Value::Number(0.0),
        None => Value::Error("is_error: expected one argument".into()),
    }
}

pub fn builtin_ok(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next()) {
        (Some(Value::Error(_)), Some(default)) => default,
        (Some(v), Some(_)) => v,
        _ => Value::Error("ok: expected (value, default)".into()),
    }
}

pub fn builtin_zip_with(args: Vec<Value>) -> Value {
    let mut iter = args.into_iter();
    match (iter.next(), iter.next(), iter.next()) {
        (Some(Value::List(a)), Some(Value::List(b)), Some(Value::Function(f))) => {
            let len = a.len().min(b.len());
            let mut result = Vec::with_capacity(len);
            for (x, y) in a.into_iter().zip(b.into_iter()).take(len) {
                let v = f.call(vec![x, y]);
                if crate::interpreter::is_signal(&v) { return v; }
                result.push(v);
            }
            Value::List(result)
        }
        _ => Value::Error("zip_with: expected (list, list, function)".into()),
    }
}

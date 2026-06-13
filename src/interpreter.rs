use crate::parser::{parse_src, Expr};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Function(Function),
    Nil,
    List(Vec<Value>),
    Dict(Vec<(String, Value)>),
    Error(String),
    Return(Box<Value>),
    Break,
    Continue,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Function(a), Value::Function(b)) => a.params == b.params && a.body == b.body,
            (Value::List(a),  Value::List(b))  => a == b,
            (Value::Dict(a),  Value::Dict(b))  => {
                if a.len() != b.len() { return false; }
                a.iter().all(|(k, v)| {
                    b.iter().find(|(bk, _)| bk == k)
                        .map(|(_, bv)| bv == v)
                        .unwrap_or(false)
                })
            }
            (Value::Error(a), Value::Error(b)) => a == b,
            (Value::Return(a), Value::Return(b)) => a == b,
            (Value::Break, Value::Break) => true,
            (Value::Continue, Value::Continue) => true,
            _ => false,
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "Number({})", n),
            Value::String(s) => write!(f, "String({})", s),
            Value::Function(_) => write!(f, "Function(...)"),
            Value::Nil => write!(f, "Nil"),
            Value::List(items) => write!(f, "List({:?})", items),
            Value::Dict(pairs) => write!(f, "Dict({:?})", pairs),
            Value::Error(msg) => write!(f, "Error({})", msg),
            Value::Return(v) => write!(f, "Return({:?})", v),
            Value::Break => write!(f, "Break"),
            Value::Continue => write!(f, "Continue"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Environment {
    name_value: HashMap<String, Value>,
    parent: Option<Rc<Environment>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            name_value: HashMap::new(),
            parent: None,
        }
    }
    pub fn with_parent(parent: Environment) -> Self {
        Environment {
            name_value: HashMap::new(),
            parent: Some(Rc::new(parent)),
        }
    }
    pub fn find(&self, name: &str) -> Option<Value> {
        self.name_value
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.find(name)))
    }
    pub fn define(&mut self, name: String, value: Value) {
        self.name_value.insert(name, value);
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub params: Vec<String>,
    pub body: Expr,
    pub parent_env: Rc<Environment>,
}

impl Function {
    pub fn call(&self, args: Vec<Value>) -> Value {
        if args.len() != self.params.len() {
            return Value::Error(format!(
                "arity mismatch: fn expects {} args, got {}",
                self.params.len(), args.len()
            ));
        }
        let parent = (*self.parent_env).clone();
        let mut env = Environment::with_parent(parent);
        for (param, arg) in self.params.iter().zip(args) {
            env.define(param.clone(), arg);
        }
        match eval_expr(&mut env, &self.body) {
            Value::Return(v)  => *v,
            Value::Break      => Value::Error("break outside of loop".into()),
            Value::Continue   => Value::Error("continue outside of loop".into()),
            other => other,
        }
    }
}

#[inline]
pub(crate) fn is_signal(v: &Value) -> bool {
    matches!(v, Value::Error(_) | Value::Return(_) | Value::Break | Value::Continue)
}

pub fn eval_expr(env: &mut Environment, expr: &Expr) -> Value {
    match expr {
        Expr::Number(n) => Value::Number(*n),
        Expr::Str(s) => Value::String(s.clone()),
        Expr::Ident(name) => env.find(name)
            .unwrap_or_else(|| Value::Error(format!("undefined variable: {}", name))),
        Expr::Neg(inner) => match eval_expr(env, inner) {
            Value::Number(n) => Value::Number(-n),
            v if is_signal(&v) => v,
            _ => Value::Error("type error: unary '-' requires a number".into()),
        },
        Expr::Block(stmts) => {
            let mut last = Value::Nil;
            for stmt in stmts {
                last = eval_expr(env, stmt);
                if is_signal(&last) { return last; }
            }
            last
        }
        Expr::Assign { name, value } => {
            let val = eval_expr(env, value);
            if matches!(&val, Value::Return(_) | Value::Break | Value::Continue) { return val; }
            // Intentional: errors are assignable values, not propagated signals.
            // x = risky_call() stores the error in x so callers can inspect it.
            env.define(name.clone(), val.clone());
            match val {
                Value::Error(_) => Value::Nil,
                other => other,
            }
        }
        Expr::BinOp { op, left, right } => {
            let l = eval_expr(env, left);
            if is_signal(&l) { return l; }
            let r = eval_expr(env, right);
            if is_signal(&r) { return r; }
            eval_binop(op, l, r)
        }
        Expr::FnDef { name, params, body } => {
            env.define(name.clone(), Value::Nil); // placeholder so name is in env before capture
            let f = Value::Function(Function {
                params: params.clone(),
                body: *body.clone(),
                parent_env: Rc::new(env.clone()),
            });
            env.define(name.clone(), f.clone());
            f
        }
        Expr::Lambda { params, body } => Value::Function(Function {
            params: params.clone(),
            body: *body.clone(),
            parent_env: Rc::new(env.clone()),
        }),
        Expr::Call { name, args } => {
            // is_error / ok must receive Error values as arguments rather than
            // having them propagate upward — suppress error signals for these.
            let suppress_error = matches!(name.as_str(), "is_error" | "ok");
            let mut eval_args = Vec::new();
            for arg in args {
                let v = eval_expr(env, arg);
                if is_signal(&v) {
                    if suppress_error && matches!(v, Value::Error(_)) {
                        // pass error through as argument value
                    } else {
                        return v;
                    }
                }
                eval_args.push(v);
            }
            call_fn(env, name, eval_args)
        }
        Expr::If { cond, then, else_ } => {
            let cond_val = eval_expr(env, cond);
            if is_signal(&cond_val) { return cond_val; }
            if is_truthy(&cond_val) {
                eval_expr(env, then)
            } else {
                eval_expr(env, else_)
            }
        }
        Expr::Pipe { left, right } => {
            let left_val = eval_expr(env, left);
            // Check if RHS is is_error or ok — these need to receive errors as values
            let suppress = match right.as_ref() {
                Expr::Ident(name) => matches!(name.as_str(), "is_error" | "ok"),
                Expr::Call { name, .. } => matches!(name.as_str(), "is_error" | "ok"),
                _ => false,
            };
            if is_signal(&left_val) && !(suppress && matches!(left_val, Value::Error(_))) {
                return left_val;
            }
            match right.as_ref() {
                Expr::Ident(name) => call_fn(env, name, vec![left_val]),
                Expr::Call { name, args } => {
                    let mut eval_args = Vec::new();
                    for arg in args {
                        let v = eval_expr(env, arg);
                        if is_signal(&v) { return v; }
                        eval_args.push(v);
                    }
                    eval_args.insert(0, left_val);
                    call_fn(env, name, eval_args)
                }
                _ => Value::Error("pipe right-hand side must be a function call".into()),
            }
        }
        Expr::Each { items, func } => {
            let func_val = eval_expr(env, func);
            if is_signal(&func_val) { return func_val; }
            // If a single expression evaluates to a List, iterate its elements
            let to_iterate: Vec<Value> = if items.len() == 1 {
                let v = eval_expr(env, &items[0]);
                if is_signal(&v) { return v; }
                match v {
                    Value::List(elems) => elems,
                    single => vec![single],
                }
            } else {
                let mut vals = Vec::new();
                for item in items {
                    let v = eval_expr(env, item);
                    if is_signal(&v) { return v; }
                    vals.push(v);
                }
                vals
            };
            let mut last = Value::Nil;
            for item_val in to_iterate {
                last = match &func_val {
                    Value::Function(f) => f.call(vec![item_val]),
                    _ => return Value::Error("each requires a function".into()),
                };
                if is_signal(&last) { return last; }
            }
            last
        }
        Expr::List(items) => {
            let mut result = Vec::new();
            for item in items {
                let v = eval_expr(env, item);
                if is_signal(&v) { return v; }
                result.push(v);
            }
            Value::List(result)
        }
        Expr::Dict(pairs) => {
            let mut result = Vec::new();
            for (k, v) in pairs {
                let val = eval_expr(env, v);
                if is_signal(&val) { return val; }
                result.push((k.clone(), val));
            }
            Value::Dict(result)
        }
        Expr::Index { object, index } => {
            let obj_val = eval_expr(env, object);
            if is_signal(&obj_val) { return obj_val; }
            let idx_val = eval_expr(env, index);
            if is_signal(&idx_val) { return idx_val; }
            match (obj_val, idx_val) {
                (Value::List(items), Value::Number(n)) => {
                    let len = items.len() as f64;
                    let idx = if n < 0.0 { len + n } else { n };
                    if idx < 0.0 || idx.fract() != 0.0 {
                        return Value::Error(format!("invalid index: {}", n));
                    }
                    items.get(idx as usize).cloned()
                        .unwrap_or_else(|| Value::Error(format!("index out of bounds: {}", n as i64)))
                }
                (Value::Dict(pairs), Value::String(key)) => {
                    pairs.into_iter()
                        .find(|(k, _)| k == &key)
                        .map(|(_, v)| v)
                        .unwrap_or(Value::Nil)
                }
                (obj, idx) => Value::Error(format!(
                    "type error: cannot index {} with {}",
                    obj, idx
                )),
            }
        }
        Expr::FieldAccess { object, field } => {
            let obj = eval_expr(env, object);
            // Handle error fields before signal propagation so .message / .type still work
            if let Value::Error(ref msg) = obj {
                return match field.as_str() {
                    "message" => Value::String(msg.clone()),
                    "type"    => Value::String("error".into()),
                    _         => obj,   // propagate error for unknown fields
                };
            }
            if is_signal(&obj) { return obj; }
            match (&obj, field.as_str()) {
                (Value::Dict(pairs), key) => pairs.iter()
                    .find(|(k, _)| k == key)
                    .map(|(_, v)| v.clone())
                    .unwrap_or(Value::Nil),
                _ => Value::Error(format!("no field '{}' on {}", field, obj)),
            }
        }
        Expr::And { left, right } => {
            let l = eval_expr(env, left);
            if is_signal(&l) { return l; }
            if !is_truthy(&l) { return Value::Number(0.0); }
            let r = eval_expr(env, right);
            if is_signal(&r) { return r; }
            Value::Number(if is_truthy(&r) { 1.0 } else { 0.0 })
        }
        Expr::Or { left, right } => {
            let l = eval_expr(env, left);
            if is_signal(&l) { return l; }
            if is_truthy(&l) { return Value::Number(1.0); }
            let r = eval_expr(env, right);
            if is_signal(&r) { return r; }
            Value::Number(if is_truthy(&r) { 1.0 } else { 0.0 })
        }
        Expr::Not(inner) => {
            let v = eval_expr(env, inner);
            if is_signal(&v) { return v; }
            Value::Number(if is_truthy(&v) { 0.0 } else { 1.0 })
        }
        Expr::TryCatch { body, catch_var, handler } => {
            let result = eval_expr(env, body);
            if let Value::Error(_) = &result {
                env.define(catch_var.clone(), result);
                eval_expr(env, handler)
            } else {
                result  // Return/Break/Continue propagate unchanged
            }
        }
        Expr::While { cond, body } => {
            let mut last = Value::Nil;
            loop {
                let cond_val = eval_expr(env, cond);
                if is_signal(&cond_val) { return cond_val; }
                if !is_truthy(&cond_val) { break; }
                last = eval_expr(env, body);
                if matches!(last, Value::Break) {
                    last = Value::Nil;
                    break;
                } else if matches!(last, Value::Continue) {
                    last = Value::Nil;
                    continue;
                } else if is_signal(&last) {
                    return last;
                }
            }
            last
        }
        Expr::Return(maybe_val) => {
            let val = match maybe_val {
                Some(e) => { let v = eval_expr(env, e); if is_signal(&v) { return v; } v }
                None => Value::Nil,
            };
            Value::Return(Box::new(val))
        }
        Expr::Break    => Value::Break,
        Expr::Continue => Value::Continue,
    }
}

pub(crate) fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Number(n)   => *n != 0.0,
        Value::String(s)   => !s.is_empty(),
        Value::Nil         => false,
        Value::Function(_) => true,
        Value::List(items) => !items.is_empty(),
        Value::Dict(pairs) => !pairs.is_empty(),
        Value::Error(_)    => false,
        Value::Return(_) | Value::Break | Value::Continue => false,
    }
}

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Number(_)   => "Number",
        Value::String(_)   => "String",
        Value::Nil         => "Nil",
        Value::Function(_) => "Function",
        Value::List(_)     => "List",
        Value::Dict(_)     => "Dict",
        Value::Error(_)    => "Error",
        Value::Return(_)   => "return",
        Value::Break       => "break",
        Value::Continue    => "continue",
    }
}

fn eval_binop(op: &str, left: Value, right: Value) -> Value {
    match (op, &left, &right) {
        // arithmetic
        ("+", Value::Number(l), Value::Number(r)) => Value::Number(l + r),
        ("-", Value::Number(l), Value::Number(r)) => Value::Number(l - r),
        ("*", Value::Number(l), Value::Number(r)) => Value::Number(l * r),
        ("/", Value::Number(l), Value::Number(r)) => {
            if *r == 0.0 {
                Value::Error("division by zero".into())
            } else {
                Value::Number(l / r)
            }
        }
        ("**", Value::Number(l), Value::Number(r)) => Value::Number(l.powf(*r)),
        ("%",  Value::Number(l), Value::Number(r)) => {
            if *r == 0.0 { Value::Error("modulo by zero".into()) }
            else { Value::Number(l % r) }
        }
        // string concat
        ("+", Value::String(l), Value::String(r)) => Value::String(l.clone() + r),
        // ordering (numbers only)
        (">",  Value::Number(l), Value::Number(r)) => Value::Number(if l > r  { 1.0 } else { 0.0 }),
        ("<",  Value::Number(l), Value::Number(r)) => Value::Number(if l < r  { 1.0 } else { 0.0 }),
        (">=", Value::Number(l), Value::Number(r)) => Value::Number(if l >= r { 1.0 } else { 0.0 }),
        (">=", _, _) => Value::Error("type error: '>=' requires numbers".into()),
        ("<=", Value::Number(l), Value::Number(r)) => Value::Number(if l <= r { 1.0 } else { 0.0 }),
        ("<=", _, _) => Value::Error("type error: '<=' requires numbers".into()),
        // equality
        ("==", Value::Number(l), Value::Number(r)) => Value::Number(if (l - r).abs() < f64::EPSILON { 1.0 } else { 0.0 }),
        ("==", Value::String(l), Value::String(r)) => Value::Number(if l == r { 1.0 } else { 0.0 }),
        ("==", Value::Nil, Value::Nil) => Value::Number(1.0),
        ("==", Value::Nil, _) | ("==", _, Value::Nil) => Value::Number(0.0),
        ("==", _, _) => Value::Error(format!("type error: cannot compare {} with {}", type_name(&left), type_name(&right))),
        // inequality
        ("!=", Value::Number(l), Value::Number(r)) => Value::Number(if (l - r).abs() >= f64::EPSILON { 1.0 } else { 0.0 }),
        ("!=", Value::String(l), Value::String(r)) => Value::Number(if l != r { 1.0 } else { 0.0 }),
        ("!=", Value::Nil, Value::Nil) => Value::Number(0.0),
        ("!=", Value::Nil, _) | ("!=", _, Value::Nil) => Value::Number(1.0),
        ("!=", _, _) => Value::Error(format!("type error: cannot compare {} with {}", type_name(&left), type_name(&right))),
        // fallback
        _ => Value::Error(format!("type error: '{}' not supported for these types", op)),
    }
}

fn call_fn(env: &mut Environment, name: &str, args: Vec<Value>) -> Value {
    match name {
        // core
        "print"      => crate::builtins::builtin_print(args),
        "eval"       => crate::builtins::builtin_eval(env, args),
        "model"      => crate::builtins::model(env, args),
        // environment
        "env"        => crate::builtins::builtin_env(args),
        // collections
        "len"        => crate::builtins::builtin_len(args),
        "keys"       => crate::builtins::builtin_keys(args),
        "values"     => crate::builtins::builtin_values(args),
        "push"       => crate::builtins::builtin_push(args),
        "range"      => crate::builtins::builtin_range(args),
        "contains"   => crate::builtins::builtin_contains(args),
        "slice"      => crate::builtins::builtin_slice(args),
        "sort"       => crate::builtins::builtin_sort(args),
        // higher-order
        "map"        => crate::builtins::builtin_map(args),
        "filter"     => crate::builtins::builtin_filter(args),
        "reduce"     => crate::builtins::builtin_reduce(args),
        // strings
        "split"      => crate::builtins::builtin_split(args),
        "join"       => crate::builtins::builtin_join(args),
        "upper"      => crate::builtins::builtin_upper(args),
        "lower"      => crate::builtins::builtin_lower(args),
        "trim"       => crate::builtins::builtin_trim(args),
        // type conversion
        "str"        => crate::builtins::builtin_to_str(args),
        "num"        => crate::builtins::builtin_to_num(args),
        "type"       => crate::builtins::builtin_type_of(args),
        // math
        "floor"      => crate::builtins::builtin_floor(args),
        "ceil"       => crate::builtins::builtin_ceil(args),
        "round"      => crate::builtins::builtin_round(args),
        "abs"        => crate::builtins::builtin_abs(args),
        "min"        => crate::builtins::builtin_min(args),
        "max"        => crate::builtins::builtin_max(args),
        // HTTP / JSON
        "http_get"   => crate::builtins::builtin_http_get(args),
        "http_post"  => crate::builtins::builtin_http_post(args),
        "json_parse" => crate::builtins::builtin_json_parse(args),
        "json_str"   => crate::builtins::builtin_json_str(args),
        // I/O
        "input"      => crate::builtins::builtin_input(args),
        // file I/O
        "read_file"   => crate::builtins::builtin_read_file(args),
        "write_file"  => crate::builtins::builtin_write_file(args),
        "append_file" => crate::builtins::builtin_append_file(args),
        // process
        "exit"        => crate::builtins::builtin_exit(args),
        "sleep"       => crate::builtins::builtin_sleep(args),
        // nil / type
        "is_nil"      => crate::builtins::builtin_is_nil(args),
        // list extras
        "concat"      => crate::builtins::builtin_concat(args),
        "flat"        => crate::builtins::builtin_flat(args),
        "first"       => crate::builtins::builtin_first(args),
        "last"        => crate::builtins::builtin_last(args),
        "pop"         => crate::builtins::builtin_pop(args),
        "set"         => crate::builtins::builtin_set(args),
        // math (extended)
        "sqrt"        => crate::builtins::builtin_sqrt(args),
        "cbrt"        => crate::builtins::builtin_cbrt(args),
        "pow"         => crate::builtins::builtin_pow(args),
        "log"         => crate::builtins::builtin_log(args),
        "log2"        => crate::builtins::builtin_log2(args),
        "log10"       => crate::builtins::builtin_log10(args),
        "exp"         => crate::builtins::builtin_exp(args),
        "sin"         => crate::builtins::builtin_sin(args),
        "cos"         => crate::builtins::builtin_cos(args),
        "tan"         => crate::builtins::builtin_tan(args),
        "asin"        => crate::builtins::builtin_asin(args),
        "acos"        => crate::builtins::builtin_acos(args),
        "atan"        => crate::builtins::builtin_atan(args),
        "atan2"       => crate::builtins::builtin_atan2(args),
        "hypot"       => crate::builtins::builtin_hypot(args),
        "clamp"       => crate::builtins::builtin_clamp(args),
        "sign"        => crate::builtins::builtin_sign(args),
        "random"      => crate::builtins::builtin_random(args),
        "rand_int"    => crate::builtins::builtin_rand_int(args),
        // strings (extended)
        "replace"     => crate::builtins::builtin_replace(args),
        "starts_with" => crate::builtins::builtin_starts_with(args),
        "ends_with"   => crate::builtins::builtin_ends_with(args),
        "index_of"    => crate::builtins::builtin_index_of(args),
        "repeat"      => crate::builtins::builtin_repeat(args),
        "char_at"     => crate::builtins::builtin_char_at(args),
        "chars"       => crate::builtins::builtin_chars(args),
        "format"      => crate::builtins::builtin_format(args),
        // list (extended)
        "reverse"     => crate::builtins::builtin_reverse(args),
        "unique"      => crate::builtins::builtin_unique(args),
        "zip"         => crate::builtins::builtin_zip(args),
        "enumerate"   => crate::builtins::builtin_enumerate(args),
        "any"         => crate::builtins::builtin_any(args),
        "all"         => crate::builtins::builtin_all(args),
        "sum"         => crate::builtins::builtin_sum(args),
        "product"     => crate::builtins::builtin_product(args),
        "find_where"  => crate::builtins::builtin_find_where(args),
        "flat_map"    => crate::builtins::builtin_flat_map(args),
        "take"        => crate::builtins::builtin_take(args),
        "skip"        => crate::builtins::builtin_skip(args),
        "count"       => crate::builtins::builtin_count(args),
        "group_by"    => crate::builtins::builtin_group_by(args),
        // dict (extended)
        "get"         => crate::builtins::builtin_get(args),
        "del"         => crate::builtins::builtin_del(args),
        "merge"       => crate::builtins::builtin_merge(args),
        "has"         => crate::builtins::builtin_has(args),
        // error helpers
        "error"       => crate::builtins::builtin_make_error(args),
        "is_error"    => crate::builtins::builtin_is_error(args),
        "ok"          => crate::builtins::builtin_ok(args),
        "zip_with"    => crate::builtins::builtin_zip_with(args),
        _ => match env.find(name) {
            Some(Value::Function(f)) => {
                // Inject self-reference into parent env so recursive calls resolve correctly.
                // f.parent_env was snapshotted at definition time and may contain only Nil
                // for the function's own name (letrec gap). We patch it here at call time.
                let mut new_parent = (*f.parent_env).clone();
                new_parent.define(name.to_string(), Value::Function(f.clone()));
                let f_with_self = Function {
                    params: f.params.clone(),
                    body: f.body.clone(),
                    parent_env: Rc::new(new_parent),
                };
                f_with_self.call(args)
            }
            _ => Value::Error(format!("unknown function: {}", name)),
        },
    }
}

fn setup_globals(env: &mut Environment) {
    env.define("true".into(),  Value::Number(1.0));
    env.define("false".into(), Value::Number(0.0));
    env.define("nil".into(),   Value::Nil);
    env.define("pi".into(),    Value::Number(std::f64::consts::PI));
    env.define("e".into(),     Value::Number(std::f64::consts::E));
    env.define("inf".into(),   Value::Number(f64::INFINITY));
    env.define("nan".into(),   Value::Number(f64::NAN));
}

/// Execute source: lex → parse → eval_expr with a fresh mutable environment.
pub fn execute(src: &str) -> Value {
    match parse_src(src) {
        Ok(ast) => {
            let mut env = Environment::new();
            setup_globals(&mut env);
            eval_expr(&mut env, &ast)
        }
        Err(e) => Value::Error(format!("parse error: {}", e)),
    }
}

pub fn new_env() -> Environment {
    let mut env = Environment::new();
    setup_globals(&mut env);
    env
}

/// Legacy wrapper kept for builtin_eval compatibility. Clones env so callers
/// see an immutable view; assignments inside eval do not escape.
pub fn eval(env: &Environment, src: &str) -> Value {
    match parse_src(src) {
        Ok(ast) => {
            let mut local = env.clone();
            eval_expr(&mut local, &ast)
        }
        Err(e) => Value::Error(format!("parse error: {}", e)),
    }
}

/// Run source in an existing mutable environment so variable assignments persist across calls.
pub fn exec_in(env: &mut Environment, src: &str) -> Value {
    match parse_src(src) {
        Ok(ast) => eval_expr(env, &ast),
        Err(e) => Value::Error(format!("parse error: {}", e)),
    }
}

/// Placeholder kept for main.rs --test flag compatibility.
pub fn run_tests() {
    println!("Running tests...");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str) -> Value { execute(src) }

    // ==
    #[test]
    fn eq_numbers_true()  { assert_eq!(run("1 == 1"), Value::Number(1.0)); }
    #[test]
    fn eq_numbers_false() { assert_eq!(run("1 == 2"), Value::Number(0.0)); }
    #[test]
    fn eq_strings_true()  { assert_eq!(run(r#""hi" == "hi""#), Value::Number(1.0)); }
    #[test]
    fn eq_strings_false() { assert_eq!(run(r#""a" == "b""#), Value::Number(0.0)); }
    #[test]
    fn eq_type_mismatch_errors() {
        assert!(matches!(run(r#"1 == "1""#), Value::Error(_)));
    }
    #[test]
    fn eq_nil_nil() {
        assert_eq!(run("d = {x: 1}\nd[\"y\"] == d[\"z\"]"), Value::Number(1.0));
    }
    #[test]
    fn eq_nil_nonnil() {
        assert_eq!(run("d = {x: 1}\nd[\"y\"] == 1"), Value::Number(0.0));
    }

    // !=
    #[test]
    fn neq_numbers_true()  { assert_eq!(run("1 != 2"), Value::Number(1.0)); }
    #[test]
    fn neq_numbers_false() { assert_eq!(run("1 != 1"), Value::Number(0.0)); }
    #[test]
    fn neq_strings_true()  { assert_eq!(run(r#""a" != "b""#), Value::Number(1.0)); }
    #[test]
    fn neq_nil_nonnil() {
        assert_eq!(run("d = {x: 1}\nd[\"y\"] != 1"), Value::Number(1.0));
    }
    #[test]
    fn neq_nil_nil() {
        assert_eq!(run("d = {x: 1}\nd[\"y\"] != d[\"z\"]"), Value::Number(0.0));
    }
    #[test]
    fn neq_strings_false() {
        assert_eq!(run(r#""a" != "a""#), Value::Number(0.0));
    }
    #[test]
    fn neq_type_mismatch_errors() {
        assert!(matches!(run(r#"1 != "x""#), Value::Error(_)));
    }

    // >=
    #[test]
    fn gte_greater() { assert_eq!(run("3 >= 2"), Value::Number(1.0)); }
    #[test]
    fn gte_equal()   { assert_eq!(run("2 >= 2"), Value::Number(1.0)); }
    #[test]
    fn gte_less()    { assert_eq!(run("1 >= 2"), Value::Number(0.0)); }
    #[test]
    fn gte_string_errors() {
        assert!(matches!(run(r#""a" >= "b""#), Value::Error(_)));
    }

    // <=
    #[test]
    fn lte_less()    { assert_eq!(run("1 <= 2"), Value::Number(1.0)); }
    #[test]
    fn lte_equal()   { assert_eq!(run("2 <= 2"), Value::Number(1.0)); }
    #[test]
    fn lte_greater() { assert_eq!(run("3 <= 2"), Value::Number(0.0)); }

    // existing operators not broken
    #[test]
    fn gt_still_works() { assert_eq!(run("3 > 2"), Value::Number(1.0)); }
    #[test]
    fn lt_still_works() { assert_eq!(run("1 < 2"), Value::Number(1.0)); }
    #[test]
    fn add_still_works() { assert_eq!(run("1 + 2"), Value::Number(3.0)); }

    // integration
    #[test]
    fn eq_in_if() {
        assert_eq!(run(r#"? "yes" == "yes" : 1 : 0"#), Value::Number(1.0));
    }
    #[test]
    fn gte_with_and() {
        assert_eq!(run("? 7 >= 5 and 7 <= 9 : 1 : 0"), Value::Number(1.0));
    }

    // new tests
    #[test] fn modulo_basic()  { assert_eq!(run("10 % 3"), Value::Number(1.0)); }
    #[test] fn modulo_float()  { assert_eq!(run("7.5 % 2.5"), Value::Number(0.0)); }
    #[test] fn modulo_zero()   { assert!(matches!(run("5 % 0"), Value::Error(_))); }
    #[test] fn power_basic()   { assert_eq!(run("2 ** 10"), Value::Number(1024.0)); }
    #[test] fn power_right_assoc() { assert_eq!(run("2 ** 3 ** 2"), Value::Number(512.0)); }
    #[test] fn neg_index_last() { assert_eq!(run("lst=[1,2,3]\nlst[-1]"), Value::Number(3.0)); }
    #[test] fn neg_index_second_last() { assert_eq!(run("lst=[1,2,3]\nlst[-2]"), Value::Number(2.0)); }
    #[test] fn nil_global()    { assert_eq!(run("nil"), Value::Nil); }
    #[test] fn pi_global()     { assert!(matches!(run("pi"), Value::Number(_))); }
    #[test] fn break_in_while() {
        assert_eq!(run("i=0\nwhile i<10\n  i=i+1\n  ?i==3:break:nil\nend\ni"), Value::Number(3.0));
    }
    #[test] fn continue_in_while() {
        assert_eq!(run("s=0\ni=0\nwhile i<5\n  i=i+1\n  ?i==3:continue:nil\n  s=s+i\nend\ns"), Value::Number(12.0));
    }
    #[test] fn return_from_fn() {
        assert_eq!(run("fn f x =>\n  ?x>0:return x*2:nil\n  99\nf 5"), Value::Number(10.0));
    }
    #[test] fn return_nil()    { assert_eq!(run("fn f => return\nf()"), Value::Nil); }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::String(s) => write!(f, "{}", s),
            Value::Nil => write!(f, "nil"),
            Value::Function(_) => write!(f, "<fn>"),
            Value::List(items) => {
                let inner: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", inner.join(", "))
            }
            Value::Dict(pairs) => {
                let inner: Vec<String> = pairs.iter()
                    .map(|(k, v)| format!("{}:{}", k, v))
                    .collect();
                write!(f, "{{{}}}", inner.join(", "))
            }
            Value::Error(msg) => write!(f, "<error: {}>", msg),
            Value::Return(v) => write!(f, "{}", v),
            Value::Break => write!(f, "<break>"),
            Value::Continue => write!(f, "<continue>"),
        }
    }
}

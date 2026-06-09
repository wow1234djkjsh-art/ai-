use std::collections::HashMap;
use std::rc::Rc;
use crate::lexer::lex;
use crate::parser::{parse, Expr};

#[derive(Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Function(Function),
    Nil,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Function(a), Value::Function(b)) => a.params == b.params && a.body == b.body,
            _ => false,
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n)   => write!(f, "Number({})", n),
            Value::String(s)   => write!(f, "String({})", s),
            Value::Function(_) => write!(f, "Function(...)"),
            Value::Nil         => write!(f, "Nil"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Environment {
    name_value: HashMap<String, Value>,
    parent: Option<Rc<Environment>>,
}

impl Default for Environment {
    fn default() -> Self { Self::new() }
}

impl Environment {
    pub fn new() -> Self {
        Environment { name_value: HashMap::new(), parent: None }
    }
    pub fn with_parent(parent: Environment) -> Self {
        Environment { name_value: HashMap::new(), parent: Some(Rc::new(parent)) }
    }
    pub fn find(&self, name: &str) -> Option<Value> {
        self.name_value.get(name).cloned()
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
        let parent = (*self.parent_env).clone();
        let mut env = Environment::with_parent(parent);
        for (param, arg) in self.params.iter().zip(args) {
            env.define(param.clone(), arg);
        }
        eval_expr(&mut env, &self.body)
    }
}

pub fn eval_expr(env: &mut Environment, expr: &Expr) -> Value {
    match expr {
        Expr::Number(n)  => Value::Number(*n),
        Expr::Str(s)     => Value::String(s.clone()),
        Expr::Ident(name) => env.find(name).unwrap_or(Value::Nil),
        Expr::Neg(inner) => match eval_expr(env, inner) {
            Value::Number(n) => Value::Number(-n),
            _ => Value::Nil,
        },
        Expr::Block(stmts) => {
            let mut last = Value::Nil;
            for stmt in stmts { last = eval_expr(env, stmt); }
            last
        }
        Expr::Assign { name, value } => {
            let val = eval_expr(env, value);
            env.define(name.clone(), val.clone());
            val
        }
        Expr::BinOp { op, left, right } => {
            let l = eval_expr(env, left);
            let r = eval_expr(env, right);
            eval_binop(*op, l, r)
        }
        Expr::FnDef { name, params, body } => {
            env.define(name.clone(), Value::Nil);  // placeholder so name is in env before capture
            let f = Value::Function(Function {
                params: params.clone(),
                body: *body.clone(),
                parent_env: Rc::new(env.clone()),
            });
            env.define(name.clone(), f.clone());
            f
        }
        Expr::Lambda { params, body } => {
            Value::Function(Function {
                params: params.clone(),
                body: *body.clone(),
                parent_env: Rc::new(env.clone()),
            })
        }
        Expr::Call { name, args } => {
            let eval_args: Vec<Value> = args.iter().map(|a| eval_expr(env, a)).collect();
            call_fn(env, name, eval_args)
        }
        Expr::If { cond, then, else_ } => {
            let truthy = match eval_expr(env, cond) {
                Value::Number(n) => n != 0.0,
                Value::String(s) => !s.is_empty(),
                Value::Nil       => false,
                Value::Function(_) => true,
            };
            if truthy { eval_expr(env, then) } else { eval_expr(env, else_) }
        }
        Expr::Pipe { left, right } => {
            let left_val = eval_expr(env, left);
            match right.as_ref() {
                Expr::Ident(name) => call_fn(env, name, vec![left_val]),
                Expr::Call { name, args } => {
                    let mut eval_args: Vec<Value> =
                        args.iter().map(|a| eval_expr(env, a)).collect();
                    eval_args.insert(0, left_val);
                    call_fn(env, name, eval_args)
                }
                _ => Value::Nil,
            }
        }
        Expr::Each { items, func } => {
            let func_val = eval_expr(env, func);
            let mut last = Value::Nil;
            for item in items {
                let item_val = eval_expr(env, item);
                last = match &func_val {
                    Value::Function(f) => f.call(vec![item_val]),
                    _ => Value::Nil,
                };
            }
            last
        }
    }
}

fn eval_binop(op: char, left: Value, right: Value) -> Value {
    match (op, &left, &right) {
        ('+', Value::Number(l), Value::Number(r)) => Value::Number(l + r),
        ('-', Value::Number(l), Value::Number(r)) => Value::Number(l - r),
        ('*', Value::Number(l), Value::Number(r)) => Value::Number(l * r),
        ('/', Value::Number(l), Value::Number(r)) => {
            if *r == 0.0 { Value::Nil } else { Value::Number(l / r) }
        }
        ('>', Value::Number(l), Value::Number(r)) => Value::Number(if l > r { 1.0 } else { 0.0 }),
        ('<', Value::Number(l), Value::Number(r)) => Value::Number(if l < r { 1.0 } else { 0.0 }),
        ('+', Value::String(l), Value::String(r)) => Value::String(l.clone() + r),
        _ => Value::Nil,
    }
}

fn call_fn(env: &mut Environment, name: &str, args: Vec<Value>) -> Value {
    match name {
        "print" => crate::builtins::builtin_print(args),
        "eval"  => crate::builtins::builtin_eval(env, args),
        "model" => crate::builtins::model(env, args),
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
            _ => Value::Nil,
        }
    }
}

/// Execute source: lex → parse → eval_expr with a fresh mutable environment.
pub fn execute(src: &str) -> Value {
    let tokens = lex(src);
    match parse(&tokens) {
        Ok(ast) => { let mut env = Environment::new(); eval_expr(&mut env, &ast) }
        Err(_)  => Value::Nil,
    }
}

/// Legacy wrapper kept for builtin_eval compatibility. Clones env so callers
/// see an immutable view; assignments inside eval do not escape.
pub fn eval(env: &Environment, src: &str) -> Value {
    let tokens = lex(src);
    match parse(&tokens) {
        Ok(ast) => { let mut local = env.clone(); eval_expr(&mut local, &ast) }
        Err(_)  => Value::Nil,
    }
}

/// Run source in an existing mutable environment so variable assignments persist across calls.
pub fn exec_in(env: &mut Environment, src: &str) -> Value {
    let tokens = lex(src);
    match parse(&tokens) {
        Ok(ast) => eval_expr(env, &ast),
        Err(e)  => { eprintln!("parse error: {}", e); Value::Nil }
    }
}

/// Placeholder kept for main.rs --test flag compatibility.
pub fn run_tests() {
    println!("Running tests...");
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 { write!(f, "{}", *n as i64) } else { write!(f, "{}", n) }
            }
            Value::String(s)   => write!(f, "{}", s),
            Value::Nil         => write!(f, "nil"),
            Value::Function(_) => write!(f, "<fn>"),
        }
    }
}

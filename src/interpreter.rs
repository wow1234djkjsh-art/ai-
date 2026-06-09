use std::collections::HashMap;
use std::rc::Rc;

// Runtime value type
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
            Value::Number(n) => write!(f, "Number({})", n),
            Value::String(s) => write!(f, "String({})", s),
            Value::Function(fun) => write!(f, "Function({:?})", fun),
            Value::Nil => write!(f, "Nil"),
        }
    }
}

// Environment for value lookups (supports nested scopes)
#[derive(Clone, Debug)]
pub struct Environment {
    name_value: HashMap<String, Value>,
    parent: Option<Rc<Environment>>,
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

// Function type with parent environment for closures
#[derive(Clone, Debug)]
pub struct Function {
    pub params: Vec<String>,
    pub body: String,
    pub parent_env: Rc<Environment>,
}
impl Function {
    pub fn call(&self, args: Vec<Value>) -> Value {
        let parent_map = self.parent_env.name_value.clone();
        let mut env = Environment::new();
        for (k, v) in parent_map {
            env.define(k, v);
        }
        for (param, arg) in self.params.iter().zip(args) {
            env.define(param.clone(), arg);
        }
        eval(&env, &self.body)
    }
}

// Evaluate an expression string
pub fn eval(env: &Environment, expr: &str) -> Value {
    let trimmed = expr.trim();
    // Number literal
    if let Ok(n) = trimmed.parse::<f64>() {
        return Value::Number(n);
    }
    // String literal (quoted)
    if trimmed.starts_with('"') && trimmed.ends_with('"') {
        let inner = &trimmed[1..trimmed.len() - 1];
        return Value::String(inner.to_string());
    }
    // Simple binary operators
    // helper closure to resolve a term: try number literal, then variable lookup
    let resolve = |term: &str| -> Option<f64> {
        if let Ok(n) = term.parse::<f64>() {
            Some(n)
        } else if let Some(Value::Number(n)) = env.find(term) {
            Some(n)
        } else {
            None
        }
    };

    if let Some(pos) = trimmed.find('+') {
        let left = trimmed[..pos].trim();
        let right = trimmed[pos + 1..].trim();
        if let (Some(l), Some(r)) = (resolve(left), resolve(right)) {
            return Value::Number(l + r);
        }
    }
    if let Some(pos) = trimmed.find('-') {
        let left = trimmed[..pos].trim();
        let right = trimmed[pos + 1..].trim();
        if let (Some(l), Some(r)) = (resolve(left), resolve(right)) {
            return Value::Number(l - r);
        }
    }
    if let Some(pos) = trimmed.find('*') {
        let left = trimmed[..pos].trim();
        let right = trimmed[pos + 1..].trim();
        if let (Some(l), Some(r)) = (resolve(left), resolve(right)) {
            return Value::Number(l * r);
        }
    }
    if let Some(pos) = trimmed.find('/') {
        let left = trimmed[..pos].trim();
        let right = trimmed[pos + 1..].trim();
        if let (Some(l), Some(r)) = (resolve(left), resolve(right)) {
            return Value::Number(l / r);
        }
    }
    // Variable lookup
    if let Some(v) = env.find(trimmed) {
        return v;
    }
    Value::Nil
}

// Execute source code
pub fn execute(code: &str) -> Value {
    let env = Environment::new();
    eval(&env, code)
}

// Run tests placeholder
pub fn run_tests() {
    println!("Running tests...");
}
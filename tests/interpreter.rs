use c_dsl::interpreter::{execute, Value};

#[test]
fn test_number_literal() {
    assert_eq!(execute("42"), Value::Number(42.0));
}

#[test]
fn test_string_literal() {
    assert_eq!(execute("\"hello\""), Value::String("hello".into()));
}

#[test]
fn test_arithmetic() {
    assert_eq!(execute("2+3"),   Value::Number(5.0));
    assert_eq!(execute("10-4"),  Value::Number(6.0));
    assert_eq!(execute("6*7"),   Value::Number(42.0));
    assert_eq!(execute("20/4"),  Value::Number(5.0));
}

#[test]
fn test_variable_assign_and_lookup() {
    assert_eq!(execute("x=10;x"), Value::Number(10.0));
}

#[test]
fn test_fn_def_and_call() {
    assert_eq!(execute("fn double x=>x*2;double 5"), Value::Number(10.0));
}

#[test]
fn test_fn_call_paren() {
    assert_eq!(execute("fn add a,b=>a+b;add(3,4)"), Value::Number(7.0));
}

#[test]
fn test_conditional_true() {
    assert_eq!(execute("x=5;?x>3:x*2:0"), Value::Number(10.0));
}

#[test]
fn test_conditional_false() {
    assert_eq!(execute("x=1;?x>3:x*2:0"), Value::Number(0.0));
}

#[test]
fn test_pipe_to_builtin() {
    // print returns its argument; check the value comes through
    assert_eq!(execute("fn double x=>x*2;3|double"), Value::Number(6.0));
}

#[test]
fn test_pipe_chain() {
    assert_eq!(execute("fn double x=>x*2;3|double|double"), Value::Number(12.0));
}

#[test]
fn test_neg() {
    assert_eq!(execute("x=5;-x"), Value::Number(-5.0));
}

#[test]
fn test_each() {
    assert_eq!(execute("each 1,2,3:fn x=>x*2"), Value::Number(6.0));
}

#[test]
fn test_closure_capture() {
    // function body should capture variables from defining scope
    assert_eq!(execute("x=10;fn add_x n=>n+x;add_x 5"), Value::Number(15.0));
}

#[test]
fn test_string_concat() {
    assert_eq!(execute("\"hello\"+\"world\""), Value::String("helloworld".into()));
}

#[test]
fn test_div_by_zero() {
    assert_eq!(execute("1/0"), Value::Nil);
}

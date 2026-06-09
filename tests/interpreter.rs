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
    assert_eq!(execute("2+3"), Value::Number(5.0));
    assert_eq!(execute("10-4"), Value::Number(6.0));
    assert_eq!(execute("6*7"), Value::Number(42.0));
    assert_eq!(execute("20/4"), Value::Number(5.0));
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
    assert_eq!(
        execute("fn double x=>x*2;3|double|double"),
        Value::Number(12.0)
    );
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
    assert_eq!(
        execute("\"hello\"+\"world\""),
        Value::String("helloworld".into())
    );
}

#[test]
fn test_div_by_zero() {
    assert_eq!(execute("1/0"), Value::Nil);
}

#[test]
fn test_print_pipeline() {
    // 5 piped through print — value should come through unchanged
    let result = execute("5|print");
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_recursive_function() {
    // fact(3) = 3 * 2 * 1 = 6
    assert_eq!(
        execute("fn fact n=>?n>0:n*fact n-1:1;fact 3"),
        Value::Number(6.0)
    );
}

#[test]
fn test_conditional_in_fn_body() {
    assert_eq!(execute("fn abs x=>?x>0:x:0-x;abs(5)"), Value::Number(5.0));
    assert_eq!(
        execute("fn abs x=>?x>0:x:0-x;n=0-3;abs n"),
        Value::Number(3.0)
    );
}

#[test]
fn test_lambda_with_conditional() {
    // each item: if >1 double it, else keep
    assert_eq!(execute("each 1,2,3:fn x=>?x>1:x*2:x"), Value::Number(6.0));
}

#[test]
fn test_list_indexing() {
    assert_eq!(execute("[10,20,30][1]"), Value::Number(20.0));
}

#[test]
fn test_list_assign_and_index() {
    assert_eq!(execute("lst=[1,2,3];lst[0]"), Value::Number(1.0));
}

#[test]
fn test_dict_literal_lookup() {
    assert_eq!(execute("{x:42}[\"x\"]"), Value::Number(42.0));
}

#[test]
fn test_dict_assign_and_lookup() {
    assert_eq!(
        execute("d={name:\"Alice\"};d[\"name\"]"),
        Value::String("Alice".into())
    );
}

#[test]
fn test_index_out_of_bounds() {
    assert_eq!(execute("[1,2][9]"), Value::Nil);
}

#[test]
fn test_dict_missing_key() {
    assert_eq!(execute("{a:1}[\"b\"]"), Value::Nil);
}

#[test]
fn test_list_in_fn_call() {
    assert_eq!(execute("fn first lst=>lst[0];first [7,8,9]"), Value::Number(7.0));
}

#[test]
fn test_list_negative_index_returns_nil() {
    assert_eq!(execute("[1,2,3][-1]"), Value::Nil);
}

#[test]
fn test_list_fractional_index_returns_nil() {
    assert_eq!(execute("[10,20,30][1.5]"), Value::Nil);
}

#[test]
fn test_dict_equality_order_independent() {
    // {a:1, b:2} should equal {b:2, a:1}
    assert_eq!(execute("{a:1,b:2}"), execute("{b:2,a:1}"));
}

#[test]
fn test_logical_and_true() {
    assert_eq!(execute("1>0 and 2>0"), Value::Number(1.0));
}

#[test]
fn test_logical_and_false() {
    assert_eq!(execute("1>0 and 0>1"), Value::Number(0.0));
}

#[test]
fn test_logical_or_true() {
    assert_eq!(execute("0>1 or 1>0"), Value::Number(1.0));
}

#[test]
fn test_logical_or_false() {
    assert_eq!(execute("0>1 or 0>1"), Value::Number(0.0));
}

#[test]
fn test_logical_not_true() {
    assert_eq!(execute("not 0>1"), Value::Number(1.0));
}

#[test]
fn test_logical_not_false() {
    assert_eq!(execute("not 1>0"), Value::Number(0.0));
}

#[test]
fn test_and_false_lhs_returns_zero() {
    assert_eq!(execute("0 and 1>0"), Value::Number(0.0));
}

#[test]
fn test_or_true_lhs_returns_one() {
    assert_eq!(execute("1 or 0>1"), Value::Number(1.0));
}

#[test]
fn test_short_circuit_and_skips_rhs() {
    // if short-circuit works: x stays 0 (rhs not evaluated)
    // if no short-circuit: x becomes 1 (rhs evaluated)
    assert_eq!(execute("x=0; 0 and x=1; x"), Value::Number(0.0));
}

#[test]
fn test_short_circuit_or_skips_rhs() {
    // if short-circuit works: x stays 0 (rhs not evaluated)
    // if no short-circuit: x becomes 1 (rhs evaluated)
    assert_eq!(execute("x=0; 1 or x=1; x"), Value::Number(0.0));
}

#[test]
fn test_logical_precedence_not_and() {
    // not 0 and 1  →  (not 0) and 1  →  1 and 1  →  1
    assert_eq!(execute("not 0 and 1"), Value::Number(1.0));
}

#[test]
fn test_logical_precedence_or_and() {
    // 0 or 1 and 1  →  0 or (1 and 1)  →  0 or 1  →  1
    assert_eq!(execute("0 or 1 and 1"), Value::Number(1.0));
}

#[test]
fn test_line_continuation() {
    assert_eq!(execute("1\\\n+2"), Value::Number(3.0));
}

#[test]
fn test_logical_in_conditional() {
    assert_eq!(execute("x=5;?x>3 and x<10:1:0"), Value::Number(1.0));
}

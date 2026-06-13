use c_dsl::interpreter::{execute, Value};
fn run(src: &str) -> Value { execute(src) }

// ── Math ──────────────────────────────────────────────────────────────────

#[test] fn sqrt_4() { assert_eq!(run("sqrt 4"), Value::Number(2.0)); }
#[test] fn pow_fn() { assert_eq!(run("pow 2,10"), Value::Number(1024.0)); }
#[test] fn log10_100() { assert_eq!(run("floor log10 100"), Value::Number(2.0)); }
#[test] fn sin_zero() { assert_eq!(run("sin 0"), Value::Number(0.0)); }
#[test] fn cos_zero() { assert_eq!(run("cos 0"), Value::Number(1.0)); }
#[test] fn clamp_mid() { assert_eq!(run("clamp 5,1,10"), Value::Number(5.0)); }
// Negative literal needs parens so the parser doesn't treat '-' as subtraction
#[test] fn clamp_lo() { assert_eq!(run("clamp(-5,0,10)"), Value::Number(0.0)); }
#[test] fn sign_pos() { assert_eq!(run("sign 7"), Value::Number(1.0)); }
// Negative literal needs parens
#[test] fn sign_neg() { assert_eq!(run("sign(-3)"), Value::Number(-1.0)); }
#[test] fn sign_zero() { assert_eq!(run("sign 0"), Value::Number(0.0)); }
#[test] fn random_range() {
    let v = run("random()");
    if let Value::Number(n) = v { assert!(n >= 0.0 && n < 1.0); } else { panic!("not a number"); }
}
#[test] fn rand_int_range() {
    let v = run("rand_int 10");
    if let Value::Number(n) = v { assert!(n >= 0.0 && n < 10.0 && n.fract() == 0.0); } else { panic!(); }
}

// ── String ────────────────────────────────────────────────────────────────

#[test] fn replace_basic() { assert_eq!(run(r#"replace "hello world" "world" "Rust""#), Value::String("hello Rust".into())); }
#[test] fn starts_with_yes() { assert_eq!(run(r#"starts_with "hello" "he""#), Value::Number(1.0)); }
#[test] fn starts_with_no()  { assert_eq!(run(r#"starts_with "hello" "lo""#), Value::Number(0.0)); }
#[test] fn ends_with_yes() { assert_eq!(run(r#"ends_with "hello" "lo""#), Value::Number(1.0)); }
#[test] fn index_of_str() { assert_eq!(run(r#"index_of "hello" "ll""#), Value::Number(2.0)); }
#[test] fn index_of_missing() { assert_eq!(run(r#"index_of "hello" "xyz""#), Value::Number(-1.0)); }
#[test] fn repeat_str() { assert_eq!(run(r#"repeat "ab" 3"#), Value::String("ababab".into())); }
#[test] fn char_at_pos() { assert_eq!(run(r#"char_at "hello" 1"#), Value::String("e".into())); }
// Negative index needs parens; '-' is not a literal-start token
#[test] fn char_at_neg() { assert_eq!(run(r#"char_at("hello",-1)"#), Value::String("o".into())); }
#[test] fn chars_len() { assert_eq!(run(r#"len chars "hello""#), Value::Number(5.0)); }
#[test] fn format_basic() { assert_eq!(run(r#"format "hello {}" "world""#), Value::String("hello world".into())); }

// ── List ──────────────────────────────────────────────────────────────────

#[test] fn reverse_list() { assert_eq!(run("reverse [1,2,3]"), Value::List(vec![Value::Number(3.0),Value::Number(2.0),Value::Number(1.0)])); }
#[test] fn unique_list() { assert_eq!(run("len unique [1,2,1,3,2]"), Value::Number(3.0)); }
// Two adjacent list literals cause apply_subscript ambiguity; use paren form
#[test] fn zip_lists() { assert_eq!(run("len(zip([1,2],[3,4]))"), Value::Number(2.0)); }
#[test] fn enumerate_list() { assert_eq!(run("len enumerate [10,20,30]"), Value::Number(3.0)); }
#[test] fn any_true() { assert_eq!(run("any [1,2,3] fn x=>x>2"), Value::Number(1.0)); }
#[test] fn any_false() { assert_eq!(run("any [1,2,3] fn x=>x>5"), Value::Number(0.0)); }
#[test] fn all_true() { assert_eq!(run("all [1,2,3] fn x=>x>0"), Value::Number(1.0)); }
#[test] fn all_false() { assert_eq!(run("all [1,2,3] fn x=>x>1"), Value::Number(0.0)); }
#[test] fn sum_list() { assert_eq!(run("sum [1,2,3,4,5]"), Value::Number(15.0)); }
#[test] fn product_list() { assert_eq!(run("product [1,2,3,4]"), Value::Number(24.0)); }
#[test] fn take_list() { assert_eq!(run("take [1,2,3,4,5] 3"), Value::List(vec![Value::Number(1.0),Value::Number(2.0),Value::Number(3.0)])); }
#[test] fn skip_list() { assert_eq!(run("skip [1,2,3,4,5] 3"), Value::List(vec![Value::Number(4.0),Value::Number(5.0)])); }
#[test] fn count_list() { assert_eq!(run("count [1,2,3,4,5] fn x=>x>2"), Value::Number(3.0)); }
#[test] fn find_where_found() { assert_eq!(run("find_where [1,2,3] fn x=>x>1"), Value::Number(2.0)); }
#[test] fn find_where_missing() { assert_eq!(run("find_where [1,2,3] fn x=>x>5"), Value::Nil); }
#[test] fn flat_map_list() { assert_eq!(run("flat_map [1,2,3] fn x=>[x,x*10]"), Value::List(vec![Value::Number(1.0),Value::Number(10.0),Value::Number(2.0),Value::Number(20.0),Value::Number(3.0),Value::Number(30.0)])); }
#[test] fn index_of_list() { assert_eq!(run("index_of [10,20,30] 20"), Value::Number(1.0)); }
#[test] fn index_of_list_missing() { assert_eq!(run("index_of [10,20,30] 99"), Value::Number(-1.0)); }

// ── Dict ──────────────────────────────────────────────────────────────────

#[test] fn get_found() { assert_eq!(run(r#"get {"a":1} "a" 0"#), Value::Number(1.0)); }
#[test] fn get_default() { assert_eq!(run(r#"get {"a":1} "b" 99"#), Value::Number(99.0)); }
#[test] fn has_yes() { assert_eq!(run(r#"has {"a":1} "a""#), Value::Number(1.0)); }
#[test] fn has_no() { assert_eq!(run(r#"has {"a":1} "b""#), Value::Number(0.0)); }
#[test] fn del_key() { assert_eq!(run(r#"len keys del {"a":1,"b":2} "a""#), Value::Number(1.0)); }
// Parenthesized form required: merge + string literal collide in space-call parsing
#[test] fn merge_dicts() { assert_eq!(run(r#"get(merge({"a":1},{"b":2}),"b",0)"#), Value::Number(2.0)); }

// ── Other ─────────────────────────────────────────────────────────────────

#[test] fn make_error() { assert!(matches!(run(r#"error "oops""#), Value::Error(_))); }
#[test] fn is_error_yes() { assert_eq!(run(r#"is_error error("x")"#), Value::Number(1.0)); }
#[test] fn is_error_no() { assert_eq!(run("is_error 42"), Value::Number(0.0)); }
#[test] fn ok_passes() { assert_eq!(run("ok 42 0"), Value::Number(42.0)); }
#[test] fn ok_fallback() { assert_eq!(run(r#"ok error("x") 99"#), Value::Number(99.0)); }

// ── Pipe + error suppression (Fix 1) ─────────────────────────────────────

#[test] fn is_error_pipe() {
    // error piped into is_error should return 1, not propagate
    assert_eq!(run(r#"error("x") | is_error"#), Value::Number(1.0));
}
#[test] fn ok_pipe() {
    // error piped into ok(default) should return the default
    assert_eq!(run(r#"error("x") | ok(42)"#), Value::Number(42.0));
}

// ── clamp bad range (Fix 2) ───────────────────────────────────────────────

#[test] fn clamp_bad_range() {
    assert!(matches!(run("clamp(5,10,3)"), Value::Error(_)));
}

// ── take/skip negative n (Fix 4) ─────────────────────────────────────────

#[test] fn take_negative() {
    assert!(matches!(run("take([1,2,3],-1)"), Value::Error(_)));
}
#[test] fn skip_negative() {
    assert!(matches!(run("skip([1,2,3],-1)"), Value::Error(_)));
}

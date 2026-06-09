use c_dsl::lexer::lex;
use c_dsl::parser::{parse, Expr};

fn p(src: &str) -> Expr {
    parse(&lex(src)).expect("parse failed")
}

#[test]
fn test_parse_number() {
    assert_eq!(p("42"), Expr::Block(vec![Expr::Number(42.0)]));
}

#[test]
fn test_parse_assign() {
    assert_eq!(
        p("x=5"),
        Expr::Block(vec![Expr::Assign {
            name: "x".into(),
            value: Box::new(Expr::Number(5.0))
        }])
    );
}

#[test]
fn test_parse_binop_add() {
    assert_eq!(
        p("1+2"),
        Expr::Block(vec![Expr::BinOp {
            op: '+',
            left: Box::new(Expr::Number(1.0)),
            right: Box::new(Expr::Number(2.0))
        }])
    );
}

#[test]
fn test_parse_neg() {
    assert_eq!(
        p("-x"),
        Expr::Block(vec![Expr::Neg(Box::new(Expr::Ident("x".into())))])
    );
}

#[test]
fn test_parse_fn_def() {
    assert_eq!(
        p("fn add a,b=>a+b"),
        Expr::Block(vec![Expr::FnDef {
            name: "add".into(),
            params: vec!["a".into(), "b".into()],
            body: Box::new(Expr::BinOp {
                op: '+',
                left: Box::new(Expr::Ident("a".into())),
                right: Box::new(Expr::Ident("b".into())),
            }),
        }])
    );
}

#[test]
fn test_parse_call_space() {
    assert_eq!(
        p("add 1,2"),
        Expr::Block(vec![Expr::Call {
            name: "add".into(),
            args: vec![Expr::Number(1.0), Expr::Number(2.0)]
        }])
    );
}

#[test]
fn test_parse_call_paren() {
    assert_eq!(
        p("add(1,2)"),
        Expr::Block(vec![Expr::Call {
            name: "add".into(),
            args: vec![Expr::Number(1.0), Expr::Number(2.0)]
        }])
    );
}

#[test]
fn test_parse_if() {
    assert_eq!(
        p("?x>0:x:0"),
        Expr::Block(vec![Expr::If {
            cond: Box::new(Expr::BinOp {
                op: '>',
                left: Box::new(Expr::Ident("x".into())),
                right: Box::new(Expr::Number(0.0))
            }),
            then: Box::new(Expr::Ident("x".into())),
            else_: Box::new(Expr::Number(0.0)),
        }])
    );
}

#[test]
fn test_parse_pipe() {
    assert_eq!(
        p("add 1,2|print"),
        Expr::Block(vec![Expr::Pipe {
            left: Box::new(Expr::Call {
                name: "add".into(),
                args: vec![Expr::Number(1.0), Expr::Number(2.0)]
            }),
            right: Box::new(Expr::Ident("print".into())),
        }])
    );
}

#[test]
fn test_parse_block() {
    assert_eq!(
        p("x=1;y=2"),
        Expr::Block(vec![
            Expr::Assign {
                name: "x".into(),
                value: Box::new(Expr::Number(1.0))
            },
            Expr::Assign {
                name: "y".into(),
                value: Box::new(Expr::Number(2.0))
            },
        ])
    );
}

#[test]
fn test_parse_each() {
    assert_eq!(
        p("each 1,2:fn x=>x"),
        Expr::Block(vec![Expr::Each {
            items: vec![Expr::Number(1.0), Expr::Number(2.0)],
            func: Box::new(Expr::Lambda {
                params: vec!["x".into()],
                body: Box::new(Expr::Ident("x".into())),
            }),
        }])
    );
}

#[test]
fn test_parse_if_pipe_in_then() {
    // then-arm containing a pipe must parse correctly
    assert_eq!(
        p("?x>0:a|b:c"),
        Expr::Block(vec![Expr::If {
            cond: Box::new(Expr::BinOp {
                op: '>',
                left: Box::new(Expr::Ident("x".into())),
                right: Box::new(Expr::Number(0.0))
            }),
            then: Box::new(Expr::Pipe {
                left: Box::new(Expr::Ident("a".into())),
                right: Box::new(Expr::Ident("b".into()))
            }),
            else_: Box::new(Expr::Ident("c".into())),
        }])
    );
}

#[test]
fn test_parse_double_neg() {
    assert_eq!(
        p("--x"),
        Expr::Block(vec![Expr::Neg(Box::new(Expr::Neg(Box::new(Expr::Ident(
            "x".into()
        )))))])
    );
}

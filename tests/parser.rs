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

#[test]
fn test_parse_list_literal() {
    assert_eq!(
        p("[1,2,3]"),
        Expr::Block(vec![Expr::List(vec![
            Expr::Number(1.0), Expr::Number(2.0), Expr::Number(3.0),
        ])])
    );
}

#[test]
fn test_parse_empty_list() {
    assert_eq!(p("[]"), Expr::Block(vec![Expr::List(vec![])]));
}

#[test]
fn test_parse_dict_literal() {
    assert_eq!(
        p("{a:1}"),
        Expr::Block(vec![Expr::Dict(vec![("a".to_string(), Expr::Number(1.0))])])
    );
}

#[test]
fn test_parse_index_ident() {
    assert_eq!(
        p("lst[0]"),
        Expr::Block(vec![Expr::Index {
            object: Box::new(Expr::Ident("lst".into())),
            index:  Box::new(Expr::Number(0.0)),
        }])
    );
}

#[test]
fn test_parse_index_dict() {
    assert_eq!(
        p("d[\"k\"]"),
        Expr::Block(vec![Expr::Index {
            object: Box::new(Expr::Ident("d".into())),
            index:  Box::new(Expr::Str("k".into())),
        }])
    );
}

#[test]
fn test_parse_index_chain() {
    assert_eq!(
        p("lst[0][1]"),
        Expr::Block(vec![Expr::Index {
            object: Box::new(Expr::Index {
                object: Box::new(Expr::Ident("lst".into())),
                index:  Box::new(Expr::Number(0.0)),
            }),
            index: Box::new(Expr::Number(1.0)),
        }])
    );
}

#[test]
fn test_parse_inline_list_index() {
    assert_eq!(
        p("[10,20][1]"),
        Expr::Block(vec![Expr::Index {
            object: Box::new(Expr::List(vec![Expr::Number(10.0), Expr::Number(20.0)])),
            index:  Box::new(Expr::Number(1.0)),
        }])
    );
}

#[test]
fn test_parse_and() {
    assert_eq!(
        p("a and b"),
        Expr::Block(vec![Expr::And {
            left:  Box::new(Expr::Ident("a".into())),
            right: Box::new(Expr::Ident("b".into())),
        }])
    );
}

#[test]
fn test_parse_or() {
    assert_eq!(
        p("a or b"),
        Expr::Block(vec![Expr::Or {
            left:  Box::new(Expr::Ident("a".into())),
            right: Box::new(Expr::Ident("b".into())),
        }])
    );
}

#[test]
fn test_parse_not() {
    assert_eq!(
        p("not a"),
        Expr::Block(vec![Expr::Not(Box::new(Expr::Ident("a".into())))])
    );
}

#[test]
fn test_parse_or_binds_looser_than_and() {
    // a or b and c  →  a or (b and c)
    assert_eq!(
        p("a or b and c"),
        Expr::Block(vec![Expr::Or {
            left: Box::new(Expr::Ident("a".into())),
            right: Box::new(Expr::And {
                left:  Box::new(Expr::Ident("b".into())),
                right: Box::new(Expr::Ident("c".into())),
            }),
        }])
    );
}

#[test]
fn test_parse_not_binds_tighter_than_and() {
    // not a and b  →  (not a) and b
    assert_eq!(
        p("not a and b"),
        Expr::Block(vec![Expr::And {
            left:  Box::new(Expr::Not(Box::new(Expr::Ident("a".into())))),
            right: Box::new(Expr::Ident("b".into())),
        }])
    );
}

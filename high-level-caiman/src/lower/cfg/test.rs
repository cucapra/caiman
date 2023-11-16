use std::collections::BTreeMap;

use crate::{
    error::Info,
    parse::ast::{NestedExpr, SchedExpr, SchedLiteral, SchedStmt, SchedTerm},
};

use super::Cfg;

#[test]
fn cfg_gen() {
    let stmts = vec![
        SchedStmt::If {
            info: Info::default(),
            guard: NestedExpr::Term(SchedTerm::Var {
                info: Info::default(),
                name: "x".to_string(),
                tag: None,
            }),
            true_block: vec![SchedStmt::If {
                info: Info::default(),
                guard: NestedExpr::Term(SchedTerm::Var {
                    info: Info::default(),
                    name: "x".to_string(),
                    tag: None,
                }),
                true_block: vec![SchedStmt::Decl {
                    info: Info::default(),
                    lhs: vec![("y".to_string(), None)],
                    expr: SchedExpr::Term(SchedTerm::Lit {
                        info: Info::default(),
                        lit: SchedLiteral::Int(2.to_string()),
                        tag: None,
                    }),
                    is_const: true,
                }],
                false_block: vec![SchedStmt::Decl {
                    info: Info::default(),
                    lhs: vec![("x".to_string(), None)],
                    expr: SchedExpr::Term(SchedTerm::Lit {
                        info: Info::default(),
                        lit: SchedLiteral::Int(4.to_string()),
                        tag: None,
                    }),
                    is_const: true,
                }],
            }],
            false_block: vec![SchedStmt::Return(
                Info::default(),
                SchedExpr::Term(SchedTerm::Var {
                    info: Info::default(),
                    name: "x".to_string(),
                    tag: None,
                }),
            )],
        },
        SchedStmt::Assign {
            info: Info::default(),
            lhs: String::from("x"),
            rhs: SchedExpr::Term(SchedTerm::Lit {
                info: Info::default(),
                lit: SchedLiteral::Int(5.to_string()),
                tag: None,
            }),
        },
    ];
    let cfg = Cfg::new("main", stmts);
    let mut ordered_graph = BTreeMap::new();
    for (id, edge) in cfg.graph {
        ordered_graph.insert(id, edge);
    }
    assert_eq!(
        format!("{ordered_graph:?}"),
        "{0: None, 1: Select { true_branch: 3, false_branch: 7 }, 2: Next(0), \
            3: Select { true_branch: 5, false_branch: 6 }, 4: Next(2), \
            5: Next(4), 6: Next(4), 7: Next(0)}"
    );
}

#[test]
fn if_gen() {
    let stmts = vec![SchedStmt::If {
        guard: NestedExpr::Term(SchedTerm::Var {
            info: Info::default(),
            name: "x".to_string(),
            tag: None,
        }),
        info: Info::default(),
        true_block: vec![SchedStmt::Return(
            Info::default(),
            SchedExpr::Term(SchedTerm::Var {
                info: Info::default(),
                name: "x".to_string(),
                tag: None,
            }),
        )],
        false_block: vec![SchedStmt::Block(
            Info::default(),
            vec![SchedStmt::Return(
                Info::default(),
                SchedExpr::Term(SchedTerm::Var {
                    info: Info::default(),
                    name: "x".to_string(),
                    tag: None,
                }),
            )],
        )],
    }];
    let cfg = Cfg::new("main", stmts);
    let mut ordered_graph = BTreeMap::new();
    for (id, edge) in cfg.graph {
        ordered_graph.insert(id, edge);
    }
    assert_eq!(
        format!("{ordered_graph:?}"),
        "{0: None, 1: Select { true_branch: 3, false_branch: 4 }, \
            3: Next(0), 4: Next(0)}"
    );
}

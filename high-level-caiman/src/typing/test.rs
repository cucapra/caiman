use std::collections::BTreeMap;

use crate::typing::types::{ClassName, UTypeName, VarName};

use super::unification::{Constraint, Env, Kind};

#[derive(Debug, PartialEq, Eq, Clone)]
enum BaseType {
    I32,
    I64,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum CompoundType {
    Int,
    Array,
    Tuple,
}

impl Kind for BaseType {}
impl Kind for CompoundType {}

#[test]
fn test_unification() {
    let mut env: Env<CompoundType, BaseType> = Env::new();
    env.new_type_if_absent(&VarName::from("v"));
    env.add_constraint(
        &"v".into(),
        &Constraint::Term(CompoundType::Int, vec![Constraint::Atom(BaseType::I32)]),
    )
    .unwrap();
    assert_eq!(
        env.get_type(VarName::from("v").as_metavar()).unwrap(),
        Constraint::Term(CompoundType::Int, vec![Constraint::Atom(BaseType::I32)])
    );
    env.new_type_if_absent(&VarName::from("w"));
    env.new_type_if_absent(&VarName::from("a"));
    env.add_constraint(&"a".into(), &Constraint::Var("v".to_string()))
        .unwrap();
    env.add_constraint(&"w".into(), &Constraint::Var("a".to_string()))
        .unwrap();
    assert_eq!(
        env.get_type(VarName::from("w").as_metavar()),
        env.get_type(VarName::from("v").as_metavar())
    );
    assert_eq!(
        env.get_type(VarName::from("w").as_metavar()).unwrap(),
        Constraint::Term(CompoundType::Int, vec![Constraint::Atom(BaseType::I32)])
    );

    env.new_type_if_absent(&VarName::from("alpha"));
    env.new_type_if_absent(&VarName::from("beta"));
    let t = env.new_temp_type();
    env.add_constraint(
        &t,
        &Constraint::Term(
            CompoundType::Tuple,
            vec![
                Constraint::Term(
                    CompoundType::Array,
                    vec![Constraint::Term(
                        CompoundType::Int,
                        vec![Constraint::Atom(BaseType::I32)],
                    )],
                ),
                Constraint::Term(
                    CompoundType::Array,
                    vec![Constraint::Var("alpha".to_string())],
                ),
            ],
        ),
    )
    .unwrap();
    env.add_constraint(
        &t,
        &Constraint::Term(
            CompoundType::Tuple,
            vec![
                Constraint::Var("alpha".to_string()),
                Constraint::Var("beta".to_string()),
            ],
        ),
    )
    .unwrap();
    assert_eq!(
        env.get_type(VarName::from("beta").as_metavar()).unwrap(),
        Constraint::Term(
            CompoundType::Array,
            vec![Constraint::Term(
                CompoundType::Array,
                vec![Constraint::Term(
                    CompoundType::Int,
                    vec![Constraint::Atom(BaseType::I32)]
                )]
            )]
        )
    );
}

#[test]
fn test_fails() {
    let mut env: Env<CompoundType, BaseType> = Env::new();
    env.new_type_if_absent(&VarName::from("v"));
    env.add_constraint(
        &"v".into(),
        &Constraint::Term(CompoundType::Int, vec![Constraint::Atom(BaseType::I32)]),
    )
    .unwrap();
    env.add_constraint(
        &"v".into(),
        &Constraint::Term(CompoundType::Int, vec![Constraint::Atom(BaseType::I64)]),
    )
    .unwrap_err();
}

#[test]
fn test_polymorphism() {
    let mut env: Env<CompoundType, BaseType> = Env::new();
    env.new_type_if_absent(&VarName::from("v"));
    let t = env.new_temp_type();
    env.add_constraint(
        &"v".into(),
        &Constraint::Term(CompoundType::Int, vec![Constraint::Var(t.into_string())]),
    )
    .unwrap();
    let t2 = env.new_temp_type();
    env.add_constraint(
        &"v".into(),
        &Constraint::Term(
            CompoundType::Int,
            vec![Constraint::Var(t2.clone().into_string())],
        ),
    )
    .unwrap();
    let any = env.get_type(t2.as_metavar()).unwrap();
    assert!(any.is_var());
    assert_eq!(
        env.get_type(VarName::from("v").as_metavar()),
        Some(Constraint::Term(CompoundType::Int, vec![any]))
    );

    // env.new_polymorphic(
    //     "id",
    //     vec!["t".to_string()],
    //     Constraint::Term(
    //         CompoundType::Fn,
    //         vec![
    //             Constraint::Var("t".to_string()),
    //             Constraint::Var("t".to_string()),
    //         ],
    //     ),
    // );
    // let id_1 = env.new_temp_type();
    // env.add_constraint(&id_1, &Constraint::Var("id".to_string()))
    //     .unwrap();
    // if let Constraint::Term(CompoundType::Fn, args) = env.get_type(&id_1).unwrap() {
    //     assert_eq!(args.len(), 2);
    //     let input = args[0].clone();
    //     let output = args[1].clone();
    //     env.add_constraint("res", &output).unwrap();
    //     env.add_constraint("arg", &input).unwrap();
    //     env.add_constraint(
    //         "arg",
    //         &Constraint::Term(CompoundType::Int, vec![Constraint::Atom(BaseType::I64)]),
    //     )
    //     .unwrap();
    // } else {
    //     panic!("Expected function type");
    // }
    // assert_eq!(
    //     env.get_type("res"),
    //     Some(Constraint::Term(
    //         CompoundType::Int,
    //         vec![Constraint::Atom(BaseType::I64)]
    //     ))
    // );
    // assert_eq!(
    //     env.get_type(&id_1),
    //     Some(Constraint::Term(
    //         CompoundType::Fn,
    //         vec![
    //             Constraint::Term(CompoundType::Int, vec![Constraint::Atom(BaseType::I64)]),
    //             Constraint::Term(CompoundType::Int, vec![Constraint::Atom(BaseType::I64)])
    //         ]
    //     ))
    // );
}

impl Kind for String {}

#[test]
fn test_rev_lookup() {
    let mut env: Env<String, String> = Env::new();
    env.add_class_constraint(
        ClassName::from_raw("$a".to_string()),
        &Constraint::Term(
            String::from("int"),
            vec![Constraint::Term(
                String::from("lit"),
                vec![Constraint::Term(String::from("1"), vec![])],
            )],
        ),
    )
    .unwrap();
    env.add_class_constraint(
        ClassName::new("b"),
        &Constraint::Term(
            String::from("int"),
            vec![Constraint::Term(
                String::from("lit"),
                vec![Constraint::Term(String::from("1"), vec![])],
            )],
        ),
    )
    .unwrap();
    env.add_class_constraint(
        ClassName::new("c"),
        &Constraint::Term(
            String::from("add"),
            vec![
                Constraint::Var(String::from("$a")),
                Constraint::Var(String::from("$b")),
            ],
        ),
    )
    .unwrap();

    env.add_constraint(
        &"a".into(),
        &Constraint::Term(
            String::from("int"),
            vec![Constraint::Term(
                String::from("lit"),
                vec![Constraint::Term(String::from("1"), vec![])],
            )],
        ),
    )
    .unwrap();
    env.add_constraint(
        &"j".into(),
        &Constraint::Term(
            String::from("int"),
            vec![Constraint::Term(
                String::from("lit"),
                vec![Constraint::Term(String::from("1"), vec![])],
            )],
        ),
    )
    .unwrap();
    env.add_constraint(
        &"r".into(),
        &Constraint::Term(
            String::from("add"),
            vec![
                Constraint::Var(String::from("a")),
                Constraint::Var(String::from("j")),
            ],
        ),
    )
    .unwrap();

    env.add_constraint(&"r".into(), &Constraint::Var(String::from("$c")))
        .unwrap();

    println!("{:?}", env.get_type(&VarName::from("r")));
    println!("{:?}", env.get_type(&ClassName::new("c")));
    assert_eq!(
        env.get_class_id(&VarName::from("r")).unwrap().get_raw(),
        "$c"
    );
    assert_eq!(
        env.get_class_id(&VarName::from("a")).unwrap().get_raw(),
        "$a"
    );
    assert_eq!(
        env.get_class_id(&VarName::from("j")).unwrap().get_raw(),
        "$b"
    );
}

#[test]
fn step_wise_rev_lookup() {
    let mut env: Env<String, String> = Env::new();
    env.add_class_constraint(
        ClassName::from_raw_str("$a"),
        &Constraint::Term(
            String::from("int"),
            vec![Constraint::Term(
                String::from("lit"),
                vec![Constraint::Term(String::from("1"), vec![])],
            )],
        ),
    )
    .unwrap();
    env.add_class_constraint(
        ClassName::from_raw_str("$b"),
        &Constraint::Term(
            String::from("int"),
            vec![Constraint::Term(
                String::from("lit"),
                vec![Constraint::Term(String::from("1"), vec![])],
            )],
        ),
    )
    .unwrap();
    env.add_class_constraint(
        ClassName::from_raw_str("$c"),
        &Constraint::Term(
            String::from("add"),
            vec![
                Constraint::Var(String::from("$a")),
                Constraint::Var(String::from("$b")),
            ],
        ),
    )
    .unwrap();

    env.add_constraint(
        &"a".into(),
        &Constraint::Term(
            String::from("int"),
            vec![Constraint::Term(
                String::from("lit"),
                vec![Constraint::Term(String::from("1"), vec![])],
            )],
        ),
    )
    .unwrap();
    env.add_constraint(&"a".into(), &Constraint::Var(String::from("$a")))
        .unwrap();

    assert_eq!(
        env.get_class_id(&VarName::from("a")).unwrap().get_raw(),
        "$a"
    );

    env.add_constraint(
        &"j".into(),
        &Constraint::Term(
            String::from("int"),
            vec![Constraint::Term(
                String::from("lit"),
                vec![Constraint::Term(String::from("1"), vec![])],
            )],
        ),
    )
    .unwrap();
    env.add_constraint(
        &"r".into(),
        &Constraint::Term(
            String::from("add"),
            vec![
                Constraint::Var(String::from("a")),
                Constraint::Var(String::from("j")),
            ],
        ),
    )
    .unwrap();

    env.add_class_constraint(
        ClassName::from_raw_str("$c"),
        &Constraint::Var(String::from("r")),
    )
    .unwrap();

    assert_eq!(
        env.get_class_id(&VarName::from("r")).unwrap().get_raw(),
        "$c"
    );
    assert_eq!(
        env.get_class_id(&VarName::from("a")).unwrap().get_raw(),
        "$a"
    );
    assert_eq!(
        env.get_class_id(&VarName::from("j")).unwrap().get_raw(),
        "$b"
    );
}

#[test]
fn test_subtype_lattice() {
    let mut env = Env::<String, String>::new();
    let mut r1 = BTreeMap::new();
    r1.insert(
        String::from("l1"),
        Constraint::Term(
            String::from("int"),
            vec![Constraint::Var(String::from("l1_t"))],
        ),
    );
    r1.insert(String::from("l2"), Constraint::Atom(String::from("I32")));
    env.add_class_constraint(
        ClassName::from_raw("$r".to_owned()),
        &Constraint::DynamicTerm(
            String::from("record"),
            r1,
            super::unification::SubtypeConstraint::Any,
        ),
    )
    .unwrap();
    let mut r2 = BTreeMap::new();
    r2.insert(
        String::from("l1"),
        Constraint::Term(
            String::from("int"),
            vec![Constraint::Atom(String::from("I32"))],
        ),
    );
    r2.insert(String::from("l3"), Constraint::Atom(String::from("bool")));
    env.add_class_constraint(
        ClassName::new("r"),
        &Constraint::DynamicTerm(
            String::from("record"),
            r2,
            super::unification::SubtypeConstraint::Any,
        ),
    )
    .unwrap();
    let ty = env.get_type(&ClassName::new("r")).unwrap();
    if let Constraint::DynamicTerm(_, r, _) = ty {
        assert_eq!(r.len(), 3);
        assert_eq!(
            r.get("l1").unwrap(),
            &Constraint::Term(
                String::from("int"),
                vec![Constraint::Atom(String::from("I32"))]
            )
        );
        assert_eq!(r.get("l2").unwrap(), &Constraint::Atom(String::from("I32")));
        assert_eq!(
            r.get("l3").unwrap(),
            &Constraint::Atom(String::from("bool"))
        );
    } else {
        panic!("Expected record type");
    }
}

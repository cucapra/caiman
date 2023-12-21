use super::unification::{Constraint, Env, Kind};

#[derive(Debug, PartialEq, Eq, Clone)]
enum BaseType {
    I32,
    I64,
    Bool,
    F64,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum CompoundType {
    Int,
    Float,
    Fn,
    Array,
    Tuple,
}

impl Kind for BaseType {}
impl Kind for CompoundType {}

#[test]
fn test_unification() {
    let mut env: Env<CompoundType, BaseType> = Env::new();
    env.new_type_if_absent("v");
    env.add_constraint(
        "v",
        &Constraint::Term(CompoundType::Int, vec![Constraint::Atom(BaseType::I32)]),
    )
    .unwrap();
    assert_eq!(
        env.get_type("v").unwrap(),
        Constraint::Term(CompoundType::Int, vec![Constraint::Atom(BaseType::I32)])
    );
    env.new_type_if_absent("w");
    env.new_type_if_absent("a");
    env.add_constraint("a", &Constraint::Var("v".to_string()))
        .unwrap();
    env.add_constraint("w", &Constraint::Var("a".to_string()))
        .unwrap();
    assert_eq!(env.get_type("w"), env.get_type("v"));
    assert_eq!(
        env.get_type("w").unwrap(),
        Constraint::Term(CompoundType::Int, vec![Constraint::Atom(BaseType::I32)])
    );

    env.new_type_if_absent("alpha");
    env.new_type_if_absent("beta");
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
        env.get_type("beta").unwrap(),
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
    env.new_type_if_absent("v");
    env.add_constraint(
        "v",
        &Constraint::Term(CompoundType::Int, vec![Constraint::Atom(BaseType::I32)]),
    )
    .unwrap();
    env.add_constraint(
        "v",
        &Constraint::Term(CompoundType::Int, vec![Constraint::Atom(BaseType::I64)]),
    )
    .unwrap_err();
}

#[test]
fn test_polymorphism() {
    let mut env: Env<CompoundType, BaseType> = Env::new();
    env.new_type_if_absent("v");
    let t = env.new_temp_type();
    env.add_constraint(
        "v",
        &Constraint::Term(CompoundType::Int, vec![Constraint::Var(t)]),
    )
    .unwrap();
    let t2 = env.new_temp_type();
    env.add_constraint(
        "v",
        &Constraint::Term(CompoundType::Int, vec![Constraint::Var(t2.clone())]),
    )
    .unwrap();
    let any = env.get_type(&t2).unwrap();
    assert!(any.is_var());
    assert_eq!(
        env.get_type("v"),
        Some(Constraint::Term(CompoundType::Int, vec![any]))
    );

    env.new_polymorphic(
        "id",
        vec!["t".to_string()],
        Constraint::Term(
            CompoundType::Fn,
            vec![
                Constraint::Var("t".to_string()),
                Constraint::Var("t".to_string()),
            ],
        ),
    );
    let id_1 = env.new_temp_type();
    env.add_constraint(&id_1, &Constraint::Var("id".to_string()))
        .unwrap();
    if let Constraint::Term(CompoundType::Fn, args) = env.get_type(&id_1).unwrap() {
        assert_eq!(args.len(), 2);
        let input = args[0].clone();
        let output = args[1].clone();
        env.add_constraint("res", &output).unwrap();
        env.add_constraint("arg", &input).unwrap();
        env.add_constraint(
            "arg",
            &Constraint::Term(CompoundType::Int, vec![Constraint::Atom(BaseType::I64)]),
        )
        .unwrap();
    } else {
        panic!("Expected function type");
    }
    assert_eq!(
        env.get_type("res"),
        Some(Constraint::Term(
            CompoundType::Int,
            vec![Constraint::Atom(BaseType::I64)]
        ))
    );
    assert_eq!(
        env.get_type(&id_1),
        Some(Constraint::Term(
            CompoundType::Fn,
            vec![
                Constraint::Term(CompoundType::Int, vec![Constraint::Atom(BaseType::I64)]),
                Constraint::Term(CompoundType::Int, vec![Constraint::Atom(BaseType::I64)])
            ]
        ))
    );
}

use crate::{
    error::Info,
    parse::ast::{
        ArgsOrEnc, LangTerm, NestedExpr, SchedFuncCall, SchedLiteral, SchedStmt, SchedTerm,
        SpecLiteral, SpecStmt, SpecTerm,
    },
};

/// Flattens a top level expression to be a statement without nested expressions
/// A flattened top level expression can be a term or an expression whose children are
/// variables.
/// # Arguments
/// * `e` - The expression to flatten
/// * `mk_var` - A function that creates a variable from a string
/// * `mk_decl` - A function that creates a declaration from a string and an
/// expression
/// * `temp_num` - The current number of temporary variables
/// * `flatten_term` - A function that flattens a term
/// # Returns
/// * A tuple containing:
///     * A list of statements that need to be added to the spec
///     * The new number of temporary variables
///     * The new spec term
pub fn flatten_top_level<
    T: LangTerm,
    F: Fn(&str) -> T,
    I,
    D: Fn(&str, NestedExpr<T>) -> I,
    C: Fn(T, usize) -> (Vec<I>, usize, NestedExpr<T>),
    B: Fn(NestedExpr<T>, usize) -> (Vec<I>, usize, NestedExpr<T>),
>(
    e: NestedExpr<T>,
    mk_var: &F,
    mk_decl: &D,
    temp_num: usize,
    flatten_term: &C,
    flatten_term_children: &B,
) -> (Vec<I>, usize, NestedExpr<T>) {
    match e {
        x @ NestedExpr::Term(_) => flatten_term_children(x, temp_num),
        NestedExpr::Binop { info, op, lhs, rhs } => {
            let (mut lhs_instrs, temp_num, lhs_expr) =
                flatten_rec(*lhs, mk_var, mk_decl, temp_num, flatten_term);
            let (rhs_instrs, temp_num, rhs_expr) =
                flatten_rec(*rhs, mk_var, mk_decl, temp_num, flatten_term);
            lhs_instrs.extend(rhs_instrs);
            (
                lhs_instrs,
                temp_num,
                NestedExpr::Binop {
                    info,
                    op,
                    lhs: Box::new(lhs_expr),
                    rhs: Box::new(rhs_expr),
                },
            )
        }
        NestedExpr::Uop { info, op, expr } => {
            let (expr_instrs, temp_num, expr_expr) =
                flatten_rec(*expr, mk_var, mk_decl, temp_num, flatten_term);
            (
                expr_instrs,
                temp_num,
                NestedExpr::Uop {
                    info,
                    op,
                    expr: Box::new(expr_expr),
                },
            )
        }
        NestedExpr::Conditional {
            info,
            if_true,
            guard,
            if_false,
        } => {
            let (mut guard_instrs, temp_num, guard_expr) =
                flatten_rec(*guard, mk_var, mk_decl, temp_num, flatten_term);
            let (if_true_instrs, temp_num, if_true_expr) =
                flatten_rec(*if_true, mk_var, mk_decl, temp_num, flatten_term);
            let (if_false_instrs, mut temp_num, if_false_expr) =
                flatten_rec(*if_false, mk_var, mk_decl, temp_num, flatten_term);
            guard_instrs.extend(if_true_instrs);
            guard_instrs.extend(if_false_instrs);
            temp_num += 1;
            (
                guard_instrs,
                temp_num,
                NestedExpr::Conditional {
                    info,
                    if_true: Box::new(if_true_expr),
                    guard: Box::new(guard_expr),
                    if_false: Box::new(if_false_expr),
                },
            )
        }
    }
}

/// Flattens a recursive expression to be a statement without nested expressions
/// A flattened recursive level expression can be a variable only.
/// # Arguments
/// * `e` - The expression to flatten
/// * `mk_var` - A function that creates a variable from a string
/// * `mk_decl` - A function that creates a declaration from a string and an
/// expression
/// * `temp_num` - The current number of temporary variables
/// * `flatten_term` - A function that flattens a term
/// # Returns
/// * A tuple containing:
///    * A list of statements that need to be added to the spec
///    * The new number of temporary variables
///    * The new spec term
pub fn flatten_rec<
    T: LangTerm,
    F: Fn(&str) -> T,
    I,
    D: Fn(&str, NestedExpr<T>) -> I,
    C: Fn(T, usize) -> (Vec<I>, usize, NestedExpr<T>),
>(
    e: NestedExpr<T>,
    mk_var: &F,
    mk_decl: &D,
    temp_num: usize,
    flatten_term: &C,
) -> (Vec<I>, usize, NestedExpr<T>) {
    match e {
        NestedExpr::Binop { info, op, lhs, rhs } => {
            let (mut lhs_instrs, temp_num, lhs_expr) =
                flatten_rec(*lhs, mk_var, mk_decl, temp_num, flatten_term);
            let (rhs_instrs, mut temp_num, rhs_expr) =
                flatten_rec(*rhs, mk_var, mk_decl, temp_num, flatten_term);
            lhs_instrs.extend(rhs_instrs);
            let temp_name = format!("_f{temp_num}");
            lhs_instrs.push(mk_decl(
                &temp_name,
                NestedExpr::Binop {
                    info,
                    op,
                    lhs: Box::new(lhs_expr),
                    rhs: Box::new(rhs_expr),
                },
            ));
            temp_num += 1;
            (lhs_instrs, temp_num, NestedExpr::Term(mk_var(&temp_name)))
        }
        NestedExpr::Uop { info, op, expr } => {
            let (mut expr_instrs, mut temp_num, expr_expr) =
                flatten_rec(*expr, mk_var, mk_decl, temp_num, flatten_term);
            let temp_name = format!("_f{temp_num}");
            expr_instrs.push(mk_decl(
                &temp_name,
                NestedExpr::Uop {
                    info,
                    op,
                    expr: Box::new(expr_expr),
                },
            ));
            temp_num += 1;
            (expr_instrs, temp_num, NestedExpr::Term(mk_var(&temp_name)))
        }
        NestedExpr::Conditional {
            info,
            if_true,
            guard,
            if_false,
        } => {
            let (mut guard_instrs, temp_num, guard_expr) =
                flatten_rec(*guard, mk_var, mk_decl, temp_num, flatten_term);
            let (if_true_instrs, temp_num, if_true_expr) =
                flatten_rec(*if_true, mk_var, mk_decl, temp_num, flatten_term);
            let (if_false_instrs, mut temp_num, if_false_expr) =
                flatten_rec(*if_false, mk_var, mk_decl, temp_num, flatten_term);
            guard_instrs.extend(if_true_instrs);
            guard_instrs.extend(if_false_instrs);
            let temp_name = format!("_f{temp_num}");
            guard_instrs.push(mk_decl(
                &temp_name,
                NestedExpr::Conditional {
                    info,
                    if_true: Box::new(if_true_expr),
                    guard: Box::new(guard_expr),
                    if_false: Box::new(if_false_expr),
                },
            ));
            temp_num += 1;
            (guard_instrs, temp_num, NestedExpr::Term(mk_var(&temp_name)))
        }
        NestedExpr::Term(x) if x.is_var() => (vec![], temp_num, NestedExpr::Term(x)),
        NestedExpr::Term(call) => flatten_term(call, temp_num),
    }
}

/// Flattens the arguments of a spec call to be arguments without nested expressions
/// # Arguments
/// * `args` - The arguments to flatten
/// * `temp_num` - The current number of temporary variables
/// * `info` - The info of the spec call
/// # Returns
/// * A tuple containing:
///     * A list of statements that need to be added to the spec
///     * The new arguments
///     * The new number of temporary variables
fn flatten_spec_call_args(
    args: Vec<NestedExpr<SpecTerm>>,
    temp_num: usize,
    info: Info,
) -> (Vec<SpecStmt>, Vec<NestedExpr<SpecTerm>>, usize) {
    let mut instrs = vec![];
    let mut temp_num = temp_num;
    let mut new_args = vec![];
    for arg in args {
        let (arg_instrs, new_temp_num, arg_expr) = flatten_rec(
            arg,
            &|v| SpecTerm::Var {
                info,
                name: v.to_string(),
            },
            &|name, e| SpecStmt::Assign {
                info,
                lhs: vec![(name.to_string(), None)],
                rhs: e,
            },
            temp_num,
            &flatten_spec_term,
        );
        temp_num = new_temp_num;
        instrs.extend(arg_instrs);
        new_args.push(arg_expr);
    }
    (instrs, new_args, temp_num)
}

/// Flattens a spec call to be a statement without nested expressions
/// # Arguments
/// * `call` - The spec call to flatten
/// * `temp_num` - The current number of temporary variables
/// # Returns
/// * A tuple containing:
///    * A list of statements that need to be added to the spec
///    * The new number of temporary variables
///    * The new spec term
/// # Panics
/// If the spec call is not a call
fn flatten_spec_call(
    call: SpecTerm,
    temp_num: usize,
) -> (Vec<SpecStmt>, usize, NestedExpr<SpecTerm>) {
    if let SpecTerm::Call {
        function,
        args,
        info,
        template,
    } = call
    {
        let (mut instrs, new_args, temp_num) = flatten_spec_call_args(args, temp_num, info);
        let (func_instrs, mut temp_num, func_expr) = flatten_rec(
            *function,
            &|v| SpecTerm::Var {
                info,
                name: v.to_string(),
            },
            &|name, e| SpecStmt::Assign {
                info,
                lhs: vec![(name.to_string(), None)],
                rhs: e,
            },
            temp_num,
            &flatten_spec_term,
        );
        instrs.extend(func_instrs);
        let temp_name = format!("_f{temp_num}");
        instrs.push(SpecStmt::Assign {
            lhs: vec![(temp_name.clone(), None)],
            rhs: NestedExpr::Term(SpecTerm::Call {
                info,
                function: Box::new(func_expr),
                args: new_args,
                template,
            }),
            info,
        });
        temp_num += 1;
        (
            instrs,
            temp_num,
            NestedExpr::Term(SpecTerm::Var {
                info,
                name: temp_name,
            }),
        )
    } else {
        panic!("flatten_spec_call called on non-call")
    }
}

/// Flattens a spec term to be a statement without nested expressions
/// # Arguments
/// * `t` - The spec term to flatten
/// * `temp_num` - The current number of temporary variables
/// # Returns
/// * A tuple containing:
///     * A list of statements that need to be added to the spec
///     * The new number of temporary variables
///     * The new spec term
fn flatten_spec_term(t: SpecTerm, temp_num: usize) -> (Vec<SpecStmt>, usize, NestedExpr<SpecTerm>) {
    match t {
        SpecTerm::Call { .. } => flatten_spec_call(t, temp_num),
        SpecTerm::Var { .. } => panic!("No need to flatten"),
        SpecTerm::Lit {
            info,
            lit: SpecLiteral::Tuple(exprs),
        } => {
            let mut instrs = vec![];
            let mut temp_num = temp_num;
            let mut new_exprs = vec![];
            for expr in exprs {
                let (expr_instrs, new_temp_num, expr_expr) = flatten_rec(
                    expr,
                    &|v| SpecTerm::Var {
                        info,
                        name: v.to_string(),
                    },
                    &|name, e| SpecStmt::Assign {
                        info,
                        lhs: vec![(name.to_string(), None)],
                        rhs: e,
                    },
                    temp_num,
                    &flatten_spec_term,
                );
                temp_num = new_temp_num;
                instrs.extend(expr_instrs);
                new_exprs.push(expr_expr);
            }
            (
                instrs,
                temp_num,
                NestedExpr::Term(SpecTerm::Lit {
                    info,
                    lit: SpecLiteral::Tuple(new_exprs),
                }),
            )
        }
        SpecTerm::Lit { info, lit } => {
            let temp_name = format!("_f{temp_num}");
            (
                vec![SpecStmt::Assign {
                    lhs: vec![(temp_name.clone(), None)],
                    rhs: NestedExpr::Term(SpecTerm::Lit { info, lit }),
                    info,
                }],
                temp_num + 1,
                NestedExpr::Term(SpecTerm::Var {
                    info,
                    name: temp_name,
                }),
            )
        }
    }
}

/// Flattens the spec term so that all children are not nested expressions
fn flatten_spec_term_children(
    term: NestedExpr<SpecTerm>,
    temp_num: usize,
) -> (Vec<SpecStmt>, usize, NestedExpr<SpecTerm>) {
    match term {
        NestedExpr::Term(SpecTerm::Call {
            args,
            info,
            function,
            template,
        }) => {
            let (instrs, new_args, temp_num) = flatten_spec_call_args(args, temp_num, info);
            (
                instrs,
                temp_num,
                NestedExpr::Term(SpecTerm::Call {
                    args: new_args,
                    info,
                    function,
                    template,
                }),
            )
        }
        _ => (vec![], temp_num, term),
    }
}

/// Flattens a list of spec statements to be statements without nested expressions
pub fn flatten_spec(stmts: Vec<SpecStmt>) -> Vec<SpecStmt> {
    let mut res = vec![];
    let mut temp_num = 0;
    for s in stmts {
        match s {
            SpecStmt::Assign { info, lhs, rhs } => {
                let (mut instrs, new_temp_num, new_rhs) = flatten_top_level(
                    rhs,
                    &|v| SpecTerm::Var {
                        info,
                        name: v.to_string(),
                    },
                    &|name, e| SpecStmt::Assign {
                        info,
                        lhs: vec![(name.to_string(), None)],
                        rhs: e,
                    },
                    temp_num,
                    &flatten_spec_term,
                    &flatten_spec_term_children,
                );
                temp_num = new_temp_num;
                instrs.push(SpecStmt::Assign {
                    info,
                    lhs,
                    rhs: new_rhs,
                });
                res.extend(instrs);
            }
            SpecStmt::Returns(info, returned_expr) => {
                let (mut instrs, new_temp_num, new_ret) = flatten_rec(
                    returned_expr,
                    &|v| SpecTerm::Var {
                        info,
                        name: v.to_string(),
                    },
                    &|name, e| SpecStmt::Assign {
                        info,
                        lhs: vec![(name.to_string(), None)],
                        rhs: e,
                    },
                    temp_num,
                    &flatten_spec_term,
                );
                temp_num = new_temp_num;
                instrs.push(SpecStmt::Returns(info, new_ret));
                res.extend(instrs);
            }
        }
    }
    res
}

/// Flattens a schedule term to be a statement without nested expressions
fn flatten_sched_term(
    t: SchedTerm,
    temp_num: usize,
) -> (Vec<SchedStmt>, usize, NestedExpr<SchedTerm>) {
    match t {
        SchedTerm::Call(info, call) => flatten_sched_call(call, temp_num, info),
        SchedTerm::Var { .. } => panic!("No need to flatten"),
        SchedTerm::Lit {
            info,
            lit: SchedLiteral::Tuple(exprs),
            tag,
        } => {
            let mut instrs = vec![];
            let mut temp_num = temp_num;
            let mut new_exprs = vec![];
            for expr in exprs {
                let (expr_instrs, new_temp_num, expr_expr) = flatten_rec(
                    expr,
                    &|v| SchedTerm::Var {
                        info,
                        name: v.to_string(),
                        tag: None,
                    },
                    &|name, e| SchedStmt::Decl {
                        info,
                        lhs: vec![(name.to_string(), None)],
                        expr: Some(e),
                        is_const: true,
                    },
                    temp_num,
                    &flatten_sched_term,
                );
                temp_num = new_temp_num;
                instrs.extend(expr_instrs);
                new_exprs.push(expr_expr);
            }
            (
                instrs,
                temp_num,
                NestedExpr::Term(SchedTerm::Lit {
                    info,
                    lit: SchedLiteral::Tuple(new_exprs),
                    tag,
                }),
            )
        }
        SchedTerm::Lit { info, lit, tag } => {
            let temp_name = format!("_f{temp_num}");
            (
                vec![SchedStmt::Decl {
                    lhs: vec![(temp_name.clone(), None)],
                    expr: Some(NestedExpr::Term(SchedTerm::Lit {
                        info,
                        lit,
                        tag: tag.clone(),
                    })),
                    info,
                    is_const: true,
                }],
                temp_num + 1,
                NestedExpr::Term(SchedTerm::Var {
                    info,
                    name: temp_name,
                    tag,
                }),
            )
        }
        x @ SchedTerm::Hole(_) => (vec![], temp_num, NestedExpr::Term(x)),
    }
}

/// Flattens the arguments of a call so that each argument is a term
/// and not a nested expression
/// # Arguments
/// * `args` - The arguments to flatten
/// * `temp_num` - The current number of temporary variables
/// * `info` - The info of the call
/// # Returns
/// * A tuple containing:
///     * A list of statements that need to be added to the schedule
///     * The new arguments
///     * The new number of temporary variables
fn flatten_sched_call_args(
    args: Vec<NestedExpr<SchedTerm>>,
    temp_num: usize,
    info: Info,
) -> (Vec<SchedStmt>, Vec<NestedExpr<SchedTerm>>, usize) {
    let mut instrs = vec![];
    let mut temp_num = temp_num;
    let mut new_args = vec![];
    for arg in args {
        let (arg_instrs, new_temp_num, arg_expr) = flatten_rec(
            arg,
            &|v| SchedTerm::Var {
                info,
                name: v.to_string(),
                tag: None,
            },
            &|name, e| SchedStmt::Decl {
                info,
                lhs: vec![(name.to_string(), None)],
                expr: Some(e),
                is_const: true,
            },
            temp_num,
            &flatten_sched_term,
        );
        temp_num = new_temp_num;
        instrs.extend(arg_instrs);
        new_args.push(arg_expr);
    }
    (instrs, new_args, temp_num)
}

/// Flattens a schedule call to be a statement without nested expressions
fn flatten_sched_call(
    call: SchedFuncCall,
    temp_num: usize,
    info: Info,
) -> (Vec<SchedStmt>, usize, NestedExpr<SchedTerm>) {
    let SchedFuncCall {
        args,
        target,
        templates,
        tag,
    } = call;
    if let ArgsOrEnc::Args(args) = *args {
        let (mut instrs, new_args, temp_num) = flatten_sched_call_args(args, temp_num, info);
        let (func_instrs, mut temp_num, func_expr) = flatten_rec(
            *target,
            &|v| SchedTerm::Var {
                info,
                name: v.to_string(),
                tag: None,
            },
            &|name, e| SchedStmt::Decl {
                info,
                lhs: vec![(name.to_string(), None)],
                expr: Some(e),
                is_const: true,
            },
            temp_num,
            &flatten_sched_term,
        );
        instrs.extend(func_instrs);
        let temp_name = format!("_f{temp_num}");
        instrs.push(SchedStmt::Decl {
            lhs: vec![(temp_name.clone(), None)],
            expr: Some(NestedExpr::Term(SchedTerm::Call(
                info,
                SchedFuncCall {
                    args: Box::new(ArgsOrEnc::Args(new_args)),
                    target: Box::new(func_expr),
                    templates,
                    tag,
                },
            ))),
            info,
            is_const: true,
        });
        temp_num += 1;
        (
            instrs,
            temp_num,
            NestedExpr::Term(SchedTerm::Var {
                info,
                name: temp_name,
                tag: None,
            }),
        )
    } else {
        todo!()
    }
}

/// Flattens the schedule term so that all children are not nested expressions
fn flatten_sched_term_children(
    term: NestedExpr<SchedTerm>,
    temp_num: usize,
) -> (Vec<SchedStmt>, usize, NestedExpr<SchedTerm>) {
    match term {
        NestedExpr::Term(SchedTerm::Call(
            info,
            SchedFuncCall {
                target,
                templates,
                args,
                tag,
            },
        )) => {
            if let ArgsOrEnc::Args(args) = *args {
                let (instrs, new_args, temp_num) = flatten_sched_call_args(args, temp_num, info);
                (
                    instrs,
                    temp_num,
                    NestedExpr::Term(SchedTerm::Call(
                        info,
                        SchedFuncCall {
                            args: Box::new(ArgsOrEnc::Args(new_args)),
                            target,
                            templates,
                            tag,
                        },
                    )),
                )
            } else {
                todo!()
            }
        }
        _ => (vec![], temp_num, term),
    }
}

/// Flattens a list of schedule statements to be statements without nested expressions
/// # Arguments
/// * `stmts` - The list of schedule statements to flatten
/// * `temp_num` - The current number of temporary variables
/// # Returns
/// * A tuple containing:
///    * A list of statements that need to be added to the spec
///    * The new number of temporary variables
#[allow(clippy::too_many_lines)]
fn flatten_sched_rec(stmts: Vec<SchedStmt>, mut temp_num: usize) -> (Vec<SchedStmt>, usize) {
    let mut res = vec![];
    for s in stmts {
        match s {
            SchedStmt::Decl {
                info,
                lhs,
                expr,
                is_const,
            } => {
                let mut instrs = vec![];
                if let Some(expr) = expr {
                    let (new_instrs, new_temp_num, new_rhs) = flatten_top_level(
                        expr,
                        &|v| SchedTerm::Var {
                            info,
                            name: v.to_string(),
                            tag: None,
                        },
                        &|name, e| SchedStmt::Decl {
                            info,
                            lhs: vec![(name.to_string(), None)],
                            expr: Some(e),
                            is_const,
                        },
                        temp_num,
                        &flatten_sched_term,
                        &flatten_sched_term_children,
                    );
                    temp_num = new_temp_num;
                    instrs.extend(new_instrs);
                    instrs.push(SchedStmt::Decl {
                        info,
                        lhs,
                        expr: Some(new_rhs),
                        is_const,
                    });
                } else {
                    instrs.push(SchedStmt::Decl {
                        info,
                        lhs,
                        expr: None,
                        is_const,
                    });
                }
                res.extend(instrs);
            }
            SchedStmt::Assign {
                info,
                lhs,
                rhs,
                tag,
            } => {
                let (mut instrs, new_temp_num, new_rhs) = flatten_rec(
                    rhs,
                    &|v| SchedTerm::Var {
                        info,
                        name: v.to_string(),
                        tag: None,
                    },
                    &|name, e| SchedStmt::Decl {
                        info,
                        lhs: vec![(name.to_string(), None)],
                        expr: Some(e),
                        is_const: true,
                    },
                    temp_num,
                    &flatten_sched_term,
                );
                temp_num = new_temp_num;
                instrs.push(SchedStmt::Assign {
                    info,
                    lhs,
                    rhs: new_rhs,
                    tag,
                });
                res.extend(instrs);
            }
            SchedStmt::Return(info, returned_expr) => {
                let (mut instrs, new_temp_num, new_ret) = flatten_rec(
                    returned_expr,
                    &|v| SchedTerm::Var {
                        info,
                        name: v.to_string(),
                        tag: None,
                    },
                    &|name, e| SchedStmt::Decl {
                        info,
                        lhs: vec![(name.to_string(), None)],
                        expr: Some(e),
                        is_const: true,
                    },
                    temp_num,
                    &flatten_sched_term,
                );
                temp_num = new_temp_num;
                instrs.push(SchedStmt::Return(info, new_ret));
                res.extend(instrs);
            }
            SchedStmt::If {
                guard,
                info,
                tag,
                true_block,
                false_block,
            } => {
                let (guard_instrs, new_temp_num, guard_expr) = flatten_rec(
                    guard,
                    &|v| SchedTerm::Var {
                        info,
                        name: v.to_string(),
                        tag: None,
                    },
                    &|name, e| SchedStmt::Decl {
                        info,
                        lhs: vec![(name.to_string(), None)],
                        expr: Some(e),
                        is_const: true,
                    },
                    temp_num,
                    &flatten_sched_term,
                );
                let (true_block, new_temp_num) = flatten_sched_rec(true_block, new_temp_num);
                let (false_block, new_temp_num) = flatten_sched_rec(false_block, new_temp_num);
                temp_num = new_temp_num;
                res.extend(guard_instrs);
                res.push(SchedStmt::If {
                    guard: guard_expr,
                    info,
                    tag,
                    true_block,
                    false_block,
                });
            }
            SchedStmt::Block(info, stmts) => {
                let (new_stmts, new_temp_num) = flatten_sched_rec(stmts, temp_num);
                temp_num = new_temp_num;
                res.push(SchedStmt::Block(info, new_stmts));
            }
            x @ (SchedStmt::Hole(_)
            | SchedStmt::InEdgeAnnotation { .. }
            | SchedStmt::OutEdgeAnnotation { .. }) => res.push(x),
            SchedStmt::Seq {
                info,
                dests,
                block,
                is_const,
            } => {
                let (mut block, new_temp_num) = flatten_sched_rec(vec![*block], temp_num);
                temp_num = new_temp_num;
                let last = block.pop().unwrap();
                res.extend(block);
                res.push(SchedStmt::Seq {
                    info,
                    dests,
                    block: Box::new(last),
                    is_const,
                });
            }
            SchedStmt::Call(..) => {
                todo!()
            }
        }
    }
    (res, temp_num)
}

/// Flattens a list of schedule statements to be statements without nested expressions
pub fn flatten_schedule(stmts: Vec<SchedStmt>) -> Vec<SchedStmt> {
    let (stmts, _) = flatten_sched_rec(stmts, 0);
    stmts
}

use crate::{
    error::{type_error, LocalError},
    parse::ast::{SchedExpr, SchedLiteral, SchedStmt, SchedTerm},
};

/// Converts if statements that return to sequences followed by returns.
/// For example, the following code:
/// ```text
/// if x {
///     1
/// } else {
///     2
/// }
/// ```
/// is converted to:
/// ```text
/// let _r0 = if x {
///     1
/// } else {
///     2
/// };
/// _r0
/// ```
///
/// # Returns
/// The converted statements and the number of values returned by the block or
/// and error if there is a type error regarding the number of values returned
/// and where they are returned from.
fn final_if_to_seq_helper(stmts: Vec<SchedStmt>) -> Result<(Vec<SchedStmt>, usize), LocalError> {
    let mut ret_count = 0;
    let mut ret = Vec::new();
    let num_stmts = stmts.len();
    for (idx, stmt) in stmts.into_iter().enumerate() {
        match stmt {
            SchedStmt::If {
                guard,
                true_block,
                false_block,
                info,
                tag,
            } => {
                let (true_block, true_count) = final_if_to_seq_helper(true_block)?;
                let (false_block, false_count) = final_if_to_seq_helper(false_block)?;
                if true_count != false_count {
                    return Err(type_error(
                        info,
                        "if branches return different numbers of values",
                    ));
                }
                if true_count == 0 {
                    ret.push(SchedStmt::If {
                        guard,
                        true_block,
                        false_block,
                        info,
                        tag,
                    });
                } else {
                    if idx != num_stmts - 1 {
                        return Err(type_error(
                            info,
                            "Return from if statement is not the last statement in the block",
                        ));
                    }
                    let rets: Vec<_> = (0..true_count).map(|i| format!("_r{i}")).collect();
                    ret.push(SchedStmt::Seq {
                        info,
                        dests: rets.iter().map(|s| (s.clone(), None)).collect(),
                        block: Box::new(SchedStmt::If {
                            guard,
                            true_block,
                            false_block,
                            info,
                            tag,
                        }),
                        is_const: true,
                    });
                    ret_count = rets.len();
                    ret.push(SchedStmt::Return(
                        info,
                        SchedExpr::Term(SchedTerm::Lit {
                            info,
                            lit: SchedLiteral::Tuple(
                                rets.into_iter()
                                    .map(|r| {
                                        SchedExpr::Term(SchedTerm::Var {
                                            info,
                                            name: r,
                                            tag: None,
                                        })
                                    })
                                    .collect(),
                            ),
                            tag: None,
                        }),
                    ));
                }
            }
            SchedStmt::Block(info, stmts) => {
                let (stmts, count) = final_if_to_seq_helper(stmts)?;
                ret_count = count;
                ret.push(SchedStmt::Block(info, stmts));
            }
            SchedStmt::Return(info, expr) => {
                if let SchedExpr::Term(SchedTerm::Lit {
                    lit: SchedLiteral::Tuple(rets),
                    ..
                }) = &expr
                {
                    ret_count = rets.len();
                } else {
                    ret_count = 1;
                }
                ret.push(SchedStmt::Return(info, expr));
                // no check if we're the last statement in the block because
                // the parser handles that
            }
            stmt => {
                ret.push(stmt);
            }
        }
    }
    Ok((ret, ret_count))
}

/// Converts if statements that return to sequences followed by returns.
/// For example, the following code:
/// ```text
/// fn foo() -> i32 {
///     if x {
///         1
///     } else {
///         2
///     }
/// }
/// ```
/// is converted to:
/// ```text
/// fn foo() -> i32 {
///     let _r0 = if x {
///         1
///     } else {
///         2
///     };
///     _r0
/// }
/// ```
#[allow(clippy::module_name_repetitions)]
pub fn final_if_to_seq(stmts: Vec<SchedStmt>) -> Result<Vec<SchedStmt>, LocalError> {
    let (stmts, _) = final_if_to_seq_helper(stmts)?;
    Ok(stmts)
}

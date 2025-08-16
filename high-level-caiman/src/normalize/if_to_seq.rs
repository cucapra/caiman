use crate::{
    error::{Info, LocalError},
    parse::ast::{NestedExpr, SchedExpr, SchedLiteral, SchedStmt, SchedTerm, Tag},
    type_error,
};

/// Converts a single if statement to become a sequence followed by a return.
/// # Arguments
/// * `guard` - The guard of the if statement
/// * `true_block` - The block of statements to execute if the guard is true
/// * `false_block` - The block of statements to execute if the guard is false
/// * `info` - The location of the if statement in the source code
/// * `tag` - The tag of the if statement
/// * `idx` - The index of the if statement in the block
/// * `num_stmts` - The number of statements in the block
/// # Returns
/// The converted statements and the number of values returned by the block or
/// and error if there is a type error regarding the number of values returned
/// and where they are returned from.
fn convert_if_to_seq(
    guard: NestedExpr<SchedTerm>,
    true_block: Vec<SchedStmt>,
    false_block: Vec<SchedStmt>,
    info: Info,
    tag: Option<Vec<Tag>>,
    idx: usize,
    num_stmts: usize,
) -> Result<(Vec<SchedStmt>, usize), LocalError> {
    let (true_block, true_count) = final_if_to_seq_helper(true_block)?;
    let (false_block, false_count) = final_if_to_seq_helper(false_block)?;
    let mut ret = vec![];
    let mut ret_count = 0;
    if true_count != false_count {
        return Err(type_error!(
            info,
            "if branches return different numbers of values"
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
            return Err(type_error!(
                info,
                "Return from if statement is not the last statement in the block"
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
    Ok((ret, ret_count))
}

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
#[allow(clippy::too_many_lines)]
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
                let (stmts, count) =
                    convert_if_to_seq(guard, true_block, false_block, info, tag, idx, num_stmts)?;
                if idx == num_stmts - 1 {
                    ret_count = count;
                }
                ret.extend(stmts);
            }
            SchedStmt::Block(info, stmts) => {
                let (stmts, count) = final_if_to_seq_helper(stmts)?;
                if idx == num_stmts - 1 {
                    ret_count = count;
                }
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
            SchedStmt::Seq {
                block,
                dests,
                info,
                is_const,
            } => {
                let mut r = seq_final_if_to_seq_helper(info, dests.len(), vec![*block])?;
                ret.push(SchedStmt::Seq {
                    block: Box::new(if r.len() == 1 {
                        r.pop().unwrap()
                    } else {
                        SchedStmt::Block(info, r)
                    }),
                    dests,
                    info,
                    is_const,
                });
            }
            stmt => {
                ret.push(stmt);
            }
        }
    }
    Ok((ret, ret_count))
}

/// Recurses on the blocks of a sequence to convert any nested ifs into sequences.
/// Since we are already inside a sequence, the top level if statement is kept as
/// an if statement and we recurse on the branches of the if.
/// # Arguments
/// * `seq_info` - The location of the sequence in the source code
/// * `seq_dests` - The number of destinations of the sequence
/// * `stmts` - The body of the sequence
/// # Returns
/// The converted body or an error if there is a type error regarding the
/// number of values returned and where they are returned from.
fn seq_final_if_to_seq_helper(
    seq_info: Info,
    seq_dests: usize,
    mut stmts: Vec<SchedStmt>,
) -> Result<Vec<SchedStmt>, LocalError> {
    if stmts.len() == 1 {
        match stmts.pop().unwrap() {
            SchedStmt::Block(_, stmts) => {
                let stmts = seq_final_if_to_seq_helper(seq_info, seq_dests, stmts)?;
                return Ok(stmts);
            }
            SchedStmt::If {
                guard,
                true_block,
                false_block,
                info,
                tag,
            } => {
                // inside a seq, we just want to convert any ifs to seqs in each
                // branch
                let (true_block, true_count) = final_if_to_seq_helper(true_block)?;
                let (false_block, false_count) = final_if_to_seq_helper(false_block)?;
                if true_count != false_count {
                    return Err(type_error!(
                        info,
                        "if branches return different numbers of values"
                    ));
                }
                if true_count != seq_dests {
                    return Err(type_error!(
                        seq_info,
                        "Cannot assign {true_count} returns into {seq_dests} destinations"
                    ));
                }
                return Ok(vec![SchedStmt::If {
                    guard,
                    true_block,
                    false_block,
                    info,
                    tag,
                }]);
            }
            x => unreachable!("{x:#?}"),
        }
    }
    final_if_to_seq(stmts)
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

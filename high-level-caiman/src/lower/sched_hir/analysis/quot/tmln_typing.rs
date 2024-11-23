//! This file contains the logic for deducing the timeline quotients of a function.
//! It follows the same approach as `val_typing`, with the main difference being
//! that it doesn't operate in SSA form. This can be done because the timeline
//! doesn't have selects right now (phi nodes are used for this) and because
//! we can't "store" to a fence or encoder. In other words, every timeline
//! operation has a unique destination. Furthermore, we don't need SSA because
//! the latest local event can be tracked in a dataflow fashion, and applied
//! to all the defs and live in variables.
//!
//! We keep track of the latest local events that the coordinator knows about
//! as a linear sequence of events. For each instruction, we keep track of the
//! local event that is active right after the instruction takes effect.

use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    vec,
};

use caiman::explication::Hole;

use crate::{
    error::{Info, LocalError},
    lower::{
        sched_hir::{
            analysis::{InOutFacts, LiveVars},
            cfg::{BasicBlock, Cfg, FINAL_BLOCK_ID, START_BLOCK_ID},
            Hir, HirBody, HirFuncCall, HirTerm, Terminator, TripleTag,
        },
        tuple_id,
    },
    parse::ast::{DataType, Quotient, SpecType},
    type_error,
    typing::{is_timeline_dtype, Context, MetaVar, NodeEnv, SchedOrExtern, SpecInfo, ValQuot},
};

use super::{add_constraint, add_node_eq, add_var_constraint};

const LOCAL_STEM: &str = "_loc";

/// Gets the implicit input variable of the starting basic block of `cfg`. If one
/// is specified, uses the user-supplied tag, otherwise assumes it's the spec's implicit input
fn get_implicit_input_var<'a>(cfg: &'a Cfg, spec_in: &'a str) -> &'a str {
    let mut res = spec_in;
    for stmt in &cfg.blocks.get(&START_BLOCK_ID).unwrap().stmts {
        if let HirBody::InAnnotation(_, annots) = stmt {
            for (a, t) in annots {
                if a == "input"
                    && !matches!(t.timeline.quot, Some(Quotient::None))
                    && t.timeline.quot_var.spec_var.is_some()
                {
                    res = t.timeline.quot_var.spec_var.as_ref().unwrap();
                }
            }
        }
    }
    res
}

/// Deduces the timeline quotients and adds them to tags of instructions in the CFG.
/// Required that active fences pass has been run and record expansion has NOT
/// been run.
/// # Arguments
/// * `inputs` - The input variables of the function
/// * `outputs` - The output variables of the function
/// * `output_dtypes` - The data types of the output variables
/// * `cfg` - The control flow graph of the function
/// * `spec_info` - The timeline spec of the function
/// * `ctx` - The context of the program
/// * `dtypes` - The data types of the variables in the function
/// * `info` - The source info for the function
/// * `num_dims` - The number of dimensions in function (non-type templates)
/// * `spec_name` - The name of the spec
/// * `live_vars` - The live variables of the function
#[allow(clippy::too_many_arguments)]
pub fn deduce_tmln_quots(
    inputs: &mut [(String, TripleTag)],
    outputs: &mut [TripleTag],
    output_dtypes: &[DataType],
    cfg: &mut Cfg,
    spec_info: &SpecInfo,
    ctx: &Context,
    dtypes: &HashMap<String, DataType>,
    info: Info,
    num_dims: usize,
    spec_name: &str,
    live_vars: &InOutFacts<LiveVars>,
) -> Result<NodeEnv, LocalError> {
    let env = spec_info.nodes.clone();
    let env = add_io_constraints(env, inputs, outputs, output_dtypes, dtypes, info, num_dims)?;
    let implicit_in = get_implicit_input_var(cfg, &spec_info.sig.input[0].0).to_string();
    let (env, implicit_events) = unify_nodes(cfg, &implicit_in, dtypes, ctx, env)?;
    // sort of a hack to support the previous tests without timeline specs:
    // A timeline spec that is trivial and never called is just not deduced so
    // it can remain none
    let do_fill =
        ctx.called_specs.contains(&spec_info.feq) || !ctx.trivial_tmlns.contains(spec_name);
    if do_fill {
        fill_io_tags(inputs, outputs, output_dtypes, &env, &implicit_events);
        fill_tmln_tags(cfg, &env, &implicit_events, live_vars, dtypes);
        add_implicit_annotations(cfg, &implicit_events);
    }
    Ok(env)
}

/// A struct containing the implicit input and output events for each funclet in the CFG.
struct InOutEvents(HashMap<usize, (TripleTag, TripleTag, Vec<i32>)>);

impl InOutEvents {
    /// Gets the implicit input event for the given block
    pub fn get_implicit_input(&self, block: usize) -> &TripleTag {
        &self.0[&block].0
    }

    /// Gets the implicit output event for the given block
    pub fn get_implicit_output(&self, block: usize) -> &TripleTag {
        &self.0[&block].1
    }

    /// Gets the local event name of the event that is active right after
    /// the given instruction
    pub fn get_local_name_after_instr(&self, block: usize, instr_id: usize) -> String {
        // +1 to skip the implicit input
        format!("{LOCAL_STEM}{}", &self.0[&block].2[instr_id + 1])
    }

    /// Gets the local event name of the implicit input
    pub fn get_implicit_in_name(&self, block: usize) -> String {
        format!("{LOCAL_STEM}{}", &self.0[&block].2[0])
    }

    /// Gets the local event name of the implicit output
    pub fn get_implicit_out_name(&self, block: usize) -> String {
        format!("{LOCAL_STEM}{}", &self.0[&block].2.last().unwrap())
    }
}

/// Adds input and output annotations for the implicit events of every funclet
fn add_implicit_annotations(cfg: &mut Cfg, annots: &InOutEvents) {
    for (block_id, block) in &mut cfg.blocks {
        block.stmts.insert(
            0,
            HirBody::InAnnotation(
                block.get_starting_info(),
                vec![
                    (
                        String::from("input"),
                        annots.get_implicit_input(*block_id).clone(),
                    ),
                    (
                        String::from("output"),
                        annots.get_implicit_output(*block_id).clone(),
                    ),
                ],
            ),
        );
    }
}

/// Unifies the timeline constraints of the nodes in the CFG.
/// # Arguments
/// * `cfg` - The control flow graph to unify
/// * `implicit_in` - The name of the implicit input variable
/// * `dtypes` - The data types of the variables in the program
/// * `ctx` - The context of the program
/// * `env` - The current environment
/// # Returns
/// The updated environment
/// # Errors
/// If a constraint cannot be added to the environment
fn unify_nodes(
    cfg: &Cfg,
    implicit_in: &str,
    dtypes: &HashMap<String, DataType>,
    ctx: &Context,
    mut env: NodeEnv,
) -> Result<(NodeEnv, InOutEvents), LocalError> {
    let mut seen = HashMap::new();
    env = add_node_eq(&format!("{LOCAL_STEM}0"), implicit_in, Info::default(), env)?;
    seen.insert(START_BLOCK_ID, 0);
    let mut node_q = cfg.topo_order_rev.clone();
    let mut block_loc_events = HashMap::new();
    // the last globally used local event number
    let mut latest_loc = 0;
    let mut implicit_annotations = ImplicitAnnotations::new(cfg);
    let mut hole_local_events = HashSet::new();
    while let Some(bb) = node_q.pop() {
        let bb = &cfg.blocks[&bb];
        // the last local event number for the current path
        let mut last_loc = seen[&bb.id];
        implicit_annotations.set_cur_block(bb.id);
        env = implicit_annotations.unify_inputs(last_loc, env)?;
        let mut local_events = vec![last_loc];
        env = unify_instrs(
            bb,
            env,
            &mut last_loc,
            &mut latest_loc,
            &mut implicit_annotations,
            &mut local_events,
            dtypes,
            &mut hole_local_events,
        )?;
        env = unify_terminator(
            bb,
            cfg,
            env,
            &mut last_loc,
            &mut latest_loc,
            dtypes,
            ctx,
            &mut implicit_annotations,
        )?;
        local_events.push(last_loc);
        env = implicit_annotations.unify_outputs(last_loc, env)?;
        block_loc_events.insert(bb.id, local_events);
        for succ in &cfg.graph[&bb.id] {
            match seen.entry(*succ) {
                Entry::Occupied(entry) => {
                    // merge last local events to be equal
                    // this must be the case since the timeline doesn't have selects
                    if entry.get() != &last_loc {
                        // println!("{env:#?}");
                        env = add_var_constraint(
                            &format!("{LOCAL_STEM}{last_loc}"),
                            &format!("{LOCAL_STEM}{}", entry.get()),
                            cfg.blocks[succ]
                                .stmts
                                .first()
                                .map_or(cfg.blocks[succ].src_loc, Hir::get_info),
                            env,
                        )?;
                    }
                }
                Entry::Vacant(entry) => {
                    // input event to successor is the current last event
                    entry.insert(last_loc);
                }
            }
        }
    }
    env.converge_types()
        .map_err(|e| type_error!(Info::default(), "Failed to converge node types:\n {e}"))?;
    for ev in hole_local_events {
        // for holes, unify the events we created for each hole if we don't need them
        if env.get_node_name(&format!("{LOCAL_STEM}{ev}")).is_none() {
            env.add_node_eq(
                &format!("{LOCAL_STEM}{ev}"),
                &format!("{LOCAL_STEM}{}", ev + 1),
            )
            .map_err(|s| type_error!(Info::default(), "{s}"))?;
        }
    }
    let io_evs = into_input_output_annotations(cfg, &env, &block_loc_events)?;
    Ok((env, io_evs))
}

/// Converts a map from block id to a vector of local events, with an element for
/// the local event right after the `i + 1` instruction in the block,
/// to a map from block id to the implicit input and output events of the block.
/// # Arguments
/// * `cfg` - The control flow graph
/// * `env` - The current environment
/// * `first_last_events` - The map from block id to the list of local events. Index i
///     of this vector is the local event right after the `i + 1` instruction in the block.
///     So index 0 is the implicit input and the last index is the implicit output.
fn into_input_output_annotations(
    cfg: &Cfg,
    env: &NodeEnv,
    first_last_events: &HashMap<usize, Vec<i32>>,
) -> Result<InOutEvents, LocalError> {
    let mut res = HashMap::new();
    for (block, events) in first_last_events {
        let mut in_ev = TripleTag::new_unspecified();
        in_ev.timeline.quot = if *block == START_BLOCK_ID {
            Some(Quotient::Input)
        } else {
            Some(Quotient::Node)
        };
        in_ev.timeline.quot_var.spec_var = Some(
            env.get_node_name(&format!("{LOCAL_STEM}{}", events.first().unwrap()))
                .ok_or_else(|| {
                    type_error!(
                        cfg.blocks[block].get_starting_info(),
                        "Need annotation for implicit in"
                    )
                })?,
        );
        let mut out_ev = TripleTag::new_unspecified();
        let out_block = cfg.get_continuation_output_block(*block);
        out_ev.timeline.quot_var.spec_var = Some(
            env.get_node_name(&format!(
                "{LOCAL_STEM}{}",
                first_last_events[&out_block].last().unwrap()
            ))
            .ok_or_else(|| {
                type_error!(
                    cfg.blocks[&out_block].get_final_info(),
                    "Need annotation for implicit out"
                )
            })?,
        );
        out_ev.timeline.quot = Some(Quotient::Node);
        res.insert(*block, (in_ev, out_ev, events.clone()));
    }
    Ok(InOutEvents(res))
}

/// A struct to manage the implicit event annotations. Allows adding an annotation
/// to the correct block and unifying the annotations of the current block with
/// the latest local event.
struct ImplicitAnnotations<'a> {
    inputs: HashMap<usize, Vec<(TripleTag, Info)>>,
    outputs: HashMap<usize, Vec<(TripleTag, Info)>>,
    cur_block: usize,
    cfg: &'a Cfg,
}

impl<'a> ImplicitAnnotations<'a> {
    pub fn new(cfg: &'a Cfg) -> Self {
        Self {
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            cfg,
            cur_block: START_BLOCK_ID,
        }
    }

    /// Sets the current block. Annotations added after this call will be added to
    /// blocks based on the set block here.
    pub fn set_cur_block(&mut self, block: usize) {
        self.cur_block = block;
    }

    /// Unifies the inputs of the current block with the latest local event.
    pub fn unify_inputs(&self, last_loc: i32, mut env: NodeEnv) -> Result<NodeEnv, LocalError> {
        for (tag, info) in self.inputs.get(&self.cur_block).unwrap_or(&vec![]) {
            let loc_event = format!("{LOCAL_STEM}{last_loc}");
            env = add_type_annot(&loc_event, tag, *info, env)?;
        }
        Ok(env)
    }

    /// Unifies the outputs of the current block with the latest local event.
    pub fn unify_outputs(&self, last_loc: i32, mut env: NodeEnv) -> Result<NodeEnv, LocalError> {
        for (tag, info) in self.outputs.get(&self.cur_block).unwrap_or(&vec![]) {
            let loc_event = format!("{LOCAL_STEM}{last_loc}");
            env = add_type_annot(&loc_event, tag, *info, env)?;
        }
        Ok(env)
    }

    /// Adds an output annotation to the current block.
    pub fn add_cur_output(&mut self, tag: &TripleTag, info: Info) {
        self.add_output(tag, info, self.cur_block);
    }

    /// Adds an input annotation to the next block. This should be called
    /// on an input annotation in an `@out` annotation.
    pub fn add_next_input(&mut self, tag: &TripleTag, info: Info) {
        for succ in self.cfg.graph[&self.cur_block].targets() {
            self.inputs
                .entry(succ)
                .or_default()
                .push((tag.clone(), info));
        }
    }

    /// Helper function for `add_cur_output`. Adds an annotation to the output
    /// the continuation of `block` if it has one, or itself if it doesn't.
    /// See the logic in `Funclets::output_vars`.
    fn add_output(&mut self, tag: &TripleTag, info: Info, block: usize) {
        self.outputs
            .entry(self.cfg.get_continuation_output_block(block))
            .or_default()
            .push((tag.clone(), info));
    }

    /// Gets the output annotations for the outputs of the final block.
    pub fn get_final_out_annotations(&mut self) -> &[(TripleTag, Info)] {
        self.outputs.entry(FINAL_BLOCK_ID).or_default()
    }
}

/// Unifies the timeline constraints of the statements in a basic block.
/// # Arguments
/// * `bb` - The basic block to unify
/// * `env` - The current environment
/// * `last_loc` - The last local event number along the current path through
///     the CFG
/// * `latest_loc` - The last used local event number globally for the entire
///     CFG
/// * `output_annotations` - Will be pushed with annotations for the final
///     local event
/// # Returns
/// The updated environment
/// # Errors
/// If a constraint cannot be added to the environment
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
fn unify_instrs(
    bb: &BasicBlock,
    mut env: NodeEnv,
    last_loc: &mut i32,
    latest_loc: &mut i32,
    implicit_annotations: &mut ImplicitAnnotations,
    local_events: &mut Vec<i32>,
    dtypes: &HashMap<String, DataType>,
    hole_locs: &mut HashSet<i32>,
) -> Result<NodeEnv, LocalError> {
    let input_loc = format!("{LOCAL_STEM}{}", *last_loc);
    for instr in &bb.stmts {
        env = match instr {
            HirBody::ConstDecl {
                lhs,
                lhs_tag,
                rhs,
                info,
                ..
            } => unify_decl(lhs, lhs_tag, rhs, *info, env)?,
            HirBody::VarDecl {
                lhs,
                lhs_tag,
                rhs,
                info,
                ..
            } => {
                if let Some(rhs) = rhs {
                    unify_decl(lhs, lhs_tag, rhs, *info, env)?
                } else {
                    env
                }
            }
            HirBody::RefStore {
                lhs,
                lhs_tags,
                rhs,
                info,
                ..
            } => unify_decl(lhs, lhs_tags, rhs, *info, env)?,
            HirBody::RefLoad {
                dest, src, info, ..
            } => add_var_constraint(dest, src, *info, env)?,
            HirBody::InAnnotation(info, tags) => {
                for (name, tag) in tags {
                    if name == "input" {
                        env = add_type_annot(&input_loc, tag, *info, env)?;
                    } else if name == "output" {
                        implicit_annotations.add_cur_output(tag, *info);
                    } else if is_timeline_dtype(&dtypes[name]) {
                        env = add_type_annot(name, tag, *info, env)?;
                    }
                }
                env
            }
            HirBody::OutAnnotation(info, tags) => {
                for (name, tag) in tags {
                    if name == "input" {
                        implicit_annotations.add_next_input(tag, *info);
                    } else if name == "output" {
                        implicit_annotations.add_cur_output(tag, *info);
                    } else if is_timeline_dtype(&dtypes[name]) {
                        env = add_type_annot(name, tag, *info, env)?;
                    }
                }
                env
            }
            HirBody::BeginEncoding {
                encoder,
                active_fences,
                tags,
                info,
                ..
            } => unify_begin_encode(
                encoder,
                active_fences,
                tags,
                env,
                *info,
                last_loc,
                latest_loc,
            )?,
            HirBody::Submit {
                dest,
                src,
                info,
                tags,
            } => unify_submit(dest, src, tags, *info, env)?,
            HirBody::Sync {
                dests,
                srcs,
                tags,
                info,
            } => unify_sync(
                &tuple_id(
                    &dests
                        .processed()
                        .iter()
                        .map(|x| x.0.clone())
                        .collect::<Vec<_>>(),
                ),
                // fence source is the first source
                &srcs.processed()[0],
                tags,
                *info,
                env,
                last_loc,
                latest_loc,
            )?,
            // ignore phi since we don't have selects
            // ignore device copy since that's just part of the begin encoding event
            HirBody::Op { .. }
            | HirBody::Phi { .. }
            | HirBody::EncodeDo { .. }
            | HirBody::DeviceCopy { .. } => env,
            HirBody::Hole {
                dests,
                info,
                active_fences,
                ..
            } => {
                let mut inced = false;
                for (d, t) in dests {
                    match dtypes.get(d) {
                        // TODO: what if they should be returned from calls?
                        Some(DataType::Encoder(Some(_))) => {
                            inced = true;
                            env = unify_begin_encode(
                                &(d.clone(), t.clone()),
                                active_fences,
                                t,
                                env,
                                *info,
                                last_loc,
                                latest_loc,
                            )?;
                        }
                        Some(DataType::Record(_)) => {
                            inced = true;
                            env = unify_sync(
                                d,
                                &env.new_temp().into_string(),
                                t,
                                *info,
                                env,
                                last_loc,
                                latest_loc,
                            )?;
                        }
                        _ => {}
                    }
                }
                if !inced {
                    hole_locs.insert(*latest_loc);
                    *latest_loc += 1;
                    *last_loc = *latest_loc;
                }
                env
            }
        };
        local_events.push(*last_loc);
    }
    Ok(env)
}

/// Unifies the terminator of a basic block.
/// # Arguments
/// * `bb` - The basic block to unify
/// * `env` - The current environment
/// * `last_loc` - The last local event number along the current path through
///     the CFG
/// * `latest_loc` - The last used local event number globally for the entire
///     CFG
/// * `dtypes` - The data types of the variables in the program
/// * `ctx` - The context of the program
/// # Returns
/// The updated environment
/// # Errors
/// If a constraint cannot be added to the environment
#[allow(clippy::too_many_arguments)]
fn unify_terminator(
    bb: &BasicBlock,
    cfg: &Cfg,
    mut env: NodeEnv,
    last_loc: &mut i32,
    latest_loc: &mut i32,
    dtypes: &HashMap<String, DataType>,
    ctx: &Context,
    implicit_annotations: &mut ImplicitAnnotations,
) -> Result<NodeEnv, LocalError> {
    match &bb.terminator {
        Terminator::CaptureCall { dests, call, .. } | Terminator::Call(dests, call, ..) => {
            unify_call(
                dests,
                call,
                &cfg.blocks[&cfg.graph[&bb.id].next().unwrap()],
                env,
                last_loc,
                latest_loc,
                ctx,
                dtypes,
            )
        }
        Terminator::Return {
            dests, rets, info, ..
        } => {
            // pass through is ignored (like next)
            // the destination tag is the tag for the merged node, we handle this
            for ((dest, _), ret) in dests.iter().zip(rets.iter()) {
                if let Hole::Filled(ret) = ret {
                    env = add_var_constraint(dest, ret, *info, env)?;
                }
            }
            Ok(env)
        }
        Terminator::FinalReturn(info, ret_names) => {
            let output_classes: Vec<_> = env.get_function_output_classes().to_vec();
            assert!(!output_classes.is_empty() && output_classes[0].is_some());
            env = unify_implicit_output(
                env,
                implicit_annotations.get_final_out_annotations(),
                &output_classes,
                *info,
                last_loc,
                latest_loc,
            )?;
            // handle the rest of the arguments
            for (idx, (ret_name, class)) in ret_names
                .iter()
                .filter(|&rname| {
                    is_timeline_dtype(
                        dtypes
                            .get(rname)
                            .unwrap_or_else(|| panic!("{info}: Missing dtype for {rname}")),
                    )
                })
                // skip the implicit output
                .zip(output_classes.into_iter().skip(1))
                .enumerate()
            {
                if let Some(func_class) = class {
                    // +1 to skip implicit output
                    if idx + 1 < env.get_spec_output_classes().len()
                        && env.get_spec_output_classes()[idx + 1] == func_class
                    {
                        env = add_constraint(
                            &format!("{ret_name}!"),
                            &ValQuot::Output(MetaVar::new_var_name(ret_name)),
                            *info,
                            env,
                        )?;
                        env = add_node_eq(&format!("{ret_name}!"), &func_class, *info, env)?;
                    } else {
                        env = add_node_eq(ret_name, &func_class, *info, env)?;
                    }
                }
            }
            Ok(env)
        }
        Terminator::Select { .. }
        | Terminator::None(..)
        | Terminator::Next(..)
        | Terminator::Yield(..) => Ok(env),
    }
}

/// Unifies the implicit output of a final return
fn unify_implicit_output(
    mut env: NodeEnv,
    out_annotations: &[(TripleTag, Info)],
    output_classes: &[Option<String>],
    info: Info,
    last_loc: &mut i32,
    latest_loc: &mut i32,
) -> Result<NodeEnv, LocalError> {
    // handle the implicit output
    let last_loc_event = format!("{LOCAL_STEM}{last_loc}");
    let implicit_out = format!("{LOCAL_STEM}{}", *latest_loc + 1);
    *latest_loc += 1;
    *last_loc = *latest_loc;
    // option of whether the output is annotated
    let output_annot = out_annotations.iter().rev().find_map(|(tag, _)| {
        tag.timeline.quot.and_then(|t| {
            if t == Quotient::None {
                todo!("None annotated outputs")
            } else {
                tag.timeline.quot_var.spec_var.clone()
            }
        })
    });
    match (output_annot, env.get_spec_output_classes()[0].clone()) {
        // no annotation or the annotation matches
        (None, t) | (Some(t), _) if t == env.get_spec_output_classes()[0] => {
            env = add_constraint(
                &implicit_out,
                &ValQuot::Output(MetaVar::new_var_name(&last_loc_event)),
                info,
                env,
            )?;
            env = add_node_eq(
                &implicit_out,
                output_classes[0].as_ref().unwrap(),
                info,
                env,
            )?;
        }
        (Some(output_annot), _) => {
            // annotation doesn't match, take the user's word for it
            env = add_var_constraint(&last_loc_event, &output_annot, info, env)?;
        }
        (None, _) => unreachable!(),
    }
    Ok(env)
}

fn unify_begin_encode(
    encoder: &(String, TripleTag),
    active_fences: &[String],
    tags: &TripleTag,
    mut env: NodeEnv,
    info: Info,
    last_loc: &mut i32,
    latest_loc: &mut i32,
) -> Result<NodeEnv, LocalError> {
    // only begin encoding requires an extraction (out of the special timeline commands)
    let node_args = std::iter::once(MetaVar::new_var_name(&format!("{LOCAL_STEM}{last_loc}")))
        .chain(active_fences.iter().map(|x| MetaVar::new_var_name(x)))
        .collect();
    let enc_result = tuple_id(&[encoder.0.clone()]);
    env = add_constraint(
        &enc_result,
        &ValQuot::Call(String::from("encode_event"), node_args),
        info,
        env,
    )?;
    env = add_constraint(
        &format!("{LOCAL_STEM}{}", *latest_loc + 1),
        &ValQuot::Extract(MetaVar::new_var_name(&enc_result), 0),
        info,
        env,
    )?;
    env = add_constraint(
        &encoder.0,
        &ValQuot::Extract(MetaVar::new_var_name(&enc_result), 1),
        info,
        env,
    )?;
    env = add_type_annot(&enc_result, tags, info, env)?;
    env = add_type_annot(&encoder.0, &encoder.1, info, env)?;
    *latest_loc += 1;
    *last_loc = *latest_loc;
    Ok(env)
}

fn unify_submit(
    dest: &str,
    src: &str,
    tags: &TripleTag,
    info: Info,
    mut env: NodeEnv,
) -> Result<NodeEnv, LocalError> {
    env = add_constraint(
        dest,
        &ValQuot::CallOne(
            String::from("submit_event"),
            vec![MetaVar::new_var_name(src)],
        ),
        info,
        env,
    )?;
    add_type_annot(dest, tags, info, env)
}

fn unify_sync(
    dest: &str,
    src: &str,
    tags: &TripleTag,
    info: Info,
    mut env: NodeEnv,
    last_loc: &mut i32,
    latest_loc: &mut i32,
) -> Result<NodeEnv, LocalError> {
    let srcs = vec![
        MetaVar::new_var_name(&format!("{LOCAL_STEM}{last_loc}")),
        MetaVar::new_var_name(src),
    ];
    env = add_constraint(
        dest,
        &ValQuot::CallOne(String::from("sync_event"), srcs),
        info,
        env,
    )?;
    env = add_var_constraint(dest, &format!("{LOCAL_STEM}{}", *latest_loc + 1), info, env)?;
    env = add_type_annot(dest, tags, info, env)?;
    *latest_loc += 1;
    *last_loc = *latest_loc;
    Ok(env)
}

/// Gets the class name of a timeline spec
fn timeline_class_name(sched_target: &str, ctx: &Context) -> String {
    ctx.specs[&match ctx.scheds.get(sched_target).unwrap() {
        SchedOrExtern::Sched(sched) => sched.timeline.clone(),
        SchedOrExtern::Extern(_) => sched_target.to_string(),
    }]
        .feq
        .clone()
}

/// Returns the implicit input annotation for a block if it exists
fn implicit_input_annot(bb: &BasicBlock) -> Option<TripleTag> {
    for stmt in bb.stmts.iter().rev() {
        if let HirBody::InAnnotation(_, annots) = stmt {
            let t = annots.iter().find_map(|(name, t)| {
                if name == "input" {
                    Some(t.clone())
                } else {
                    None
                }
            });
            if t.is_some() {
                return t;
            }
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn unify_call(
    dests: &[(String, TripleTag)],
    call: &HirFuncCall,
    succ: &BasicBlock,
    mut env: NodeEnv,
    last_loc: &mut i32,
    latest_loc: &mut i32,
    ctx: &Context,
    dtypes: &HashMap<String, DataType>,
) -> Result<NodeEnv, LocalError> {
    let f_class = timeline_class_name(&call.target, ctx);
    let tuple_name = tuple_id(
        &dests
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>(),
    );
    let call_constraint = ValQuot::SchedCall(
        f_class,
        std::iter::once(MetaVar::new_var_name(&format!("{LOCAL_STEM}{last_loc}")))
            .chain(call.args.iter().filter_map(|arg| {
                if let Hole::Filled(arg) = arg {
                    if is_timeline_dtype(&dtypes[arg]) {
                        Some(MetaVar::new_var_name(arg))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }))
            .collect(),
    );
    let succ_implicit_input = implicit_input_annot(succ);
    if !env.spec_has_match(&call_constraint) {
        // there is no call for this function in the spec, so this isn't something to worry about
        // we do this for "backwards compatibility" for all the tests written
        // before the timeline was implemented and thus do not contain
        // timeline annotations. This isn't a good reason on its own,
        // but I quite liked the ability to not need to specify any timeline tags
        // if it's all none. So if the function
        // call isn't in the spec, we assume it's none. This is slightly different from
        // the value language which requires a type annotation to do this.

        // this decision makes more sense in the timeline language since we can assume
        // the timeline isn't affected by the function and let the previous local
        // event carry over. The value language can't make such an assumption,
        // we need to determine the tags for the return values of the function.

        if succ_implicit_input.is_some() {
            // we increment the latest loc here even if we skip the call bc the call
            // might alter the timeline even if it isn't in the spec
            // further, there is an annotation in the successor block to constrain
            // this new event
            *latest_loc += 1;
            *last_loc = *latest_loc;
        }
        return Ok(env);
    }
    *latest_loc += 1;
    *last_loc = *latest_loc;
    env = add_type_annot(&tuple_name, &call.tag, call.info, env)?;
    env = add_overrideable_constraint(&tuple_name, &call.tag, &call_constraint, call.info, env)?;
    // allow an annotation to overload the local constraint
    env = add_overrideable_constraint(
        &format!("{LOCAL_STEM}{}", *latest_loc),
        succ_implicit_input
            .as_ref()
            .unwrap_or(&TripleTag::new_unspecified()),
        &ValQuot::Extract(MetaVar::new_var_name(&tuple_name), 0),
        call.info,
        env,
    )?;
    for (idx, (dest, tag)) in dests
        .iter()
        .filter(|(x, _)| is_timeline_dtype(&dtypes[x]))
        .enumerate()
    {
        env = add_type_annot(dest, tag, call.info, env)?;
        env = add_overrideable_constraint(
            dest,
            tag,
            &ValQuot::Extract(MetaVar::new_var_name(&tuple_name), idx + 1),
            call.info,
            env,
        )?;
    }
    Ok(env)
}

/// Adds constraints to the environment based on input and output annotations.
/// Any unspecified annotations are going to be assumed to match up with the
/// spec. Requires that the input and output variables of a given dimension
/// (timeline, value, etc.) are kept in the same relative order as the spec.
fn add_io_constraints(
    mut env: NodeEnv,
    inputs: &[(String, TripleTag)],
    outputs: &[TripleTag],
    output_dtypes: &[DataType],
    dtypes: &HashMap<String, DataType>,
    info: Info,
    num_dims: usize,
) -> Result<NodeEnv, LocalError> {
    env.override_output_classes(
        output_dtypes
            .iter()
            .zip(outputs.iter().map(|t| &t.timeline)),
        &is_timeline_dtype,
        1,
    );
    for (idx, (arg_name, fn_in_tag)) in inputs
        .iter()
        .filter(|(arg, _)| is_timeline_dtype(&dtypes[arg]))
        .enumerate()
    {
        if fn_in_tag.timeline.quot == Some(Quotient::None) {
            continue;
        }
        let class_name = if let Some(annoted_quot) = &fn_in_tag.timeline.quot_var.spec_var {
            annoted_quot.clone()
        } else {
            let spec_classes = env.get_input_classes();
            assert!(!spec_classes.is_empty());
            if idx < spec_classes.len() - 1 {
                // match up the explicit arguments
                spec_classes[idx + 1].clone()
            } else {
                continue;
            }
        };
        env = super::add_node_eq(arg_name, &class_name, info, env)?;
    }
    let implicit_in = env.get_input_classes()[0].clone();
    for i in 0..num_dims {
        // TODO: allow user to override this
        env = super::add_node_eq(&format!("_dim{i}"), &implicit_in, info, env)?;
    }
    Ok(env)
}

/// Fills timeline tags for the inputs and outputs of a function.
/// # Arguments
/// * `inputs` - The inputs of the function
/// * `outputs` - The output tags of the function
/// * `output_dtypes` - The data types of the outputs
/// * `env` - The current environment
/// * `event_info` - The implicit input and output events for each funclet
/// * `fill_non_tmln` - Whether to fill in non-timeline tags with the last local event
fn fill_io_tags(
    inputs: &mut [(String, TripleTag)],
    outputs: &mut [TripleTag],
    output_dtypes: &[DataType],
    env: &NodeEnv,
    event_info: &InOutEvents,
) {
    // the io has already been expanded, so we need to carry over the annotations
    // for fences and encoders
    for (name, tag) in inputs.iter_mut() {
        fill_tmln_quotient(name, tag, env, START_BLOCK_ID);
        if name.contains("::") {
            let record_name = name.split("::").next().unwrap();
            fill_tmln_quotient(record_name, tag, env, START_BLOCK_ID);
        }
        add_tmln_quotient(
            &event_info.get_implicit_in_name(START_BLOCK_ID),
            tag,
            env,
            START_BLOCK_ID,
        );
    }
    // output_class[0] is the implicit out
    let output_classes = env.get_function_output_classes().to_vec();
    // map from output index to the class name
    let mut annots = HashMap::new();
    for (((out_idx, tag), dt), output_class) in outputs
        .iter_mut()
        .enumerate()
        .zip(output_dtypes.iter())
        .filter(|(_, dt)| is_timeline_dtype(dt))
        .zip(output_classes.iter().skip(1))
    {
        if tag.timeline.quot.is_none() {
            tag.timeline.quot = Some(Quotient::Node);
        }
        if let Some(output_class) = output_class {
            fill_tmln_quotient(
                &MetaVar::new_class_name(output_class).into_string(),
                tag,
                env,
                FINAL_BLOCK_ID,
            );
            if let DataType::Fence(Some(t)) | DataType::Encoder(Some(t)) = dt {
                if let DataType::RemoteObj { all, .. } = &**t {
                    // annotate all of the following elements of the encoder/fence
                    for i in 0..all.len() {
                        annots.insert(i + out_idx + 1, output_class.clone());
                    }
                }
            }
        }
    }
    let implicit_out = event_info.get_implicit_out_name(FINAL_BLOCK_ID);
    assert!(output_classes[0].is_some());
    for (out_idx, tag) in outputs
        .iter_mut()
        .enumerate()
        .zip(output_dtypes)
        .filter_map(|(t, dt)| if is_timeline_dtype(dt) { None } else { Some(t) })
    {
        if let Some(node) = annots.get(&out_idx) {
            fill_tmln_quotient(
                &MetaVar::new_class_name(node).into_string(),
                tag,
                env,
                FINAL_BLOCK_ID,
            );
        } else {
            add_tmln_quotient(&implicit_out, tag, env, FINAL_BLOCK_ID);
        }
    }
}

/// Adds a type constraint to the environment, allowing
/// information from `TripleTag` to override the constraint.
/// # Arguments
/// * `lhs` - The name of the variable to constrain
/// * `rhs` - The constraint to apply to the type variable
/// * `info` - The source info for the constraint
/// * `env` - The current environment
/// # Returns
/// The updated environment
#[allow(clippy::unnecessary_wraps)]
fn add_overrideable_constraint(
    lhs: &str,
    lhs_tag: &TripleTag,
    rhs: &ValQuot,
    info: Info,
    env: NodeEnv,
) -> Result<NodeEnv, LocalError> {
    super::add_overrideable_constraint(lhs, lhs_tag, rhs, info, env, &|dt| &dt.timeline)
}

/// Adds a type annotation for `name` to the environement if the given annotation
/// provides a node matching.
/// # Arguments
/// * `name` - The name of the variable to annotate
/// * `annot` - The annotation to add
/// * `env` - The current environment
/// # Returns
/// The updated environment
fn add_type_annot(
    name: &str,
    annot: &TripleTag,
    info: Info,
    env: NodeEnv,
) -> Result<NodeEnv, LocalError> {
    super::add_type_annot(name, annot, info, env, &|dt| &dt.timeline)
}

/// Unifies an assignment to the variable `lhs` with the given rhs.
/// The lhs name will become a type variable and the rhs will be a constraint on it.
/// # Arguments
/// * `lhs` - The name of the variable being assigned to
/// * `lhs_tag` - The type annotation for the variable being assigned to
/// * `rhs` - The value being assigned to the variable
/// * `specs` - The specs
/// * `env` - The current environment
/// # Returns
/// The updated environment
fn unify_decl(
    lhs: &str,
    lhs_tag: &TripleTag,
    rhs: &HirTerm,
    decl_info: Info,
    mut env: NodeEnv,
) -> Result<NodeEnv, LocalError> {
    if let HirTerm::Var { name, info, tag } = rhs {
        env = add_type_annot(lhs, tag, *info, env)?;
        env = add_var_constraint(lhs, name, *info, env)?;
    }
    add_type_annot(lhs, lhs_tag, decl_info, env)
}

/// Inserts timeline tags into each instruction
fn fill_tmln_tags(
    cfg: &mut Cfg,
    env: &NodeEnv,
    implicit_events: &InOutEvents,
    live_vars: &InOutFacts<LiveVars>,
    dtypes: &HashMap<String, DataType>,
) {
    // insertion index, var name, info, metavar name, block id of new in annotations
    for block in cfg.blocks.values_mut() {
        for (instr_id, stmt) in block.stmts.iter_mut().enumerate() {
            let local_event = implicit_events.get_local_name_after_instr(block.id, instr_id);
            match stmt {
                HirBody::ConstDecl { lhs_tag, .. }
                | HirBody::VarDecl { lhs_tag, .. }
                | HirBody::RefStore {
                    lhs_tags: lhs_tag, ..
                } => add_tmln_quotient(&local_event, lhs_tag, env, block.id),
                HirBody::Op { dests, .. } => {
                    for (_, t) in dests {
                        add_tmln_quotient(&local_event, t, env, block.id);
                    }
                }
                HirBody::InAnnotation(_, tags)
                | HirBody::OutAnnotation(_, tags)
                | HirBody::Hole { dests: tags, .. } => {
                    for (name, tag) in tags {
                        fill_tmln_quotient(name, tag, env, block.id);
                        if name.contains("::") {
                            let record_name = name.split("::").next().unwrap();
                            add_tmln_quotient(record_name, tag, env, block.id);
                        }
                    }
                }
                HirBody::BeginEncoding {
                    encoder,
                    tags,
                    device_vars,
                    ..
                } => {
                    fill_tmln_quotient(&tuple_id(&[encoder.0.clone()]), tags, env, block.id);
                    fill_tmln_quotient(&encoder.0, &mut encoder.1, env, block.id);
                    for (_, dtag) in device_vars.iter_mut() {
                        fill_tmln_quotient(&encoder.0, dtag, env, block.id);
                    }
                }
                HirBody::Submit { dest, tags, .. } => {
                    fill_tmln_quotient(dest, tags, env, block.id);
                }
                HirBody::Sync { dests, tags, .. } => {
                    let nm = tuple_id(
                        &dests
                            .processed()
                            .iter()
                            .map(|(n, _)| n.clone())
                            .collect::<Vec<_>>(),
                    );
                    fill_tmln_quotient(&nm, tags, env, block.id);
                    for (_, dest_tag) in dests.processed_mut() {
                        add_tmln_quotient(&nm, dest_tag, env, block.id);
                    }
                }
                HirBody::Phi { .. } => unreachable!(),
                _ => {}
            }
        }
        fill_terminator_tags(block, env, implicit_events);
    }

    // collect so we don't borrow
    #[allow(clippy::needless_collect)]
    for block in cfg.graph.keys().copied().collect::<Vec<_>>() {
        fill_local_in_annotations(
            block,
            live_vars,
            dtypes,
            &implicit_events.get_implicit_in_name(block),
            cfg,
            env,
        );
    }
}

fn fill_terminator_tags(block: &mut BasicBlock, env: &NodeEnv, implicit_events: &InOutEvents) {
    let local_event = implicit_events.get_local_name_after_instr(block.id, block.stmts.len());
    match &mut block.terminator {
        Terminator::CaptureCall { dests, call, .. } | Terminator::Call(dests, call, ..) => {
            for (dest, tag) in dests.iter_mut() {
                fill_tmln_quotient(dest, tag, env, block.id);
                if dest.contains("::") {
                    let record_name = dest.split("::").next().unwrap();
                    add_tmln_quotient(record_name, tag, env, block.id);
                }
                add_tmln_quotient(&local_event, tag, env, block.id);
            }
            fill_tmln_quotient(
                &tuple_id(&dests.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>()),
                &mut call.tag,
                env,
                block.id,
            );
        }
        Terminator::Next(..)
        | Terminator::Select { .. }
        | Terminator::None(_)
        | Terminator::FinalReturn(..)
        | Terminator::Return { .. }
        | Terminator::Yield(..) => {}
    }
}

/// Adds an annotation for all non-timeline live-ins to have a quotient matching the
/// current local event.
fn fill_local_in_annotations(
    block: usize,
    live_vars: &InOutFacts<LiveVars>,
    dtypes: &HashMap<String, DataType>,
    last_loc: &str,
    cfg: &mut Cfg,
    env: &NodeEnv,
) {
    let preds = cfg.predecessors(block);
    // hash sets can be viewed in debugger more easily
    let mut live_ins: std::collections::HashSet<_> = live_vars
        .get_in_fact(block)
        .live_set
        .iter()
        .cloned()
        .chain(
            preds
                .iter()
                .flat_map(|b| live_vars.get_out_fact(*b).live_set.iter().cloned()),
        )
        .collect();
    for term_dest in preds
        .iter()
        .filter_map(|b| cfg.blocks[b].terminator.get_defs())
        .flatten()
    {
        live_ins.remove(&term_dest);
    }
    let mut annots = Vec::new();
    for live_in in live_ins {
        if let Some(dt) = &dtypes.get(&live_in) {
            if !is_timeline_dtype(dt)
                && (!live_in.contains("::")
                    || !is_timeline_dtype(&dtypes[live_in.split("::").next().unwrap()]))
            {
                let mut tag = TripleTag::new_unspecified();
                add_tmln_quotient(last_loc, &mut tag, env, block);
                annots.push((live_in, tag));
            }
        }
    }
    let starting_info = cfg.blocks[&block].get_starting_info();
    cfg.blocks
        .get_mut(&block)
        .unwrap()
        .stmts
        .insert(0, HirBody::InAnnotation(starting_info, annots));
}

/// Overwrites the tag with the class of the given meta variable
fn fill_tmln_quotient(name: &str, tag: &mut TripleTag, env: &NodeEnv, block_id: usize) {
    super::fill_quotient(name, tag, env, block_id, SpecType::Timeline, false, &|dt| {
        &mut dt.timeline
    });
}

/// Attempts to add a timeline quotient to the given tag. Does nothing
/// if the tag already has a timeline quotient.
fn add_tmln_quotient(name: &str, tag: &mut TripleTag, env: &NodeEnv, block_id: usize) {
    super::fill_quotient(name, tag, env, block_id, SpecType::Timeline, true, &|dt| {
        &mut dt.timeline
    });
}

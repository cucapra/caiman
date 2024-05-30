use std::{
    collections::{hash_map::Entry, HashMap, VecDeque},
    vec,
};

use crate::{
    error::{Info, LocalError},
    lower::{
        sched_hir::{
            cfg::{BasicBlock, Cfg, Edge, FINAL_BLOCK_ID, START_BLOCK_ID},
            Hir, HirBody, HirFuncCall, Terminator, TripleTag,
        },
        tuple_id,
    },
    parse::ast::{DataType, Quotient, SchedTerm, SpecType},
    typing::{is_timeline_dtype, Context, MetaVar, NodeEnv, SchedOrExtern, SpecInfo, ValQuot},
};

use super::{add_constraint, add_node_eq, add_var_constraint};

const LOCAL_STEM: &str = "_loc";

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
) -> Result<(), LocalError> {
    let env = spec_info.nodes.clone();
    let mut overrides = Vec::new();
    for i in &cfg.blocks[&START_BLOCK_ID].stmts {
        if let HirBody::InAnnotation(_, tags) = i {
            overrides.extend(tags.iter().cloned());
        }
    }
    let env = add_io_constraints(
        env,
        inputs,
        &overrides,
        outputs,
        output_dtypes,
        dtypes,
        info,
    )?;
    let (env, implicit_events) = unify_nodes(cfg, &spec_info.sig.input[0].0, dtypes, ctx, env)?;
    //add_implicit_annotations(cfg, &implicit_events);
    Ok(())
}

/// A struct containing the implicit input and output events for each funclet in the CFG.
struct InOutEvents(HashMap<usize, (TripleTag, TripleTag)>);

impl InOutEvents {
    /// Gets the implicit input event for the given block
    pub fn get_implicit_input(&self, block: usize) -> &TripleTag {
        &self.0[&block].0
    }

    /// Gets the implicit output event for the given block
    pub fn get_implicit_output(&self, block: usize) -> &TripleTag {
        &self.0[&block].1
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
    env = add_var_constraint(implicit_in, &format!("{LOCAL_STEM}0"), Info::default(), env)?;
    seen.insert(START_BLOCK_ID, 0);
    let mut node_q = VecDeque::new();
    node_q.push_back(START_BLOCK_ID);
    let mut block_loc_events = HashMap::new();
    // the last globally used local event number
    let mut latest_loc = 0;
    let mut implicit_annotations = ImplicitAnnotations::new(cfg);
    while let Some(bb) = node_q.pop_front() {
        let bb = &cfg.blocks[&bb];
        // the last local event number for the current path
        let mut last_loc = seen[&bb.id];
        let in_local_event = last_loc;
        implicit_annotations.set_cur_block(bb.id);
        env = implicit_annotations.unify_inputs(last_loc, env)?;
        env = unify_instrs(
            bb,
            env,
            &mut last_loc,
            &mut latest_loc,
            &mut implicit_annotations,
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
        env = implicit_annotations.unify_outputs(last_loc, env)?;
        block_loc_events.insert(
            bb.id,
            (
                format!("{LOCAL_STEM}{in_local_event}"),
                format!("{LOCAL_STEM}{last_loc}"),
            ),
        );
        for succ in cfg.graph[&bb.id].targets() {
            match seen.entry(succ) {
                Entry::Occupied(entry) => {
                    // merge last local events to be equal
                    // this must be the case since the timeline doesn't have selects
                    if entry.get() != &last_loc {
                        // println!("{env:#?}");
                        env = add_var_constraint(
                            &format!("{LOCAL_STEM}{last_loc}"),
                            &format!("{LOCAL_STEM}{}", entry.get()),
                            cfg.blocks[&succ]
                                .stmts
                                .first()
                                .map_or(cfg.blocks[&succ].src_loc, Hir::get_info),
                            env,
                        )?;
                    }
                }
                Entry::Vacant(entry) => {
                    // input event to successor is the current last event
                    entry.insert(last_loc);
                    node_q.push_back(succ);
                }
            }
        }
    }
    let io_evs = into_input_output_annotations(cfg, &env, &block_loc_events);
    Ok((env, io_evs))
}

/// Converts a map from block id to the first and last local events of the block
/// to a map from block id to the implicit input and output events of the block.
fn into_input_output_annotations(
    cfg: &Cfg,
    env: &NodeEnv,
    first_last_events: &HashMap<usize, (String, String)>,
) -> InOutEvents {
    let mut res = HashMap::new();
    for (block, (first, _)) in first_last_events {
        let mut in_ev = TripleTag::new_unspecified();
        in_ev.timeline.quot = if *block == START_BLOCK_ID {
            Some(Quotient::Input)
        } else {
            Some(Quotient::Node)
        };
        in_ev.timeline.quot_var.spec_var = Some(env.get_node_name(first).unwrap());
        let mut out_ev = TripleTag::new_unspecified();
        out_ev.timeline.quot_var.spec_var = Some(
            env.get_node_name(&first_last_events[&cfg.get_continuation_output_block(*block)].1)
                .unwrap(),
        );
        out_ev.timeline.quot = Some(Quotient::Node);
        res.insert(*block, (in_ev, out_ev));
    }
    InOutEvents(res)
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
/// the CFG
/// * `latest_loc` - The last used local event number globally for the entire
/// CFG
/// * `output_annotations` - Will be pushed with annotations for the final
/// local event
/// # Returns
/// The updated environment
/// # Errors
/// If a constraint cannot be added to the environment
#[allow(clippy::too_many_lines)]
fn unify_instrs(
    bb: &BasicBlock,
    mut env: NodeEnv,
    last_loc: &mut i32,
    latest_loc: &mut i32,
    implicit_annotations: &mut ImplicitAnnotations,
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
                    } else {
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
                    } else {
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
                dests.initial(),
                srcs.initial(),
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
            | HirBody::DeviceCopy { .. }
            | HirBody::EncodeDo { .. }
            | HirBody::Hole { .. } => env,
        }
    }
    Ok(env)
}

/// Unifies the terminator of a basic block.
/// # Arguments
/// * `bb` - The basic block to unify
/// * `env` - The current environment
/// * `last_loc` - The last local event number along the current path through
/// the CFG
/// * `latest_loc` - The last used local event number globally for the entire
/// CFG
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
        Terminator::CaptureCall { .. } => unreachable!(),
        Terminator::Call(dests, call) => unify_call(
            dests,
            call,
            &cfg.blocks[&cfg.graph[&bb.id].next().unwrap()],
            env,
            last_loc,
            latest_loc,
            ctx,
            dtypes,
        ),
        Terminator::Return {
            dests, rets, info, ..
        } => {
            // pass through is ignored (like next)
            // the destination tag is the tag for the merged node, we handle this
            for ((dest, _), ret) in dests.iter().zip(rets.iter()) {
                env = add_var_constraint(dest, ret, *info, env)?;
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
                todo!("None outputs")
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
    // the annotation is not tupled, so it refers to the fence
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

/// Returns true if the successor of `bb` has an implicit input annotation
/// Requires that `bb` has only one successor
fn has_implicit_input_annot(bb: &BasicBlock) -> bool {
    for stmt in &bb.stmts {
        if let HirBody::InAnnotation(_, annots) = stmt {
            if annots.iter().find(|(name, _)| name == "input").is_some() {
                return true;
            }
        }
    }
    false
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
                let t = dtypes
                    .get(arg)
                    .unwrap_or_else(|| panic!("Missing type info for {arg}"));
                if is_timeline_dtype(t) {
                    Some(MetaVar::new_var_name(arg))
                } else {
                    None
                }
            }))
            .collect(),
    );
    if !env.spec_has_match(&call_constraint) {
        // there is no call for this function in the spec, so this isn't something to worry about

        if has_implicit_input_annot(succ) {
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
    env = add_constraint(
        &format!("{LOCAL_STEM}{}", *latest_loc),
        &ValQuot::Extract(MetaVar::new_var_name(&tuple_name), 0),
        call.info,
        env,
    )?;
    for (idx, (dest, tag)) in dests
        .iter()
        .filter(|(x, _)| {
            is_timeline_dtype(
                dtypes
                    .get(x)
                    .unwrap_or_else(|| panic!("Missing type info for {x}")),
            )
        })
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
    inputs: &mut [(String, TripleTag)],
    input_overrides: &[(String, TripleTag)],
    outputs: &mut [TripleTag],
    output_dtypes: &[DataType],
    dtypes: &HashMap<String, DataType>,
    info: Info,
) -> Result<NodeEnv, LocalError> {
    env.override_output_classes(
        output_dtypes.iter().zip(outputs.iter().map(|t| &t.value)),
        &is_timeline_dtype,
        1,
    );
    for (name, tag) in input_overrides {
        for (n2, t2) in inputs.iter_mut() {
            if n2 == name {
                t2.set_specified_info(tag.clone());
            }
        }
    }
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
    Ok(env)
}

/// Adds a type constraint to the environment, allowing value
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
/// provides a value node matching.
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
    rhs: &SchedTerm,
    decl_info: Info,
    mut env: NodeEnv,
) -> Result<NodeEnv, LocalError> {
    if let SchedTerm::Var { name, info, tag } = rhs {
        env = add_type_annot(lhs, &TripleTag::from_opt(tag), *info, env)?;
        env = add_var_constraint(lhs, name, *info, env)?;
    }
    add_type_annot(lhs, lhs_tag, decl_info, env)
}

/// Inserts timeline tags into each instruction
fn fill_tmln_tags(cfg: &mut Cfg, env: &NodeEnv, implicit_events: &InOutEvents) {
    for (_, block) in &mut cfg.blocks {
        for stmt in &mut block.stmts {
            match stmt {
                // HirBody::ConstDecl { lhs, lhs_tag, .. }
                // | HirBody::VarDecl { lhs, lhs_tag, .. }
                // | HirBody::RefStore {
                //     lhs,
                //     lhs_tags: lhs_tag,
                //     ..
                // } => {
                //     fill_tmln_quotient(lhs, lhs_tag, env, block.id);
                // }
                // HirBody::Op { dests, .. } => {
                //     for (d, t) in dests {
                //         fill_val_quotient(d, t, env, block.id);
                //     }
                // }
                HirBody::InAnnotation(_, tags) | HirBody::OutAnnotation(_, tags) => {
                    for (name, tag) in tags {
                        fill_tmln_quotient(name, tag, env, block.id);
                    }
                }
                HirBody::BeginEncoding { encoder, tags, .. } => {
                    fill_tmln_quotient(&tuple_id(&[encoder.0.clone()]), tags, env, block.id);
                    fill_tmln_quotient(&encoder.0, &mut encoder.1, env, block.id);
                }
                HirBody::Submit { dest, tags, .. } => {}
                // HirBody::Hole(_)
                // | HirBody::RefLoad { .. }
                // | HirBody::BeginEncoding { .. }
                // | HirBody::Submit { .. } => {}
                // HirBody::Phi { dest, info, .. } => {
                //     insertions.push((
                //         idx,
                //         HirBody::InAnnotation(
                //             *info,
                //             vec![(dest.clone(), construct_new_tag(dest, env, block.id))],
                //         ),
                //     ));
                // }
                // HirBody::Sync { dests, .. } => {
                //     for (dest, dest_tag) in dests.processed_mut() {
                //         fill_val_quotient(dest, dest_tag, env, block.id);
                //     }
                // }
                _ => {}
            }
            match &mut block.terminator {
                // Terminator::CaptureCall { dests, call, .. } => {
                //     for (dest, tag) in dests.iter_mut() {
                //         fill_val_quotient(dest, tag, env, block.id);
                //     }
                //     fill_val_quotient(
                //         &tuple_id(&dests.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>()),
                //         &mut call.tag,
                //         env,
                //         block.id,
                //     );
                // }
                // Terminator::Select { dests, tag, .. } => {
                //     for (dest, tag) in dests {
                //         fill_val_quotient(dest, tag, env, block.id);
                //     }
                //     fill_val_quotient(&selects[&block.id], tag, env, block.id);
                // }
                // Terminator::Call(..) | Terminator::None(..) => unreachable!(),
                // // TODO: check the return, I think this is right bc returns should be handled
                // // by Phi nodes
                // Terminator::Next(..)
                // | Terminator::FinalReturn(..)
                // | Terminator::Return { .. }
                // | Terminator::Yield(..) => {}
                _ => {}
            }
        }
    }
}

fn fill_tmln_quotient(name: &str, tag: &mut TripleTag, env: &NodeEnv, block_id: usize) {
    super::fill_quotient(name, tag, env, block_id, SpecType::Timeline);
}

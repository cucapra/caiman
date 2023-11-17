use caiman::assembly::ast::{self as asm, FuncletArgument, Hole};

use crate::{
    error::{type_error, LocalError},
    lower::data_type_to_ffi_type,
    parse::ast::{
        Arg, Flow, FullType, Quotient, QuotientReference, SchedLiteral, SchedTerm, SchedulingFunc,
        Tag,
    },
};
use caiman::ir;

use super::{
    cfg::{
        self,
        hir::{Hir, Terminator},
        BasicBlock, Cfg, InOutFacts, LiveVars,
    },
    global_context::{Context, SpecType},
};

/// A vector of commands with holes.
/// A hole in a command means `???`
type CommandVec = Vec<Hole<asm::Command>>;

/// Gets the name of a temporary variable with the given id
fn temp_var_name(temp_id: usize) -> String {
    format!("_t{temp_id}")
}

/// Lowers a flattened declaration statement into a caiman assembly command
/// # Returns
/// A tuple containing the commands that implement the statement
/// and the next available temporary id
fn lower_flat_decl(
    dest: &str,
    dest_tag: &Option<FullType>,
    rhs: &SchedTerm,
    temp_id: usize,
) -> (CommandVec, usize) {
    if let SchedTerm::Lit {
        lit: SchedLiteral::Int(_x),
        tag: _tag,
        ..
    } = rhs
    {
        let dest_tag = dest_tag
            .as_ref()
            .expect("We require all variables to have type annotations");
        assert_eq!(dest_tag.tags.len(), 1);
        let temp_node_name = format!("_t{temp_id}");
        let temp = asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(temp_node_name.clone())),
            node: asm::Node::AllocTemporary {
                place: Some(ir::Place::Local),
                buffer_flags: Some(ir::BufferFlags::new()),
                storage_type: Some(data_type_to_ffi_type(&dest_tag.base.base)),
            },
        });
        let mv = asm::Command::Node(asm::NamedNode {
            name: None,
            node: asm::Node::LocalDoBuiltin {
                operation: Some(tag_to_quot(&dest_tag.tags[0])),
                // no inputs
                inputs: Some(Vec::new()),
                outputs: Some(vec![Some(asm::NodeId(temp_node_name.clone()))]),
            },
        });
        let rd_ref = asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(dest.to_string())),
            node: asm::Node::ReadRef {
                source: Some(asm::NodeId(temp_node_name)),
                storage_type: Some(data_type_to_ffi_type(&dest_tag.base.base)),
            },
        });
        (vec![Some(temp), Some(mv), Some(rd_ref)], temp_id + 1)
    } else {
        todo!()
    }
}

/// Lowers a scheduling statement into a caiman assembly command
/// # Returns
/// A tuple containing the commands that implement the statement
/// and the next available temporary id
fn lower_instr(s: &Hir, temp_id: usize) -> (CommandVec, usize) {
    match s {
        Hir::Decl {
            lhs, rhs, lhs_tag, ..
        } => {
            assert_eq!(lhs.len(), 1);
            lower_flat_decl(lhs, lhs_tag, rhs, temp_id)
        }
        _ => todo!(),
    }
}

/// Gets the quotient for a particular spec type from a list of tags
#[allow(dead_code)]
fn get_quotient(
    specs: &Specs,
    tag: &Option<Vec<Tag>>,
    qtype: SpecType,
) -> asm::Hole<asm::Quotient> {
    if let Some(tags) = tag {
        for t in tags {
            if let res @ Some(_) = t.quot_var.as_ref().and_then(|qr| {
                if qr.spec_name == specs.value.0 && qtype == SpecType::Value {
                    return Some(tag_to_quot(t));
                }
                if qr.spec_name == specs.timeline.0 && qtype == SpecType::Timeline {
                    return Some(tag_to_quot(t));
                }
                if qr.spec_name == specs.spatial.0 && qtype == SpecType::Spatial {
                    return Some(tag_to_quot(t));
                }
                None
            }) {
                return res;
            }
        }
    }
    None
}

/// A reference to a cfg and a basic block to query information about the block
/// and its relation to other blocks in the cfg
struct BlockRef<'a> {
    block: &'a BasicBlock,
    cfg: &'a Cfg,
}

impl<'a> BlockRef<'a> {
    const fn new(block: &'a BasicBlock, cfg: &'a Cfg) -> Self {
        Self { block, cfg }
    }

    /// Gets the next blocks in the cfg as `FuncletIds`
    fn next_blocks(&self) -> Vec<asm::Hole<asm::FuncletId>> {
        match &self.block.terminator {
            Terminator::Return(_) => vec![],
            Terminator::Select(_) => {
                let mut e = self
                    .cfg
                    .successors(self.block.id)
                    .into_iter()
                    .map(|id| asm::FuncletId(self.cfg.funclet_id(id)));
                let mut res = vec![];
                if let Some(true_block) = e.next() {
                    res.push(Some(true_block));
                }
                if let Some(false_block) = e.next() {
                    res.push(Some(false_block));
                }
                assert_eq!(res.len(), 2);
                res
            }
            Terminator::Call(..) => todo!(),
            Terminator::None => {
                let e = self
                    .cfg
                    .successors(self.block.id)
                    .into_iter()
                    .map(|id| Some(asm::FuncletId(self.cfg.funclet_id(id))));
                let res: Vec<_> = e.collect();
                assert_eq!(res.len(), 1);
                res
            }
        }
    }
}

/// Lowers a basic block terminator into a caiman assembly command
/// # Returns
/// A tuple containing the commands that implement the terminator
/// and the next available temporary id
fn lower_terminator(
    t: &Terminator,
    temp_id: usize,
    cfg: &BlockRef<'_>,
    specs: &Specs,
    live_vars: &InOutFacts<LiveVars>,
) -> CommandVec {
    // we do not return the new `temp_id` because this is the last instruction
    // in the block
    match t {
        Terminator::Return(Some(name)) => {
            vec![Some(asm::Command::TailEdge(asm::TailEdge::Return {
                return_values: Some(vec![Some(asm::NodeId(name.clone()))]),
            }))]
        }
        Terminator::Select(guard_name) => lower_select(guard_name, temp_id, cfg, specs, live_vars),
        Terminator::Return(_) => panic!("Return not flattened or its a void return!"),
        _ => todo!(),
    }
}

/// Lowers a select terminator into a series of caiman assembly commands
/// # Returns
/// The commands that implement the terminator
fn lower_select(
    guard_name: &str,
    temp_id: usize,
    cfg: &BlockRef<'_>,
    specs: &Specs,
    live_vars: &InOutFacts<LiveVars>,
) -> CommandVec {
    let djoin_id = temp_id;
    let djoin_name = temp_var_name(djoin_id);
    let join = temp_id + 1;
    let join_var = temp_var_name(join);
    vec![
        Some(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(djoin_name.clone())),
            node: asm::Node::DefaultJoin,
        })),
        Some(asm::Command::Node(asm::NamedNode {
            name: Some(asm::NodeId(join_var.clone())),
            node: asm::Node::SerializedJoin {
                funclet: cfg
                    .block
                    .join_block
                    .map(|id| asm::FuncletId(cfg.cfg.funclet_id(id))),
                captures: None,
                continuation: Some(asm::NodeId(djoin_name)),
            },
        })),
        Some(asm::Command::TailEdge(asm::TailEdge::ScheduleSelect {
            /// TODO don't hardcode
            value_operation: Some(asm::Quotient::Node(Some(asm::RemoteNodeId {
                /// the select in the spec must store into the node `r`
                node: Some(asm::NodeId(String::from("r"))),
                funclet: Some(specs.value.clone()),
            }))),
            // TODO
            timeline_operation: Some(asm::Quotient::None(Some(asm::RemoteNodeId {
                node: None,
                funclet: Some(specs.timeline.clone()),
            }))),
            // TODO
            spatial_operation: Some(asm::Quotient::None(Some(asm::RemoteNodeId {
                node: None,
                funclet: Some(specs.spatial.clone()),
            }))),
            condition: Some(asm::NodeId(guard_name.to_string())),
            callee_funclet_ids: Some(cfg.next_blocks()),
            callee_arguments: Some(
                live_vars
                    .get_out_fact(cfg.block.id)
                    .live_set()
                    .iter()
                    .cloned()
                    .map(asm::NodeId)
                    .map(Some)
                    .collect(),
            ),
            continuation_join: Some(asm::NodeId(join_var)),
        })),
    ]
}

/// Scheduling funclet specs
struct Specs {
    value: asm::FuncletId,
    timeline: asm::FuncletId,
    spatial: asm::FuncletId,
}

/// Converts a quotient reference (the part that refers to a variable in a spec)
/// to a remote node id in the assembly
fn quot_ref_to_remote_node(qr: &QuotientReference) -> asm::RemoteNodeId {
    asm::RemoteNodeId {
        node: qr.spec_var.clone().map(asm::NodeId),
        funclet: Some(asm::FuncletId(qr.spec_name.clone())),
    }
}

/// Gets the assembly quotient from a high level caiman tag
fn tag_to_quot(t: &Tag) -> asm::Quotient {
    t.quot.as_ref().map_or_else(
        || asm::Quotient::None(t.quot_var.as_ref().map(quot_ref_to_remote_node)),
        |x| match x {
            Quotient::Node => asm::Quotient::Node(t.quot_var.as_ref().map(quot_ref_to_remote_node)),
            Quotient::Input => {
                asm::Quotient::Input(t.quot_var.as_ref().map(quot_ref_to_remote_node))
            }
            Quotient::Output => {
                asm::Quotient::Output(t.quot_var.as_ref().map(quot_ref_to_remote_node))
            }
            Quotient::None => asm::Quotient::None(t.quot_var.as_ref().map(quot_ref_to_remote_node)),
        },
    )
}

/// Converts a hlc tag to a tag in the assembly
fn tag_to_tag(t: &Tag) -> asm::Tag {
    asm::Tag {
        quot: tag_to_quot(t),
        // TODO: this is a mistake in the IR/assembly, the flow should be able
        // to be a hole
        flow: t.flow.as_ref().map_or(ir::Flow::Usable, |f| match f {
            Flow::Dead => ir::Flow::Dead,
            Flow::Need => ir::Flow::Need,
            Flow::Usable => ir::Flow::Usable,
            Flow::Save => ir::Flow::Save,
        }),
    }
}

/// Converts a high level caiman function argument into a funclet argument
fn hlc_arg_to_asm_arg(arg: &Arg<FullType>) -> FuncletArgument {
    let ft = &arg.1;
    FuncletArgument {
        name: Some(asm::NodeId(arg.0.to_string())),
        typ: super::data_type_to_local_type(&ft.base.base),
        tags: ft.tags.iter().map(tag_to_tag).collect(),
    }
}

/// Information about a high level caiman function
struct FuncInfo {
    input: Vec<Arg<FullType>>,
    output: Arg<FullType>,
}

/// Lowers a basic block into a caiman assembly funclet
///
fn lower_block(
    func: &FuncInfo,
    specs: &Specs,
    cfg: &Cfg,
    b: &BasicBlock,
    live_vars: &InOutFacts<LiveVars>,
) -> asm::Funclet {
    let mut commands = vec![];
    let mut temp_id = 0;
    for cmd in &b.stmts {
        let (mut new_cmds, new_id) = lower_instr(cmd, temp_id);
        temp_id = new_id;
        commands.append(&mut new_cmds);
    }
    commands.extend(lower_terminator(
        &b.terminator,
        temp_id,
        &BlockRef::new(b, cfg),
        specs,
        live_vars,
    ));
    asm::Funclet {
        kind: ir::FuncletKind::ScheduleExplicit,
        header: asm::FuncletHeader {
            name: asm::FuncletId(cfg.funclet_id(b.id)),
            args: block_inputs(b.id, func, specs, live_vars),
            ret: block_outputs(b.id, func, specs, live_vars),
            binding: asm::FuncletBinding::ScheduleBinding(asm::ScheduleBinding {
                implicit_tags: None,
                value: Some(specs.value.clone()),
                timeline: Some(specs.timeline.clone()),
                spatial: Some(specs.spatial.clone()),
            }),
        },
        commands,
    }
}

/// Gets a vector of tags for each of the specs, all with a none quotient
/// and a usable flow
fn none_usable(specs: &Specs) -> Vec<asm::Tag> {
    vec![
        asm::Tag {
            quot: asm::Quotient::None(Some(asm::RemoteNodeId {
                funclet: Some(specs.value.clone()),
                node: None,
            })),
            flow: ir::Flow::Usable,
        },
        asm::Tag {
            quot: asm::Quotient::None(Some(asm::RemoteNodeId {
                funclet: Some(specs.timeline.clone()),
                node: None,
            })),
            flow: ir::Flow::Usable,
        },
        asm::Tag {
            quot: asm::Quotient::None(Some(asm::RemoteNodeId {
                funclet: Some(specs.spatial.clone()),
                node: None,
            })),
            flow: ir::Flow::Usable,
        },
    ]
}

/// Gets the input arguments for each block based on the block's live in variables
fn block_inputs(
    block_id: usize,
    func: &FuncInfo,
    specs: &Specs,
    live_vars: &InOutFacts<LiveVars>,
) -> Vec<FuncletArgument> {
    if block_id == cfg::START_BLOCK_ID {
        func.input.iter().map(hlc_arg_to_asm_arg).collect()
    } else {
        live_vars
            .get_in_fact(block_id)
            .live_set()
            .iter()
            .map(|var| {
                FuncletArgument {
                    name: Some(asm::NodeId(var.clone())),
                    // TODO: support more types
                    typ: asm::TypeId::Local(String::from("i64")),
                    // TODO: don't hardcode this
                    tags: none_usable(specs),
                }
            })
            .collect()
    }
}

/// Gets the return arguments of a funclet based on the block's live out variables
fn block_outputs(
    block_id: usize,
    func: &FuncInfo,
    specs: &Specs,
    live_vars: &InOutFacts<LiveVars>,
) -> Vec<FuncletArgument> {
    if block_id == cfg::START_BLOCK_ID {
        vec![hlc_arg_to_asm_arg(&func.output)]
    } else {
        // I don't think this is right
        live_vars
            .get_out_fact(block_id)
            .live_set()
            .iter()
            .map(|var| FuncletArgument {
                name: Some(asm::NodeId(var.clone())),
                // TODO: support more types
                typ: asm::TypeId::Local(String::from("i64")),
                // TODO: don't hardcode this
                tags: none_usable(specs),
            })
            .collect()
    }
}

/// Lower a scheduling function into one or more caiman assembly funclet.
/// # Errors
/// Returns an error if the function is missing a spec.
pub fn lower_schedule(
    ctx: &Context,
    func: SchedulingFunc,
) -> Result<Vec<asm::Funclet>, LocalError> {
    let cfg = Cfg::new(&func.name, func.statements);
    let live_vars = cfg::analyze(&cfg, &cfg::LiveVars::top());
    let mut val = None;
    let mut timeline = None;
    let mut spatial = None;
    if func.specs.len() > 3 {
        return Err(type_error(func.info, "Too many specs"));
    }
    for spec in &func.specs {
        match ctx.specs.get(spec) {
            Some(SpecType::Value) => val = Some(spec.to_string()),
            Some(SpecType::Timeline) => timeline = Some(spec.to_string()),
            Some(SpecType::Spatial) => spatial = Some(spec.to_string()),
            None => return Err(type_error(func.info, &format!("Spec '{spec}' not found"))),
        }
    }
    let specs = Specs {
        value: val
            .map(asm::FuncletId)
            .ok_or_else(|| type_error(func.info, "Missing value spec"))?,
        timeline: timeline
            .map(asm::FuncletId)
            .ok_or_else(|| type_error(func.info, "Missing timeline spec"))?,
        spatial: spatial
            .map(asm::FuncletId)
            .ok_or_else(|| type_error(func.info, "Missing spatial spec"))?,
    };
    let finfo = FuncInfo {
        input: func.input,
        output: (
            String::new(),
            func.output.expect("Functions must return values for now"),
        ),
    };
    // if cfg.blocks[cfg::FINAL_BLOCK_ID].stmts.is_empty() {
    //     cfg.blocks.remove(cfg::FINAL_BLOCK_ID);
    // }
    Ok(cfg
        .blocks
        .iter()
        .map(|bb| lower_block(&finfo, &specs, &cfg, bb, &live_vars))
        .collect())
}

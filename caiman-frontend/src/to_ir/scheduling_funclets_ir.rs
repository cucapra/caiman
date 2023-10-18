use std::collections::HashSet;

use crate::syntax::ast;
use crate::syntax::ast::scheduling as sch;
use caiman::assembly::ast as asm;

use super::funclet_util::vf_node_with_name;

use super::function_classes::FunctionClassContext;
use super::typing;
//use super::typing::TypingContext;
use super::value_funclets::ValueFunclet;
//use caiman::ir;

#[derive(Clone, Debug)]
pub struct Operation
{
    pub kind: sch::FullSchedulable,

    pub value_funclet_node: Option<asm::NodeId>,
    pub value_funclet_name: Option<asm::FuncletId>,
}

#[derive(Clone, Debug)]
pub struct Expr
{
    pub operation: Option<Operation>,
    pub storage_type: Option<asm::TypeId>,
    // TODO: info here about locality of allocation
}

#[derive(Clone, Debug)]
pub enum UnsplitStmt
{
    Let
    {
        x: String, e: sch::Hole<Expr>
    },
}

impl UnsplitStmt
{
    fn variables_used(&self) -> Vec<String>
    {
        match self {
            UnsplitStmt::Let {
                e: sch::Hole::Filled(Expr { operation: Some(op), .. }), ..
            } => match op.clone().kind {
                sch::FullSchedulable::Call(_, args)
                | sch::FullSchedulable::CallExternal(_, args) => {
                    args.into_iter().filter_map(|h| h.to_option_move()).collect()
                },
                _ => vec![],
            },
            _ => vec![],
        }
    }

    fn expr_kind(&self) -> Option<sch::FullSchedulable>
    {
        match self {
            UnsplitStmt::Let {
                e: sch::Hole::Filled(Expr { operation: Some(op), .. }), ..
            } => Some(op.kind.clone()),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UnsplitFunclet
{
    pub stmts: Vec<sch::Hole<UnsplitStmt>>,
    pub inputs: Vec<ast::Arg<sch::Type>>,
    pub output: sch::Type, // Should maybe be arg, but AST type isn't for now

    // XXX is this always return? If not, should make an "UnsplitTailEdge" type
    // Also, this should be a hole if AST return changes to a hole too
    pub returned_var: String,
}

#[derive(Clone, Debug)]
pub enum SplitStmt
{
    Unsplit(UnsplitStmt),
    // Inline join with default continuation
    InlineDefaultJoin(usize /* TODO captures */),
}

#[derive(Clone, Debug)]
pub enum TailEdge
{
    Return(String),
    ScheduleCall
    {
        callee_funclet_id: String,
        // TODO callee arguments
    },
}

#[derive(Clone, Debug)]
pub struct SplitFunclet
{
    pub stmts: Vec<sch::Hole<SplitStmt>>,
    pub inputs: Vec<ast::Arg<sch::Type>>,
    pub output: ast::Arg<sch::Type>,
    pub tail_edge: TailEdge,
}

#[derive(Clone, Debug)]
pub struct TotalFunclet
{
    pub split_funclets: Vec<SplitFunclet>,

    // XXX
    // FOR NOW: timeline & space funclets will "distribute" over all the sub-funclets.
    // This is almost certainly wrong in any case where the timeline/space funclets
    // are nontrivial
    pub timeline_funclet: Option<String>,
    pub spatial_funclet: Option<String>,

    pub name: String,
}

pub fn ast_to_total_funclet(
    function_class_ctx: &FunctionClassContext,
    //typing_ctx: &mut TypingContext,
    funclet_being_scheduled: &ValueFunclet,
    scheduling_funclet: &sch::SchedulingFunclet,
) -> TotalFunclet
{
    let u = ast_to_unsplit(
        function_class_ctx,
        //typing_ctx,
        funclet_being_scheduled,
        scheduling_funclet,
    );

    //println!("UNSPLIT: {:#?}", u);

    let split_funclets = split(u);
    //println!("SPLIT: {:#?}", split_funclets);

    TotalFunclet {
        split_funclets,
        timeline_funclet: scheduling_funclet.timeline_funclet.clone(),
        spatial_funclet: scheduling_funclet.spatial_funclet.clone(),
        name: scheduling_funclet.name.clone(),
    }
}

fn translate_expr(
    expr: &sch::Expr,
    funclet_being_scheduled: &ValueFunclet,
    function_class_ctx: &FunctionClassContext,
) -> sch::Hole<Expr>
{
    let (info, kind) = expr;
    let (value_var, full) = match kind {
        ast::scheduling::Hole::Filled(ast::scheduling::ExprKind::Simple { value_var, full }) => {
            (value_var, full)
        },
        _ => return sch::Hole::Vacant,
    };

    // TODO obviously hole-able stuff should be possible here (that would be a case where search
    // for var fails)
    let vf_node = vf_node_with_name(funclet_being_scheduled, &value_var)
        .unwrap_or_else(|| panic!("Scheduling an unknown variable {} at {:?}", value_var, info));
    let operation = Some(Operation {
        value_funclet_name: Some(funclet_being_scheduled.0.header.name.clone()),
        value_funclet_node: vf_node.name.clone(),
        kind: full.clone(),
    });
    let storage_type =
        typing::type_of_asm_node(&vf_node.node, funclet_being_scheduled, function_class_ctx);
    sch::Hole::Filled(Expr { operation, storage_type })
}

fn ast_to_unsplit(
    function_class_ctx: &FunctionClassContext,
    //typing_ctx: &mut TypingContext,
    funclet_being_scheduled: &ValueFunclet,
    scheduling_funclet: &sch::SchedulingFunclet,
) -> UnsplitFunclet
{
    let returned_var = scheduling_funclet
        .statements
        .iter()
        .find_map(|(_info, kind_hole)| match kind_hole {
            sch::Hole::Filled(sch::StmtKind::Return(x)) => Some(x),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{:?}: No return for funclet", scheduling_funclet.info))
        .to_string();

    let stmts = scheduling_funclet
        .statements
        .iter()
        .filter_map(|(_info, h)| match h {
            sch::Hole::Filled(sch::StmtKind::Return(_)) => None,
            sch::Hole::Filled(sch::StmtKind::Let(x, orig_e)) => {
                // schedule expr 2 node combo!
                let e = translate_expr(orig_e, funclet_being_scheduled, function_class_ctx);
                Some(sch::Hole::Filled(UnsplitStmt::Let { x: x.clone(), e }))
            },
            sch::Hole::Vacant => Some(sch::Hole::Vacant),
        })
        .collect();

    UnsplitFunclet {
        stmts,
        inputs: scheduling_funclet.input.clone(),
        output: scheduling_funclet.output.clone(),
        returned_var,
    }
}

#[derive(Clone, Debug)]
struct UnsplitBasicBlock
{
    block: Vec<sch::Hole<UnsplitStmt>>,
    prev: Vec<usize>,
    next: Vec<usize>,
}

fn partition_unsplit_basic_blocks(stmts: Vec<sch::Hole<UnsplitStmt>>) -> Vec<UnsplitBasicBlock>
{
    let mut blocks: Vec<UnsplitBasicBlock> =
        vec![UnsplitBasicBlock { block: vec![], prev: vec![], next: vec![] }];
    for stmt in stmts.into_iter() {
        let l = blocks.len();

        let next = match stmt {
            sch::Hole::Filled(UnsplitStmt::Let {
                e:
                    sch::Hole::Filled(Expr {
                        operation: Some(Operation { kind: sch::FullSchedulable::Call(_, _), .. }),
                        ..
                    }),
                ..
            }) => Some(vec![l]),
            _ => None,
        };

        blocks[l - 1].block.push(stmt);

        if let Some(n) = next {
            blocks[l - 1].next = n;
            blocks.push(UnsplitBasicBlock { block: vec![], prev: vec![], next: vec![] });
        }
    }
    // Attach backpointers
    for i in 0..blocks.len() {
        let nexts = blocks[i].next.clone();
        for next in nexts {
            blocks[next].prev.push(i);
        }
    }
    blocks
}

fn calculate_variables_used(
    blocks: &Vec<UnsplitBasicBlock>,
    returned_var: String,
) -> Vec<HashSet<String>>
{
    // TODO optimize this maybe as it's quite slow

    let mut vars_used: Vec<HashSet<String>> = vec![HashSet::new(); blocks.len()];
    vars_used[blocks.len() - 1].insert(returned_var);
    for (i, block) in blocks.iter().enumerate() {
        for stmt in block.block.clone() {
            if let sch::Hole::Filled(stmt) = stmt {
                for x in stmt.variables_used() {
                    vars_used[i].insert(x);
                }
            }
        }
    }
    for (i, block) in blocks.iter().enumerate().rev() {
        for n in block.next.iter() {
            for x in vars_used[i].clone().into_iter() {
                vars_used[*n].insert(x);
            }
        }
    }
    vars_used
}

fn split(unsplit_funclet: UnsplitFunclet) -> Vec<SplitFunclet>
{
    let UnsplitFunclet { stmts, inputs, output, returned_var } = unsplit_funclet;
    let mut unsplit_blocks = partition_unsplit_basic_blocks(stmts);
    let mut vars_used = calculate_variables_used(&unsplit_blocks, returned_var.clone());
    let num_blocks = unsplit_blocks.len();
    let mut split_funclets = Vec::new();
    for i in 0..num_blocks {
        let inputs = if i == 0 {
            inputs.clone()
        } else {
            // TODO use vars used!!! this is a hack
            // Doing hack for now because getting types of the vars is hard
            vec![(returned_var.clone(), output.clone())]
        };
        // TODO this is very incorrect!!!! hack for now
        let output = ("out".to_string(), output.clone());

        let block = &mut unsplit_blocks[i];
        let mut inner_block = std::mem::take(&mut block.block);

        let (join, tail_edge) = if block.next.len() > 0 {
            let last = inner_block
                .pop()
                .unwrap_or_else(|| panic!("Split on empty block somehow"))
                .to_option_move()
                .unwrap_or_else(|| panic!("Split on ???"));
            let last_expr_kind = last.expr_kind().unwrap_or_else(|| panic!("Split on ?"));
            let (join, tail_edge) = match last_expr_kind {
                sch::FullSchedulable::Call(f, _xs) => {
                    let next = match block.next[..] {
                        [n] => n,
                        _ => panic!("Schedule call block somehow has multiple nexts"),
                    };
                    let join = sch::Hole::Filled(SplitStmt::InlineDefaultJoin(next));

                    let f = f
                        .to_option()
                        .unwrap_or_else(|| panic!("? unavailable for use as callee id"));

                    let tail = TailEdge::ScheduleCall { callee_funclet_id: f.clone() };
                    (join, tail)
                },
                sch::FullSchedulable::Primitive | sch::FullSchedulable::CallExternal(_, _) => {
                    panic!("Split on statement which is not supposed to split (e.g. primitive)")
                },
            };

            (Some(join), tail_edge)
        } else {
            // Is return
            (None, TailEdge::Return(returned_var.clone()))
        };


        let mut stmts: Vec<sch::Hole<SplitStmt>> = inner_block
            .into_iter()
            .map(|hole| hole.map(|stmt| SplitStmt::Unsplit(stmt)))
            .collect();
        if let Some(j) = join {
            stmts.push(j);
        }

        split_funclets.push(SplitFunclet { stmts, inputs, output, tail_edge });
    }
    split_funclets
}

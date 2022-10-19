use crate::scheduling_language::ast as schedule_ast;
use crate::value_language::ast as value_ast;
use crate::value_language::typing;
use caiman::ir;
use std::collections::HashMap;

// TODO: turn panics into results
// should be pretty easy; can map a lot of options to results and then use
// '?' operator

type Index<T> = HashMap<T, usize>;

#[derive(Eq, PartialEq, Hash)]
enum Type
{
    Native(typing::Type),
    Slot(typing::Type, ir::ResourceQueueStage, ir::Place),
}

struct InnerFunclet
{
    pub input_types: Box<[ir::TypeId]>,
    pub output_types: Box<[ir::TypeId]>,
    pub nodes: Box<[ir::Node]>,
    pub tail_edge: ir::TailEdge,
}

struct ValueFunclet
{
    pub involved_variables: Index<String>,
    pub inner_funclet: InnerFunclet,
}

struct ScheduleExplicitFunclet
{
    pub inner_funclet: InnerFunclet,
}

pub fn go(
    value_ast: &value_ast::ParsedProgram,
    schedule_ast: &schedule_ast::ParsedProgram,
) -> ir::Program
{
    let ffi_types_index = ffi_types_index(value_ast);
    let context = generate_context(value_ast);
    let types_index = types_index(&ffi_types_index, &context, schedule_ast);
    let value_funclets = value_funclets(&types_index, value_ast);
    let schedule_explicit_funclets = schedule_explicit_funclets(
        &types_index,
        &context,
        &value_funclets,
        schedule_ast,
    );
    // TODO: timeline funclets,
    // last lil pipeline step
    /*let pipe1 = ir::Pipeline {
        name: String::from("A"),
        entry_funclet: 0,
        yield_points: Default::default(),
    };*/
    ir::Program {
        //pipelines: vec![pipe1],
        ..Default::default()
    }
}

fn value_funclets(
    types_index: &Index<Type>,
    value_ast: &value_ast::ParsedProgram,
) -> Vec<ValueFunclet>
{
    // Just gonna make one big value funclet... for now??
    let mut nodes_vec = vec![ir::Node::Phi { index: 0 }];
    let mut involved_variables: Index<String> = HashMap::new();
    for stmt in value_ast.iter()
    {
        let (info_s, kind_s) = stmt;
        use value_ast::ExprKind::*;
        use value_ast::StmtKind::*;
        match kind_s
        {
            Let((_mut, var, t), exp) =>
            {
                let (info_e, kind_e) = exp;
                match kind_e
                {
                    Num(s) =>
                    {
                        // TODO: not this
                        let value: i64 = s.parse().unwrap();
                        let type_id = match types_index.get(&Type::Native(*t))
                        {
                            None => panic!("Type not found {:?}", info_e),
                            Some(i) => *i,
                        };
                        nodes_vec.push(ir::Node::ConstantInteger {
                            value,
                            type_id,
                        });
                    },
                    _ => panic!("TODO"),
                };
                if involved_variables
                    .insert(var.clone(), nodes_vec.len())
                    .is_some()
                {
                    panic!("Involved variable issue {:?}", info_s)
                }
            },
            _ => panic!("TODO"),
        }
    }
    // TODO: not this
    let return_values = Box::new([nodes_vec.len()]);
    vec![ValueFunclet {
        involved_variables,
        inner_funclet: InnerFunclet {
            input_types: Box::new([0]),  // TODO
            output_types: Box::new([0]), // TODO
            nodes: nodes_vec.into_boxed_slice(),
            tail_edge: ir::TailEdge::Return { return_values },
        },
    }]
}

fn schedule_explicit_funclets(
    types_index: &Index<Type>,
    context: &HashMap<String, typing::Type>,
    value_funclets: &Vec<ValueFunclet>,
    schedule_ast: &schedule_ast::ParsedProgram,
) -> Vec<ScheduleExplicitFunclet>
{
    let mut nodes = vec![];
    for (info, kind) in schedule_ast.iter()
    {
        use schedule_ast::StmtKind::*;
        match kind
        {
            Var(x) =>
            {
                // XXX Perhaps iterating through VFs is too slow?
                let (funclet_id, x_vf) = value_funclets
                    .iter()
                    .enumerate()
                    .find(|(_, vf)| vf.involved_variables.contains_key(x))
                    .ok_or_else(|| {
                        panic!("Unbound variable {} at {:?}", x, info)
                    })
                    .unwrap();
                let node_id = x_vf.involved_variables[x];
                let x_type = context[x];
                let place = ir::Place::Local;
                let storage_type = ir::ffi::TypeId(
                    *types_index
                        .get(&Type::Slot(
                            x_type,
                            ir::ResourceQueueStage::Ready,
                            place,
                        ))
                        .ok_or_else(|| panic!(""))
                        .unwrap(),
                );
                let operation = ir::RemoteNodeId { funclet_id, node_id };
                nodes.push(ir::Node::AllocTemporary {
                    place,
                    storage_type,
                    operation,
                });
                let inputs_v : Vec<usize> = vec![];
                let outputs_v : Vec<usize> = vec![node_id];
                nodes.push(ir::Node::EncodeDo {
                    place,
                    operation,
                    inputs: inputs_v.into_boxed_slice(),
                    outputs: outputs_v.into_boxed_slice(),
                });
            },
        }
    }
    vec![ScheduleExplicitFunclet {
        inner_funclet: InnerFunclet {
            input_types: Box::new([0]),  // TODO
            output_types: Box::new([0]), // TODO
            nodes: nodes.into_boxed_slice(),
            tail_edge: ir::TailEdge::Return { return_values: Box::new([0]) },
        },
    }]
}

fn ffi_types_index(value_ast: &value_ast::ParsedProgram)
    -> Index<typing::Type>
{
    let mut map: HashMap<typing::Type, usize> = HashMap::new();
    let mut index = 0;
    for stmt in value_ast.iter()
    {
        for t in types_of_value_stmt(stmt).iter()
        {
            if !map.contains_key(t)
            {
                let check = map.insert(*t, index);
                if check.is_some()
                {
                    panic!("MAP CREATION ERROR {:?}", stmt.0);
                }
                index += 1;
            }
        }
    }
    map
}

fn types_of_value_stmt(stmt: &value_ast::ParsedStmt) -> Vec<typing::Type>
{
    let (_info, kind) = stmt;
    use value_ast::StmtKind::*;
    match kind
    {
        Let((_mut, _var, t), _exp) => vec![*t],
        _ => panic!("TODO"),
    }
}

fn generate_context(
    value_ast: &value_ast::ParsedProgram,
) -> HashMap<String, typing::Type>
{
    let mut ctx: HashMap<String, typing::Type> = HashMap::new();
    for (info, kind) in value_ast.iter()
    {
        use value_ast::StmtKind::*;
        match kind
        {
            Let((_mut, var, t), _exp) =>
            {
                if ctx.insert(var.to_string(), *t).is_some()
                {
                    panic!("Variable name collision {:?}", info);
                }
            },
            _ => (),
        }
    }
    ctx
}

fn types_index(
    ffi_types_index: &Index<typing::Type>,
    context: &HashMap<String, typing::Type>,
    schedule_ast: &schedule_ast::ParsedProgram,
) -> Index<Type>
{
    let mut index = 0;
    let mut types_index: Index<Type> = HashMap::new();
    for (t, _) in ffi_types_index.iter()
    {
        if types_index.insert(Type::Native(*t), index).is_some()
        {
            panic!("Native conversion error");
        }
        index += 1;
    }
    for (info, kind) in schedule_ast.iter()
    {
        use schedule_ast::StmtKind::*;
        match kind
        {
            Var(x) =>
            {
                let x_type = match context.get(x)
                {
                    None => panic!("Unbound var {:?} {:?}", x, info),
                    Some(t) => t,
                };
                let inserted_type = Type::Slot(
                    *x_type,
                    ir::ResourceQueueStage::Ready,
                    ir::Place::Local,
                );
                if !types_index.contains_key(&inserted_type)
                {
                    types_index.insert(inserted_type, index);
                    index += 1;
                }
            },
        }
    }
    types_index
}

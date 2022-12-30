use crate::error;
use crate::scheduling_language::ast as schedule_ast;
use crate::value_language::ast as value_ast;
use crate::value_language::typing;
use caiman::arena::Arena;
use caiman::ir;
use std::collections::HashMap;

pub enum ToIRError
{
    UnboundScheduleVar(String),
}

fn make_error(e: ToIRError, i: error::Info) -> error::DualLocalError
{
    let file_kind = match &e
    {
        ToIRError::UnboundScheduleVar(_) => error::FileKind::Scheduling,
    };
    error::DualLocalError {
        error: error::LocalError {
            kind: error::ErrorKind::ToIR(e),
            location: error::ErrorLocation::Double(i.location),
        },
        file_kind,
    }
}

type Index<T> = HashMap<T, usize>;
type ToIRResult<T> = Result<T, error::DualLocalError>;

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

trait Funclet
{
    fn inner(self) -> InnerFunclet;
    fn kind(&self) -> ir::FuncletKind;
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
) -> ToIRResult<ir::Program>
{
    let ffi_types_index = ffi_types_index(value_ast);
    let context = generate_context(value_ast)?;
    let types_index = types_index(&ffi_types_index, &context, schedule_ast)?;
    let value_funclets = value_funclets(&types_index, value_ast)?;
    let schedule_explicit_funclets =
        schedule_explicit_funclets(&types_index, &context, &value_funclets, schedule_ast)?;
    // TODO: timeline funclets,

    // Finally, constructing the program out of other stuff we just made
    let pipelines: Vec<ir::Pipeline> = schedule_explicit_funclets
        .iter()
        .enumerate()
        .map(|(i, _sef)| ir::Pipeline {
            name: String::from("Pipeline") + &i.to_string(),
            entry_funclet: value_funclets.len() + i,
            yield_points: Default::default(),
        })
        .collect();
    let types = map_index_to_arena(types_index, &|t| to_ir_type(&ffi_types_index, t));
    let native_interface = ir::ffi::NativeInterface {
        types: map_index_to_arena(ffi_types_index, &to_ffi_type),
        ..Default::default()
    };
    let funclets = vec_to_arena(
        value_funclets
            .into_iter()
            .map(make_ir_funclet)
            .chain(schedule_explicit_funclets.into_iter().map(make_ir_funclet))
            .collect(),
    );
    Ok(ir::Program { native_interface, types, funclets, pipelines, ..Default::default() })
}

fn get_native_type_index(
    types_index: &Index<Type>,
    native_type: &typing::Type,
    info: error::Info,
) -> usize
{
    match types_index.get(&Type::Native(*native_type))
    {
        None => panic!(
            "Type index improperly constructed: Type {:?} not found at {:?}",
            *native_type, info
        ),
        Some(i) => *i,
    }
}

struct ValueFuncletContext<'a>
{
    pub nodes_vec: Vec<ir::Node>,
    pub involved_variables: Index<String>,
    pub types_index: &'a Index<Type>,
}

fn add_value_expr(
    exp: &value_ast::ParsedExpr,
    t: &typing::Type,
    ctx: &mut ValueFuncletContext,
) -> ToIRResult<usize>
{
    use value_ast::ExprKind::*;
    let (info, kind) = exp;
    match kind
    {
        Var(x) =>
        {
            let x_index = ctx.involved_variables.get(x)
                .unwrap_or_else(|| 
                    panic!("Unbound var {}, type checker is either bugged or disabled.", x)
                );
            Ok(*x_index)
        },
        Num(s) =>
        {
            let value: i64 = s.parse().unwrap();
            let type_id = get_native_type_index(ctx.types_index, t, *info);
            ctx.nodes_vec.push(ir::Node::ConstantInteger { value, type_id });
            Ok(ctx.nodes_vec.len() - 1)
        },
        Bool(b) =>
        {
            let value: i64 = if *b { 1 } else { 0 };
            let type_id = get_native_type_index(ctx.types_index, t, *info);
            ctx.nodes_vec.push(ir::Node::ConstantInteger { value, type_id });
            Ok(ctx.nodes_vec.len() - 1)
        },
        If(e1, e2, e3) =>
        {
            let condition = add_value_expr(e1, &typing::Type::Bool, ctx)?;
            let true_case = add_value_expr(e2, t, ctx)?;
            let false_case = add_value_expr(e3, t, ctx)?;
            ctx.nodes_vec.push(ir::Node::Select { condition, true_case, false_case });
            Ok(ctx.nodes_vec.len() - 1)
        },
        _ => panic!("TODO"),
    }
}

fn value_funclets(
    types_index: &Index<Type>,
    value_ast: &value_ast::ParsedProgram,
) -> ToIRResult<Vec<ValueFunclet>>
{
    // Just gonna make one big value funclet... for now??
    let mut ctx = ValueFuncletContext {
        nodes_vec: vec![ir::Node::Phi { index: 0 }],
        involved_variables: HashMap::new(),
        types_index: &types_index,
    };
    for stmt in value_ast.iter()
    {
        let (info_s, kind_s) = stmt;
        use value_ast::StmtKind::*;
        match kind_s
        {
            Let((_mut, var, t), exp) =>
            {
                let expr_index = add_value_expr(exp, t, &mut ctx)?;
                if ctx.involved_variables.insert(var.clone(), expr_index).is_some()
                {
                    panic!("Duplicate {:?} in involved variables, {:?}", var.clone(), info_s)
                }
            },
            _ => panic!("TODO"),
        }
    }
    // TODO: not this
    let return_values = Box::new([ctx.nodes_vec.len() - 1]);
    Ok(vec![ValueFunclet {
        involved_variables: ctx.involved_variables,
        inner_funclet: InnerFunclet {
            input_types: Box::new([0]),  // TODO
            output_types: Box::new([0]), // TODO
            nodes: ctx.nodes_vec.into_boxed_slice(),
            tail_edge: ir::TailEdge::Return { return_values },
        },
    }])
}

fn schedule_explicit_funclets(
    types_index: &Index<Type>,
    context: &HashMap<String, typing::Type>,
    value_funclets: &Vec<ValueFunclet>,
    schedule_ast: &schedule_ast::ParsedProgram,
) -> ToIRResult<Vec<ScheduleExplicitFunclet>>
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
                    .ok_or(make_error(ToIRError::UnboundScheduleVar(x.to_string()), *info))?;
                let node_id = x_vf.involved_variables[x];
                let x_type = context[x];
                let place = ir::Place::Local;
                let storage_type = ir::ffi::TypeId(
                    *types_index
                        .get(&Type::Slot(x_type, ir::ResourceQueueStage::Ready, place))
                        .ok_or_else(|| panic!("Necessary storage type not created in index"))
                        .unwrap(),
                );
                let operation = ir::RemoteNodeId { funclet_id, node_id };
                nodes.push(ir::Node::AllocTemporary { place, storage_type, operation });
                let inputs_v: Vec<usize> = vec![];
                let outputs_v: Vec<usize> = vec![node_id];
                nodes.push(ir::Node::EncodeDo {
                    place,
                    operation,
                    inputs: inputs_v.into_boxed_slice(),
                    outputs: outputs_v.into_boxed_slice(),
                });
            },
        }
    }
    Ok(vec![ScheduleExplicitFunclet {
        inner_funclet: InnerFunclet {
            input_types: Box::new([0]),  // TODO
            output_types: Box::new([0]), // TODO
            nodes: nodes.into_boxed_slice(),
            tail_edge: ir::TailEdge::Return { return_values: Box::new([0]) },
        },
    }])
}

fn ffi_types_index(value_ast: &value_ast::ParsedProgram) -> Index<typing::Type>
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
) -> ToIRResult<HashMap<String, typing::Type>>
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
                    panic!("Variable name collision bypassed value language checker {:?}", info);
                }
            },
            _ => (),
        }
    }
    Ok(ctx)
}

fn types_index(
    ffi_types_index: &Index<typing::Type>,
    context: &HashMap<String, typing::Type>,
    schedule_ast: &schedule_ast::ParsedProgram,
) -> ToIRResult<Index<Type>>
{
    let mut index = 0;
    let mut types_index: Index<Type> = HashMap::new();
    for (t, _) in ffi_types_index.iter()
    {
        if types_index.insert(Type::Native(*t), index).is_some()
        {
            panic!("Unexpected index type collision from FFI types");
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
                let x_type = context
                    .get(x)
                    .ok_or(make_error(ToIRError::UnboundScheduleVar(x.to_string()), *info))?;
                let inserted_type =
                    Type::Slot(*x_type, ir::ResourceQueueStage::Ready, ir::Place::Local);
                if !types_index.contains_key(&inserted_type)
                {
                    types_index.insert(inserted_type, index);
                    index += 1;
                }
            },
        }
    }
    Ok(types_index)
}

impl Funclet for ValueFunclet
{
    fn inner(self) -> InnerFunclet { self.inner_funclet }
    fn kind(&self) -> ir::FuncletKind { ir::FuncletKind::Value }
}
impl Funclet for ScheduleExplicitFunclet
{
    fn inner(self) -> InnerFunclet { self.inner_funclet }
    fn kind(&self) -> ir::FuncletKind { ir::FuncletKind::ScheduleExplicit }
}

fn make_ir_funclet<T: Funclet>(f: T) -> ir::Funclet
{
    let kind = f.kind();
    let inner = f.inner();
    ir::Funclet {
        kind,
        input_types: inner.input_types,
        output_types: inner.output_types,
        nodes: inner.nodes,
        tail_edge: inner.tail_edge,
    }
}

fn to_ffi_type(t: typing::Type) -> ir::ffi::Type
{
    use typing::Type::*;
    match t
    {
        I32 => ir::ffi::Type::I32,
        Bool => ir::ffi::Type::I8,
    }
}

fn to_ir_type(ffi_types_index: &Index<typing::Type>, t: Type) -> ir::Type
{
    match t
    {
        Type::Native(t) =>
        {
            ir::Type::NativeValue { storage_type: ir::ffi::TypeId(ffi_types_index[&t]) }
        },
        Type::Slot(t, queue_stage, queue_place) => ir::Type::Slot {
            storage_type: ir::ffi::TypeId(ffi_types_index[&t]),
            queue_stage,
            queue_place,
        },
    }
}

fn vec_to_arena<T>(ts: Vec<T>) -> Arena<T>
{
    let mut map: HashMap<usize, T> = HashMap::new();
    for (i, elt) in ts.into_iter().enumerate()
    {
        map.insert(i, elt);
    }
    Arena::from_hash_map(map)
}

fn map_index_to_arena<T, U>(index: Index<T>, f: &dyn Fn(T) -> U) -> Arena<U>
{
    let mut map: HashMap<usize, U> = HashMap::new();
    for (t, i) in index.into_iter()
    {
        map.insert(i, f(t));
    }
    Arena::from_hash_map(map)
}

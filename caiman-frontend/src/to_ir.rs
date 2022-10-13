use crate::scheduling_language::ast as schedule_ast;
use crate::value_language::ast as value_ast;
use crate::value_language::typing;
use caiman::ir;
use std::collections::HashMap;

type Index<T> = HashMap<T, usize>;

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

pub fn go(
    value_ast: &value_ast::ParsedProgram,
    schedule_ast: &schedule_ast::ParsedProgram,
) -> ir::Program
{
    let ffi_types_index = ffi_types_index(value_ast);

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
    ffi_types_map: Index<typing::Type>,
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
                        let type_id = match ffi_types_map.get(t)
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

fn ffi_types_index(value_ast: &value_ast::ParsedProgram)
    -> Index<typing::Type>
{
    let mut map: HashMap<typing::Type, usize> = HashMap::new();
    let mut index = 0;
    for stmt in value_ast.iter()
    {
        for t in types_of_stmt(stmt).iter()
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

fn types_of_stmt(stmt: &value_ast::ParsedStmt) -> Vec<typing::Type>
{
    let (_info, kind) = stmt;
    use value_ast::StmtKind::*;
    match kind
    {
        Let((_mut, _var, t), _exp) => vec![*t],
        _ => panic!("TODO"),
    }
}

use std::cmp::Eq;
use std::hash::Hash; 
use crate::arena::Arena;
use crate::ir;
use caiman_frontend::ast;
use std::collections::HashMap;

pub enum SemanticError {}

pub fn semantic_error_to_string(e: SemanticError) -> String { String::new() }

// Plan:
// So Results implement the FromIterator trait which is great news
// We have each function like for example, ast_to_funclets, which takes a
// reference to the full AST and produces our funclets arena
// Start this using filter map! Of course everything non-funclet is useless
// to have
// Hopefully there isn't difficulty with reference business, but I did
// derive Clone for ast so should be ok?

pub fn from_ast(program: ast::Program) -> Result<ir::Program, SemanticError>
{
    let (types_arena, types_map) = generate_types_arena(&program)?;
    //println!("{:?}", &types_arena[&0]);
    //println!("{:?}", &types_map[&ir::Type::I32]);
    panic!("TODO")
}

// Creates both the arena (used as part of the IR program) as well as the
// inverse of it, which is necessary for translation
fn vec_to_arena_and_hashmap<T: Hash + Eq + Clone>(
    v: Vec<T>,
) -> (Arena<T>, HashMap<T, usize>)
{
    let mut hash_map: HashMap<usize, T> = HashMap::new();
    let mut inverse: HashMap<T, usize> = HashMap::new();
    for (i, t) in v.into_iter().enumerate()
    {
        if hash_map.insert(i, t.clone()).is_some()
        {
            panic!("Failure in vector to arena conversion");
        }
        if inverse.insert(t.clone(), i).is_some()
        {
            panic!("Failure in vector to hash map conversion");
        }
    }
    (Arena::from_hash_map(hash_map), inverse)
}

fn convert_type(t: ast::Type) -> Result<ir::Type, SemanticError>
{
    match t
    {
        ast::Type::F32 => Ok(ir::Type::F32),
        ast::Type::F64 => Ok(ir::Type::F64),
        ast::Type::U8 => Ok(ir::Type::U8),
        ast::Type::U16 => Ok(ir::Type::U16),
        ast::Type::U32 => Ok(ir::Type::U32),
        ast::Type::U64 => Ok(ir::Type::U64),
        ast::Type::I8 => Ok(ir::Type::I8),
        ast::Type::I16 => Ok(ir::Type::I16),
        ast::Type::I32 => Ok(ir::Type::I32),
        ast::Type::I64 => Ok(ir::Type::I64),
        _ => panic!("Unimplemented"),
    }
}

fn types_used(d: &ast::Declaration) -> Vec<ast::Type>
{
    let from_func_type = |ft: &(Vec<ast::Type>, Vec<ast::Type>)| {
        let mut v = ft.0.clone();
        v.append(&mut ft.1.clone());
        v
    };
    match d
    {
        ast::Declaration::Funclet(_, _, ft, nodes, _) =>
        {
            let mut v = from_func_type(ft);
            for (_, nodetype) in nodes.iter()
            {
                if let ast::NodeType::Constant(_, t) = nodetype
                {
                    v.push(t.clone());
                }
            }
            v
        }
        ast::Declaration::CPU(_, ft) => from_func_type(ft),
        ast::Declaration::GPU(_, _, ft, _, _) => from_func_type(ft),
        ast::Declaration::ValueFunction(_, _, ft) => from_func_type(ft),
        ast::Declaration::Pipeline(_, _) => vec![],
    }
}

fn generate_types_arena(
    program: &ast::Program,
) -> Result<(Arena<ir::Type>, HashMap<ir::Type, usize>), SemanticError>
{
    let mut all_types_used: Vec<ast::Type> =
        program.iter().fold(vec![], |mut v, mut d| {
            v.append(&mut types_used(d));
            v
        });
    all_types_used.sort();
    all_types_used.dedup();
    let converted_types: Result<Vec<ir::Type>, SemanticError> =
        all_types_used.into_iter().map(convert_type).collect();
    converted_types.map(vec_to_arena_and_hashmap)
}

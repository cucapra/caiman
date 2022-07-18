use crate::arena::Arena;
use crate::ir;
use caiman_frontend::ir_version::ast;
use std::cmp::Eq;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::hash::Hash;

pub enum SemanticError
{
    Index(String),
    NonIntegerType(ast::Type),
    ValueParsing(String),
    IntegerTooLarge(String, ast::Type),
}

pub fn semantic_error_to_string(e: SemanticError) -> String
{
    match e
    {
        SemanticError::Index(f_id) => format!("Unbound name {}", f_id),
        SemanticError::NonIntegerType(t) =>
        {
            format!("Expected integer type, but is {:?} instead", t)
        }
        SemanticError::ValueParsing(s) => format!("Error parsing value {}", s),
        SemanticError::IntegerTooLarge(s, t) =>
        {
            format!("Value {} does not fit into type {:?}", s, t)
        }
    }
}

type Index<T> = HashMap<T, usize>;

pub fn from_ast(program: ast::Program) -> Result<ir::Program, SemanticError>
{
    let (types, types_index) = generate_types(&program);

    let (external_cpu_functions, cpu_index) =
        generate_cpu(&program, &types_index)?;
    let (external_gpu_functions, gpu_index) =
        generate_gpu(&program, &types_index)?;

    let funclet_index = generate_funclet_index(&program);
    let (value_functions, vf_index) =
        generate_value_functions(&program, &types_index, &funclet_index)?;

    let funclets = convert_funclets(
        &program,
        &types_index,
        &funclet_index,
        &cpu_index,
        &gpu_index,
        &vf_index,
    )?;

    let pipelines = convert_pipelines(&program, &funclet_index)?;

    Ok(ir::Program {
        types,
        funclets,
        external_cpu_functions,
        external_gpu_functions,
        value_functions,
        pipelines,
    })
}

fn vec_to_arena<T: Clone>(v: Vec<T>) -> Arena<T>
{
    let mut hash_map: HashMap<usize, T> = HashMap::new();
    for (i, t) in v.into_iter().enumerate()
    {
        if hash_map.insert(i, t.clone()).is_some()
        {
            panic!("Failure in vector to arena conversion");
        }
    }
    Arena::from_hash_map(hash_map)
}

fn vec_to_index<T: Clone + Hash + Eq>(v: Vec<T>) -> Index<T>
{
    let mut index: Index<T> = HashMap::new();
    for (i, s) in v.into_iter().enumerate()
    {
        if index.insert(s, i).is_some()
        {
            panic!("Failure in vector to index conversion");
        }
    }
    index
}

fn index_get(index: &Index<String>, key: &str)
    -> Result<usize, SemanticError>
{
    match index.get(key)
    {
        Some(v) => Ok(*v),
        None => Err(SemanticError::Index(String::from(key))),
    }
}

fn convert_type(
    t: ast::Type,
    f: &mut dyn FnMut(ast::Type) -> usize,
) -> ir::Type
{
    match t
    {
        ast::Type::F32 => ir::Type::F32,
        ast::Type::F64 => ir::Type::F64,
        ast::Type::U8 => ir::Type::U8,
        ast::Type::U16 => ir::Type::U16,
        ast::Type::U32 => ir::Type::U32,
        ast::Type::U64 => ir::Type::U64,
        ast::Type::I8 => ir::Type::I8,
        ast::Type::I16 => ir::Type::I16,
        ast::Type::I32 => ir::Type::I32,
        ast::Type::I64 => ir::Type::I64,
        ast::Type::Array(t_box, length) => ir::Type::Array {
            element_type: f(*t_box),
            length,
        },
        ast::Type::Ref(t_box) => ir::Type::ConstRef {
            element_type: f(*t_box),
        },
        ast::Type::MutRef(t_box) => ir::Type::MutRef {
            element_type: f(*t_box),
        },
        ast::Type::Slice(t_box) => ir::Type::ConstSlice {
            element_type: f(*t_box),
        },
        ast::Type::MutSlice(t_box) => ir::Type::MutSlice {
            element_type: f(*t_box),
        },
        _ => panic!("Unimplemented"),
    }
}

// The following panics instead of using SemanticError because its not
// working as intended means the index was built wrong, i.e. my fault and
// unintended behavior
fn ast_type_to_id(t: ast::Type, types_index: &Index<ir::Type>) -> usize
{
    let mut recur = |t| ast_type_to_id(t, types_index);
    let ir_type = convert_type(t, &mut recur);
    types_index[&ir_type]
}

fn convert_function_type(
    ft: ast::FuncType,
    types_index: &Index<ir::Type>,
) -> Result<(Box<[usize]>, Box<[usize]>), SemanticError>
{
    let (input, output) = ft;
    let convert = |v: Vec<ast::Type>| -> Box<[usize]> {
        v.into_iter().map(|t| ast_type_to_id(t, types_index)).collect()
    };
    Ok((convert(input), convert(output)))
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

fn add_to_type_index(
    t: ast::Type,
    counter: &mut usize,
    index: &mut Index<ir::Type>,
    arena_map: &mut HashMap<usize, ir::Type>,
) -> usize
{
    let mut recur = |t| add_to_type_index(t, counter, index, arena_map);
    let ir_typ = convert_type(t, &mut recur);
    match index.get(&ir_typ)
    {
        Some(x) => *x,
        None =>
        {
            if index.insert(ir_typ.clone(), *counter).is_some()
                || arena_map.insert(*counter, ir_typ.clone()).is_some()
            {
                panic!("Failure in type arena creation");
            }
            let old_counter = *counter;
            *counter += 1;
            old_counter
        }
    }
}

fn type_vec_to_arena_and_index(
    v: Vec<ast::Type>,
) -> (Arena<ir::Type>, Index<ir::Type>)
{
    let mut counter = 0;
    let mut type_index: Index<ir::Type> = HashMap::new();
    let mut arena_map: HashMap<usize, ir::Type> = HashMap::new();
    for typ in v.iter()
    {
        add_to_type_index(
            typ.clone(),
            &mut counter,
            &mut type_index,
            &mut arena_map,
        );
    }
    let arena = Arena::from_hash_map(arena_map);
    (arena, type_index)
}

fn generate_types(program: &ast::Program)
    -> (Arena<ir::Type>, Index<ir::Type>)
{
    let mut all_types_used: Vec<ast::Type> =
        program.iter().fold(vec![], |mut v, d| {
            v.append(&mut types_used(d));
            v
        });
    all_types_used.sort();
    all_types_used.dedup();
    type_vec_to_arena_and_index(all_types_used)
}

fn convert_cpu(
    d: &ast::Declaration,
    types_index: &Index<ir::Type>,
) -> Option<Result<ir::ExternalCpuFunction, SemanticError>>
{
    match d
    {
        ast::Declaration::CPU(name, ft) =>
        {
            let func_type = convert_function_type(ft.clone(), types_index);
            Some(func_type.map(|(input_types, output_types)| {
                ir::ExternalCpuFunction {
                    name: String::from(name),
                    input_types,
                    output_types,
                }
            }))
        }
        _ => None,
    }
}

fn construct_ir_gpu(
    name: &str,
    entry_point: &str,
    ft: &ast::FuncType,
    rbs: &Vec<ast::ResourceBinding>,
    shader: &Option<ast::Shader>,
    types_index: &Index<ir::Type>,
) -> Result<ir::ExternalGpuFunction, SemanticError>
{
    let (i, o) = convert_function_type(ft.clone(), types_index)?;
    let resource_bindings: Box<[ir::ExternalGpuFunctionResourceBinding]> = rbs
        .iter()
        .map(|rb| ir::ExternalGpuFunctionResourceBinding {
            group: rb.group,
            binding: rb.binding,
            input: rb.input,
            output: rb.output,
        })
        .collect();
    let shader_module_content = match shader {
        None => ir::ShaderModuleContent::Wgsl(String::new()),
        Some((shader_type, content)) => match shader_type {
            ast::ShaderType::Wgsl => 
                ir::ShaderModuleContent::Wgsl(String::from(content))
        },
    };
    Ok(ir::ExternalGpuFunction {
        name: String::from(name),
        entry_point: String::from(entry_point),
        input_types: i,
        output_types: o,
        resource_bindings,
        shader_module_content, 
    })
}

fn convert_gpu(
    d: &ast::Declaration,
    types_index: &Index<ir::Type>,
) -> Option<Result<ir::ExternalGpuFunction, SemanticError>>
{
    match d
    {
        ast::Declaration::GPU(name, entry, ft, rbs, shader) =>
        {
            Some(construct_ir_gpu(name, entry, ft, rbs, shader, types_index))
        }
        _ => None,
    }
}

// Making the generic version of the following two functions to factor out
// code was very hard! I gave up on doing it after the iterator refused to
// collect to a generic Vec<T>. If you are reading this and are not me and
// would like to improve this code by helping me make the generic version
// of these two functions (and you know how), please let me know! Thanks :)

fn generate_cpu(
    program: &ast::Program,
    types_index: &Index<ir::Type>,
) -> Result<(Vec<ir::ExternalCpuFunction>, Index<String>), SemanticError>
{
    let converted: Result<Vec<ir::ExternalCpuFunction>, SemanticError> =
        program.iter().filter_map(|d| convert_cpu(d, types_index)).collect();

    converted.map(|functions| {
        let names = functions.iter().map(|c| String::from(&c.name)).collect();
        let index = vec_to_index(names);
        (functions, index)
    })
}

fn generate_gpu(
    program: &ast::Program,
    types_index: &Index<ir::Type>,
) -> Result<(Vec<ir::ExternalGpuFunction>, Index<String>), SemanticError>
{
    let converted: Result<Vec<ir::ExternalGpuFunction>, SemanticError> =
        program.iter().filter_map(|d| convert_gpu(d, types_index)).collect();

    converted.map(|functions| {
        let names = functions.iter().map(|g| String::from(&g.name)).collect();
        let index = vec_to_index(names);
        (functions, index)
    })
}

fn generate_funclet_index(program: &ast::Program) -> Index<String>
{
    let names = program
        .iter()
        .filter_map(|d| match d
        {
            ast::Declaration::Funclet(_, s, _, _, _) => Some(String::from(s)),
            _ => None,
        })
        .collect();
    vec_to_index(names)
}

fn generate_value_functions(
    program: &ast::Program,
    types_index: &Index<ir::Type>,
    funclet_index: &Index<String>,
) -> Result<(Arena<ir::ValueFunction>, Index<String>), SemanticError>
{
    let vfs: Result<Vec<ir::ValueFunction>, SemanticError> = program
        .iter()
        .filter_map(|d| match d
        {
            ast::Declaration::ValueFunction(name, funclet_op, ft) => Some(
                // Explanation of this code, if this is too unreadable:
                //  index_get and convert_function_type both return results,
                //  but funclet_op is an option, so I apply index_get to it
                //  using map, then transpose, switching option and result
                //  (of course, we want option on the inside). Result is
                //  self explanatory I think (and_then for first result,
                //  followed by map for second)
                funclet_op
                    .clone()
                    .map(|f| index_get(funclet_index, &f))
                    .transpose()
                    .and_then(|f_id| {
                        convert_function_type(ft.clone(), types_index).map(
                            |(i, o)| ir::ValueFunction {
                                name: String::from(name),
                                input_types: i,
                                output_types: o,
                                default_funclet_id: f_id,
                            },
                        )
                    }),
            ),
            _ => None,
        })
        .collect();
    vfs.map(|vfs| {
        let names = vfs.iter().map(|vf| String::from(&vf.name)).collect();
        (vec_to_arena(vfs), vec_to_index(names))
    })
}

fn generate_node_index(nodes: &Vec<ast::Node>) -> Index<String>
{
    let v = nodes.iter().map(|(s, _)| String::from(s)).collect();
    vec_to_index(v)
}

fn is_unsigned_integer(t: ast::Type) -> Result<bool, SemanticError>
{
    match t
    {
        ast::Type::U8 | ast::Type::U16 | ast::Type::U32 | ast::Type::U64 =>
        {
            Ok(true)
        }
        ast::Type::I8 | ast::Type::I16 | ast::Type::I32 | ast::Type::I64 =>
        {
            Ok(false)
        }
        _ => Err(SemanticError::NonIntegerType(t.clone())),
    }
}

fn convert_size_error<T, V>(
    result: Result<T, V>,
    value_str: &str,
    t: ast::Type,
) -> Result<(), SemanticError>
{
    match result
    {
        Err(_) =>
        {
            Err(SemanticError::IntegerTooLarge(String::from(value_str), t))
        }
        Ok(_) => Ok(()),
    }
}

fn check_size<T>(
    v: T,
    value_str: &str,
    t: ast::Type,
) -> Result<(), SemanticError>
where
    u8: TryFrom<T>,
    u16: TryFrom<T>,
    u32: TryFrom<T>,
    i8: TryFrom<T>,
    i16: TryFrom<T>,
    i32: TryFrom<T>,
{
    match t
    {
        ast::Type::U8 => convert_size_error(u8::try_from(v), value_str, t),
        ast::Type::U16 => convert_size_error(u16::try_from(v), value_str, t),
        ast::Type::U32 => convert_size_error(u32::try_from(v), value_str, t),
        ast::Type::I8 => convert_size_error(i8::try_from(v), value_str, t),
        ast::Type::I16 => convert_size_error(i16::try_from(v), value_str, t),
        ast::Type::I32 => convert_size_error(i32::try_from(v), value_str, t),
        ast::Type::I64 | ast::Type::U64 => Ok(()),
        _ => panic!("Not an integer"),
    }
}

fn map_index(
    args: &Vec<String>,
    node_index: &Index<String>,
) -> Result<Box<[usize]>, SemanticError>
{
    args.iter().map(|s| index_get(node_index, s)).collect()
}

fn convert_node(
    node: &ast::NodeType,
    types_index: &Index<ir::Type>,
    funclet_index: &Index<String>,
    cpu_index: &Index<String>,
    gpu_index: &Index<String>,
    vf_index: &Index<String>,
    node_index: &Index<String>,
) -> Result<ir::Node, SemanticError>
{
    match node
    {
        ast::NodeType::Phi(index) => Ok(ir::Node::Phi {
            index: *index,
        }),
        ast::NodeType::Extract(sub_node, index) =>
        {
            let node_id = index_get(node_index, sub_node)?;
            Ok(ir::Node::ExtractResult {
                node_id,
                index: *index,
            })
        }
        ast::NodeType::Constant(value_str, typ) =>
        {
            let type_id = ast_type_to_id(typ.clone(), types_index);
            // This error is used in both branches
            let value_parsing_error =
                Err(SemanticError::ValueParsing(String::from(value_str)));
            if is_unsigned_integer(typ.clone())?
            {
                let value_parse: Result<u64, _> = value_str.parse();
                match value_parse
                {
                    Err(_) => value_parsing_error,
                    Ok(value) =>
                    {
                        check_size(value, &value_str, typ.clone())?;
                        Ok(ir::Node::ConstantUnsignedInteger {
                            value,
                            type_id,
                        })
                    }
                }
            }
            else
            {
                let value_parse: Result<i64, _> = value_str.parse();
                match value_parse
                {
                    Err(_) => value_parsing_error,
                    Ok(value) =>
                    {
                        check_size(value, &value_str, typ.clone())?;
                        Ok(ir::Node::ConstantInteger {
                            value,
                            type_id,
                        })
                    }
                }
            }
        }
        ast::NodeType::Call(name, args) =>
        {
            let arguments = map_index(args, node_index)?;
            // Could be CPU or ValueFunction call
            if cpu_index.contains_key(&name.clone())
            {
                // Assume CPU
                let external_function_id = index_get(cpu_index, &name)?;
                Ok(ir::Node::CallExternalCpu {
                    external_function_id,
                    arguments,
                })
            }
            else
            // Assume ValueFunction
            {
                let function_id = index_get(vf_index, &name)?;
                Ok(ir::Node::CallValueFunction {
                    function_id,
                    arguments,
                })
            }
        }
        ast::NodeType::GPUCall(name, dims, args) =>
        {
            let external_function_id = index_get(gpu_index, &name)?;
            let arguments = map_index(args, node_index)?;
            let dimensions = map_index(dims, node_index)?;
            Ok(ir::Node::CallExternalGpuCompute {
                external_function_id,
                arguments,
                dimensions,
            })
        }
        ast::NodeType::GPUSubmit(vals) =>
        {
            let values = map_index(vals, node_index)?;
            Ok(ir::Node::SubmitGpu {
                values,
            })
        }
        ast::NodeType::GPUEncode(vals) =>
        {
            let values = map_index(vals, node_index)?;
            Ok(ir::Node::EncodeGpu {
                values,
            })
        }
        ast::NodeType::LocalSync(vals) =>
        {
            let values = map_index(vals, node_index)?;
            Ok(ir::Node::SyncLocal {
                values,
            })
        }
    }
}

fn convert_tail_edge(
    tail: &ast::FuncletTail,
    node_index: &Index<String>,
    funclet_index: &Index<String>,
) -> Result<ir::TailEdge, SemanticError>
{
    match tail
    {
        ast::FuncletTail::Return(rvs) =>
        {
            let return_values = map_index(&rvs, node_index)?;
            Ok(ir::TailEdge::Return {
                return_values,
            })
        }
        ast::FuncletTail::Yield(rvs, args, funclets) =>
        {
            let captured_arguments = map_index(&args, node_index)?;
            let return_values = map_index(&rvs, node_index)?;
            let funclet_ids = map_index(&funclets, funclet_index)?;
            Ok(ir::TailEdge::Yield {
                funclet_ids,
                captured_arguments,
                return_values,
            })
        }
    }
}

fn convert_funclet(
    is_inline: bool,
    function_type: &ast::FuncType,
    ast_nodes: &Vec<ast::Node>,
    ast_tail: &ast::FuncletTail,
    types_index: &Index<ir::Type>,
    funclet_index: &Index<String>,
    cpu_index: &Index<String>,
    gpu_index: &Index<String>,
    vf_index: &Index<String>,
) -> Result<ir::Funclet, SemanticError>
{
    let node_index = generate_node_index(ast_nodes);

    let kind = if is_inline
    {
        ir::FuncletKind::Inline
    }
    else
    {
        ir::FuncletKind::MixedImplicit
    };
    let (input_types, output_types) =
        convert_function_type(function_type.clone(), types_index)?;
    let nodes_res: Result<Box<[ir::Node]>, SemanticError> = ast_nodes
        .iter()
        .map(|(_, node)| {
            convert_node(
                node,
                types_index,
                funclet_index,
                cpu_index,
                gpu_index,
                vf_index,
                &node_index,
            )
        })
        .collect();
    let nodes = nodes_res?;
    let tail_edge = convert_tail_edge(ast_tail, &node_index, funclet_index)?;

    Ok(ir::Funclet {
        kind,
        input_types,
        output_types,
        nodes,
        tail_edge,
        // Dunno about these
        input_resource_states: Default::default(),
        output_resource_states: Default::default(),
        local_meta_variables: Default::default(),
    })
}

fn convert_funclets(
    program: &ast::Program,
    types_index: &Index<ir::Type>,
    funclet_index: &Index<String>,
    cpu_index: &Index<String>,
    gpu_index: &Index<String>,
    vf_index: &Index<String>,
) -> Result<Arena<ir::Funclet>, SemanticError>
{
    let v: Result<Vec<ir::Funclet>, SemanticError> = program
        .iter()
        .filter_map(|d| match d
        {
            ast::Declaration::Funclet(is_inline, _, ft, nodes, tail) =>
            {
                Some(convert_funclet(
                    *is_inline,
                    ft,
                    nodes,
                    tail,
                    types_index,
                    funclet_index,
                    cpu_index,
                    gpu_index,
                    vf_index,
                ))
            }
            _ => None,
        })
        .collect();
    v.map(vec_to_arena)
}

fn convert_pipelines(
    program: &ast::Program,
    funclet_index: &Index<String>,
) -> Result<Vec<ir::Pipeline>, SemanticError>
{
    program
        .iter()
        .filter_map(|d| match d
        {
            ast::Declaration::Pipeline(name, entry_funclet_name) =>
            {
                Some(index_get(funclet_index, entry_funclet_name).map(
                    |entry_funclet| ir::Pipeline {
                        name: String::from(name),
                        entry_funclet,
                    },
                ))
            }
            _ => None,
        })
        .collect()
}

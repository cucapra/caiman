use crate::arena::Arena;
use crate::frontend;
use crate::ir;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, Write};

fn write_indent(oc: &mut dyn Write, indent: usize) -> std::io::Result<()>
{
    for i in 0..indent
    {
        write!(oc, "  ")?;
    }
    Ok(())
}

fn slice_to_string_specific<T>(
    slice: &[T],
    to_string: &dyn Fn(&T) -> String,
    delimiter: String,
    start: String,
    end: String,
    exclude_last: bool,
) -> String
{
    let mut s = slice
        .iter()
        .map(to_string)
        .fold(String::new(), |acc, s| acc + &s + &delimiter);
    // Removing last delimiter
    if exclude_last 
    {
        for i in 0..delimiter.len()
        {
            s.pop();
        }
    }
    start + &s + end.as_str()
}

fn slice_to_string<T>(slice: &[T], to_string: &dyn Fn(&T) -> String)
    -> String
{
    slice_to_string_specific(
        slice, 
        to_string, 
        String::from(", "), 
        String::from("["),
        String::from("]"),
        true,
    )
}

fn funclet_name(num: usize) -> String {
    format!("f{}", num)
}

fn node_name(num: usize) -> String {
    format!("n{}", num)
}

fn write_function_type_generic<T>(
    oc: &mut dyn Write,
    input_types: &[T],
    output_types: &[T],
    to_string: &dyn Fn(&T) -> String,
) -> std::io::Result<()>
{
    let input_s = slice_to_string_specific(
        input_types, 
        to_string,
        String::from(", "), 
        String::from("("),
        String::from(")"),
        true,
    );
    let output_s = slice_to_string_specific(
        output_types, 
        to_string,
        String::from(", "), 
        String::from("("),
        String::from(")"),
        true,
    );
    write!(oc, "{} -> {}", input_s, output_s)?;
    Ok(())
}

fn write_function_type(
    oc: &mut dyn Write,
    input_types: &Box<[usize]>,
    output_types: &Box<[usize]>,
    types_arena: &Arena<ir::Type>,
) -> std::io::Result<()>
{
    let true_input_types: Vec<&ir::Type> =
        input_types.iter().map(|u| &types_arena[&u]).collect();
    let true_output_types: Vec<&ir::Type> =
        output_types.iter().map(|u| &types_arena[&u]).collect();
    let to_string = |t: &&ir::Type| format!("{:?}", t);
    write_function_type_generic(
        oc,
        true_input_types.as_slice(),
        true_output_types.as_slice(),
        &to_string,
    )
}

fn write_function_type_numbers(
    oc: &mut dyn Write,
    input_types: &Box<[usize]>,
    output_types: &Box<[usize]>,
) -> std::io::Result<()>
{
    let to_string = |u: &usize| u.to_string();
    write_function_type_generic(
        oc,
        &(*input_types),
        &(*output_types),
        &to_string,
    )
}

fn usize_node_slice_borders_string(arguments: &[usize], l: String, r: String) 
    -> String 
{
    slice_to_string_specific(
        arguments, 
        &|u : &usize| node_name(*u), 
        String::from(", "), 
        l,
        r,
        true,
    )
}

fn args_to_string(arguments: &[usize]) -> String
{
    usize_node_slice_borders_string(
        arguments, 
        String::from("("), 
        String::from(")")
    )
}

fn string_of_node(
    node: &ir::Node, 
    types_arena: &Arena<ir::Type>,
    value_functions_arena: &Arena<ir::ValueFunction>,
    cpu_functions: &Vec<ir::ExternalCpuFunction>,
    gpu_functions: &Vec<ir::ExternalGpuFunction>,
) -> String
{
    match node 
    {
        ir::Node::Phi {index} => format!("phi {}", index),
        ir::Node::ExtractResult { node_id, index } => { 
            let s = format!("extract {}", node_name(*node_id));
            if *index == 0 { s } else { format!("{}[{}]", s, index) }
        },
        ir::Node::ConstantInteger{value, type_id} => {
            format!("const {} : {:?}", value, &types_arena[&type_id])
        },
        ir::Node::ConstantUnsignedInteger{value, type_id} => {
            format!("uconst {} : {:?}", value, &types_arena[&type_id])
        },
        ir::Node::CallValueFunction { function_id, arguments } => {
            format!(
                "{}{}", 
                (&value_functions_arena[&function_id]).name, 
                args_to_string(arguments),
            )
        },
        ir::Node::CallExternalCpu { external_function_id, arguments } => {
            format!(
                "{}{}", 
                (cpu_functions[*external_function_id]).name,
                args_to_string(arguments),
            )
        },
        ir::Node::CallExternalGpuCompute {
            external_function_id, 
            arguments, 
            dimensions
        } => {
            format!(
                "{}{}{}", 
                (gpu_functions[*external_function_id]).name,
                usize_node_slice_borders_string(
                    dimensions, 
                    String::from("<"),
                    String::from(">"),
                ),
                args_to_string(arguments),
            )
        },
        _ => String::from("TODO")
    }
}

// This is to make the printing deterministic!
fn funclets_arena_to_vector(a: &Arena<ir::Funclet>) -> Vec<&ir::Funclet>
{
    let len = a.iter().count();
    let mut v = Vec::new();
    for i in 0..len
    {
        v.push(&a[&i]);
    }
    v
}

fn write_funclets(
    oc: &mut dyn Write,
    funclets: Arena<ir::Funclet>,
    types_arena: &Arena<ir::Type>,
    numbers_mode: bool,
    value_functions_arena : &Arena<ir::ValueFunction>,
    cpu_functions: &Vec<ir::ExternalCpuFunction>,
    gpu_functions: &Vec<ir::ExternalGpuFunction>,
) -> std::io::Result<()>
{
    let funclets_vec = funclets_arena_to_vector(&funclets);
    for (num, funclet) in funclets_vec.iter().enumerate()
    {
        if let ir::FuncletKind::Inline = funclet.kind
        {
            write!(oc, "Inline ")?;
        }
        write!(oc, "Funclet {} ", funclet_name(num))?;
        // This is subject to change!
        //write!(oc, "({:?}) ", funclet.kind)?;
        write!(oc, ": ")?;
        if numbers_mode
        {
            write_function_type_numbers(
                oc,
                &funclet.input_types,
                &funclet.output_types,
            )?;
        }
        else
        {
            write_function_type(
                oc,
                &funclet.input_types,
                &funclet.output_types,
                types_arena,
            )?;
        }
        write!(oc, " {{\n")?;
        // There seem to be more things in the actual data
        // structure in ir.rs than there are in the RON file.
        // For now, I'll only print the nodes and tail edge.
        for (i, node) in funclet.nodes.iter().enumerate()
        {
            write_indent(oc, 1)?;
            // Node-printing could be subject to change
            write!(
                oc, "{} = {};", 
                node_name(i), 
                string_of_node(
                    node, 
                    types_arena, 
                    value_functions_arena,
                    cpu_functions,
                    gpu_functions,
                ),
            )?;
            write!(oc, "\n")?;
        }

        write_indent(oc, 1)?;

        let rv_to_string = |rv| slice_to_string_specific(
            rv, 
            &|u : &usize| node_name(*u),
            String::from(", "), 
            String::from(""), 
            String::from(""),
            true,
        );
        let f_to_string = |f| slice_to_string_specific(
            f, 
            &|u| funclet_name(*u),
            String::from(", "), 
            String::from(""), 
            String::from(""),
            true,
        );
        let arg_to_string = |a| slice_to_string_specific(
            a, 
            &|u| node_name(*u),
            String::from(", "), 
            String::from(""), 
            String::from(""),
            true,
        );
        match &funclet.tail_edge
        {
            ir::TailEdge::Return {
                return_values,
            } =>
            {
                let return_vals_s = rv_to_string(&return_values);
                write!(oc, "Return {}\n", &return_vals_s)?;
            }
            ir::TailEdge::Yield {
                funclet_ids,
                captured_arguments,
                return_values,
            } =>
            {
                let funclet_ids_s = f_to_string(&funclet_ids);
                let cap_args_s = arg_to_string(&captured_arguments);
                let return_vals_s = rv_to_string(&return_values);
                write!(oc, "Yield {} ", return_vals_s)?;
                write!(oc, "{{{} => {}}}\n", cap_args_s, funclet_ids_s)?;
            }
        }

        write!(oc, "}}\n\n")?;
    }
    Ok(())
}

fn write_external_cpu_functions(
    oc: &mut dyn Write,
    funcs: Vec<ir::ExternalCpuFunction>,
    types_arena: &Arena<ir::Type>,
    numbers_mode: bool,
) -> std::io::Result<()>
{
    for func in funcs.iter()
    {
        write!(oc, "CPU {} : ", func.name)?;
        if numbers_mode
        {
            write_function_type_numbers(
                oc,
                &func.input_types,
                &func.output_types,
            )?;
        }
        else
        {
            write_function_type(
                oc,
                &func.input_types,
                &func.output_types,
                types_arena,
            )?;
        }
        write!(oc, ";\n\n")?;
    }
    Ok(())
}

fn write_external_gpu_functions(
    oc: &mut dyn Write,
    funcs: Vec<ir::ExternalGpuFunction>,
    types_arena: &Arena<ir::Type>,
    numbers_mode: bool,
) -> std::io::Result<()>
{
    for func in funcs.iter()
    {
        write!(oc, "GPU {} ({}) : ", func.name, func.entry_point)?;

        if numbers_mode
        {
            write_function_type_numbers(
                oc,
                &func.input_types,
                &func.output_types,
            )?;
        }
        else
        {
            write_function_type(
                oc,
                &func.input_types,
                &func.output_types,
                types_arena,
            )?;
        }
        write!(oc, " {{\n")?;

        let map_rb_type = |t : Option<usize>| 
            t.map_or(String::from("_"), |u : usize| u.to_string());
        write_indent(oc, 1)?;
        let resource_bindings_str = slice_to_string_specific(
            &(*func.resource_bindings),
            &|rb| {
                format!(
                    "[{}][{}] = {} -> {}",
                    rb.group, rb.binding, 
                    map_rb_type(rb.input), map_rb_type(rb.output),
                )
            },
            String::from(";\n  "), 
            String::from(""), 
            String::from(""),
            false,
        );
        write!(oc, "{}", resource_bindings_str)?;

        // Shader module content seems inherently not pretty; it just looks
        // like a block of WGSL code, so I am not going to print it (for now)
        
        //write_indent(oc, 2)?;
        //write!(oc, "Shader module content: {:?}\n",
        //func.shader_module_content)?;
        write!(oc, "\n}}\n\n");
    }
    Ok(())
}

fn write_value_functions(
    oc: &mut dyn Write,
    value_functions: Arena<ir::ValueFunction>,
    types_arena: &Arena<ir::Type>,
    numbers_mode: bool,
) -> std::io::Result<()>
{
    for (_, vf) in value_functions.iter()
    {
        let default_funclet = vf.default_funclet_id.map_or(
            String::from(""), 
            funclet_name,
        );
        write!(oc, "ValueFunction {} ({}) : ", vf.name, default_funclet)?;
        if numbers_mode
        {
            write_function_type_numbers(
                oc,
                &vf.input_types,
                &vf.output_types,
            )?;
        }
        else
        {
            write_function_type(
                oc,
                &vf.input_types,
                &vf.output_types,
                types_arena,
            )?;
        }
        write!(oc, ";\n\n")?;
    }
    Ok(())
}

fn write_pipelines(
    oc: &mut dyn Write,
    pipelines: Vec<ir::Pipeline>,
) -> std::io::Result<()>
{
    for p in pipelines.iter()
    {
        let entry_funclet = funclet_name(p.entry_funclet);
        write!(oc, "Pipeline {}({});\n", p.name, entry_funclet)?;
    }
    Ok(())
}

fn write_program(
    oc: &mut dyn Write,
    program: ir::Program,
    numbers_mode: bool,
) -> std::io::Result<()>
{
    write_funclets(
        oc, 
        program.funclets, 
        &program.types, 
        numbers_mode,
        &program.value_functions,
        &program.external_cpu_functions,
        &program.external_gpu_functions,
    )?;
    write_external_cpu_functions(
        oc,
        program.external_cpu_functions,
        &program.types,
        numbers_mode,
    )?;
    write_external_gpu_functions(
        oc,
        program.external_gpu_functions,
        &program.types,
        numbers_mode,
    )?;
    write_value_functions(
        oc,
        program.value_functions,
        &program.types,
        numbers_mode,
    )?;
    write_pipelines(oc, program.pipelines)?;
    Ok(())
}

fn write_definition(
    oc: &mut dyn Write,
    definition: frontend::Definition,
) -> std::io::Result<()>
{
    let v = definition.version;
    if v != (0, 0, 1)
    {
        write!(oc, "Warning: Version differs from pretty printer (0.0.1)")?;
    }
    // Not going to write version for now (I imagine you wouldn't write
    // a language's version name at the top of a file for it)
    //write!(oc, "Version: {}.{}.{}\n", v.0, v.1, v.2)?;
    write_program(oc, definition.program, false);
    Ok(())
}

fn file_to_definition(
    input_file: &mut File,
) -> Result<frontend::Definition, String>
{
    let mut input_string = String::new();
    match input_file.read_to_string(&mut input_string)
    {
        Err(why) => return Err(format!("File could not be read: {}", why)),
        Ok(_) => (),
    };
    match ron::from_str(&input_string)
    {
        Err(why) => Err(format!("Parse error: {}", why)),
        Ok(mut definition) => Ok(definition),
    }
}

pub fn print_file(input_file: &mut File) -> std::io::Result<()>
{
    let mut oc = io::stdout();
    match file_to_definition(input_file)
    {
        Err(msg) =>
        {
            write!(oc, "{}", msg)?;
        }
        Ok(mut definition) => write_definition(&mut oc, definition)?,
    };
    Ok(())
}

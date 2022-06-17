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

fn slice_to_string_delimiter<T>(
    slice: &[T],
    to_string: &dyn Fn(&T) -> String,
    delimiter: String,
) -> String
{
    let mut s = slice
        .iter()
        .map(to_string)
        .fold(String::new(), |acc, s| acc + &s + &delimiter);
    // Removing last delimiter
    for i in 0..delimiter.len()
    {
        s.pop();
    }
    String::from("[") + &s + "]"
}

fn slice_to_string<T>(slice: &[T], to_string: &dyn Fn(&T) -> String)
    -> String
{
    slice_to_string_delimiter(slice, to_string, String::from(", "))
}

fn write_function_type_generic<T>(
    oc: &mut dyn Write,
    input_types: &[T],
    output_types: &[T],
    to_string: &dyn Fn(&T) -> String,
) -> std::io::Result<()>
{
    let input_s = slice_to_string(input_types, to_string);
    let output_s = slice_to_string(output_types, to_string);
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

fn write_funclets(
    oc: &mut dyn Write,
    funclets: Arena<ir::Funclet>,
    types_arena: &Arena<ir::Type>,
    numbers_mode: bool,
) -> std::io::Result<()>
{
    for (num, funclet) in funclets.iter()
    {
        write!(oc, "Funclet {} ({:?}) : ", num, funclet.kind)?;
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
        write_indent(oc, 1)?;
        write!(oc, "Nodes :\n")?;
        for node in funclet.nodes.iter()
        {
            write_indent(oc, 2)?;
            // Node-printing could be subject to change
            write!(oc, "{:?}", node)?;
            write!(oc, "\n")?;
        }

        write_indent(oc, 1)?;
        let to_string = |b| slice_to_string(b, &|u: &usize| u.to_string());
        match &funclet.tail_edge
        {
            ir::TailEdge::Return {
                return_values,
            } =>
            {
                let return_vals_s = to_string(&return_values);
                write!(oc, "Return {}\n", &return_vals_s)?;
            }
            ir::TailEdge::Yield {
                funclet_ids,
                captured_arguments,
                return_values,
            } =>
            {
                let funclet_ids_s = to_string(&funclet_ids);
                let cap_args_s = to_string(&captured_arguments);
                let return_vals_s = to_string(&return_values);
                write!(oc, "Yield {}\n", return_vals_s)?;
                write_indent(oc, 2)?;
                write!(oc, "Funclet IDs: {} \n", funclet_ids_s)?;
                write_indent(oc, 2)?;
                write!(oc, "Captured Arguments: {} \n", cap_args_s)?;
            }
        }

        write!(oc, "}}\n")?;
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
        write!(oc, "\n")?;
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
        write!(oc, "GPU {} {{\n", func.name)?;

        write_indent(oc, 2)?;
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
        write!(oc, "\n")?;

        write_indent(oc, 2)?;
        write!(oc, "Entry point: {}\n", func.entry_point)?;

        write_indent(oc, 2)?;
        let resource_bindings_str = slice_to_string_delimiter(
            &(*func.resource_bindings),
            &|rb| {
                format!(
                    "Group {}; Binding {}; {:?} -> {:?}",
                    rb.group, rb.binding, rb.input, rb.output
                )
            },
            String::from(",\n    "),
        );
        write!(oc, "{}\n", resource_bindings_str)?;

        // Shader module content seems inherently not pretty; it just looks
        // like a block of WGSL code, so I am not going to print it (for now)
        //
        //write_indent(oc, 2)?;
        //write!(oc, "Shader module content: {:?}\n",
        //func.shader_module_content)?;
        write_indent(oc, 1)?;
        write!(oc, "}}\n");
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
        write_indent(oc, 1)?;
        write!(oc, "ValueFunc {} : ", vf.name)?;
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
        write!(oc, " (Default Funclet ID: {:?})", vf.default_funclet_id)?;
        write!(oc, "\n")?;
    }
    Ok(())
}

fn write_pipelines(
    oc: &mut dyn Write,
    pipelines: Vec<ir::Pipeline>,
) -> std::io::Result<()>
{
    write!(oc, "Pipelines:\n")?;
    for p in pipelines.iter()
    {
        write_indent(oc, 1)?;
        write!(oc, "{} [entry funclet {}]\n", p.name, p.entry_funclet)?;
    }
    Ok(())
}

fn write_program(
    oc: &mut dyn Write,
    program: ir::Program,
    numbers_mode: bool,
) -> std::io::Result<()>
{
    write_funclets(oc, program.funclets, &program.types, numbers_mode)?;
    write!(oc, "\n")?;
    write_external_cpu_functions(
        oc,
        program.external_cpu_functions,
        &program.types,
        numbers_mode,
    )?;
    write!(oc, "\n")?;
    write_external_gpu_functions(
        oc,
        program.external_gpu_functions,
        &program.types,
        numbers_mode,
    )?;
    write!(oc, "\n")?;
    write_value_functions(
        oc,
        program.value_functions,
        &program.types,
        numbers_mode,
    )?;
    write!(oc, "\n")?;
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
    write!(oc, "Version: {}.{}.{}\n", v.0, v.1, v.2)?;
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

use crate::syntax::ast;
use caiman::assembly::ast as asm;
use super::typing;
use std::collections::HashMap;

pub struct FunctionClassContext
{
    // Funclet IDs to Function class IDs
    map: HashMap<String, String>,
}

impl FunctionClassContext
{
    pub fn get(&self, funclet_name: &str) -> Option<asm::FuncletId>
    {
        self.map.get(funclet_name).map(|elt| asm::FuncletId(elt.clone()))
    }
}

pub fn make(program: &ast::Program) -> (Vec<asm::FunctionClass>, FunctionClassContext)
{
    // For ease of use and less nesting
    let ast_function_classes: Vec<(String, Vec<String>)> = program
        .iter()
        .filter_map(|(_, decl)| match decl {
            ast::DeclKind::FunctionClass { name, functions } => {
                Some((name.clone(), functions.clone()))
            },
            _ => None,
        })
        .collect();


    // TODO: add to function classes vector below all the "default" function classes that should
    // be made if none are declared for a value funclet

    let mut funclet_fc_map: HashMap<String, String> = HashMap::new();
    let mut function_classes: Vec<asm::FunctionClass> = Vec::new();
    for (name, functions) in ast_function_classes.into_iter() {
        for funclet_name in functions.iter() {
            let was_duplicate_insert =
                funclet_fc_map.insert(funclet_name.clone(), name.clone()).is_some();
            if was_duplicate_insert {
                println!("WARNING: Multiple function classes include the funclet {}", funclet_name)
            }
        }

        match type_of_first_matching_decl(program, &functions) {
            None => panic!("Empty function class declared"),
            Some((t_in, t_out)) => {
                let input_types = t_in.into_iter().map(typing::convert_value_type).collect();
                let output_types = vec![typing::convert_value_type(t_out)];
                function_classes.push(asm::FunctionClass { name, input_types, output_types });
            },
        }
    }

    (function_classes, FunctionClassContext { map: funclet_fc_map })
}

fn type_of_first_matching_decl(
    program: &ast::Program,
    funclet_names: &Vec<String>,
) -> Option<(Vec<ast::value::Type>, ast::value::Type)>
{
    for (_, decl) in program.iter() {
        match decl {
            ast::DeclKind::ValueFunclet { name, input, output: (_, t_out), statements: _ } => {
                if funclet_names.contains(name) {
                    let t_in = input.iter().map(|(_, input_type)| input_type.clone()).collect();
                    return Some((t_in, t_out.clone()));
                }
            },
            ast::DeclKind::ExternCPU { name, input, output, } => {
                if funclet_names.contains(name) {
                    return Some((input.clone(), output.clone()));
                }
            },
            _ => (),
        }
    }
    None
}

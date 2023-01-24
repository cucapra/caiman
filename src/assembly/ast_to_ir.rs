
use std::collections::HashMap;
use crate::{ir, frontend};
use crate::ir::{ffi};
use crate::arena::Arena;
use crate::assembly::{ast, context};
use crate::assembly::ast::UncheckedDict;
use crate::assembly::context::Context;

// Utility

fn lookup(dict : &ast::UncheckedDict, key : &ast::Value, context : &mut Context) -> ast::DictValue {
    for pair in dict.iter() {
        if *key == pair.key {
            return pair.value.clone()
        }
    }
    panic!(format!("Missing dictionary element {:?}", key));
}

/*
	ID(String),
    FunctionLoc(RemoteNodeId),
    VarName(String),
    FnName(String),
    Type(Type),
    Place(ir::Place),
    Stage(ir::ResourceQueueStage),
    Tag(Tag),
 */

fn value_string(v : &ast::Value, context : &mut Context) -> String {
    match v {
        ast::Value::ID(s) => s.clone(),
        _ => panic!(format!("Expected id got {:?}", v))
    }
}

fn value_function_loc(v : &ast::Value, context : &mut Context) -> ir::RemoteNodeId {
    match v {
        ast::Value::FunctionLoc(remote) => ir::RemoteNodeId {
            funclet_id : *context.local_funclet_id(remote.funclet_id.clone()),
            node_id : *context.local_funclet_id(remote.node_id.clone())
        },
        _ => panic!(format!("Expected function location got {:?}", v))
    }
}

fn value_var_name(v : &ast::Value, context : &mut Context) -> usize {
    match v {
        ast::Value::VarName(s) => *context.current_node_id(s.clone()),
        _ => panic!(format!("Expected variable name got {:?}", v))
    }
}

fn value_funclet_name(v : &ast::Value, context : &mut Context) -> context::Location {
    match v {
        ast::Value::FnName(s) => context.funclet_id(s.clone()),
        _ => panic!(format!("Expected funclet name got {:?}", v))
    }
}

fn value_type(v : &ast::Value, context : &mut Context) -> context::Location {
    match v {
        ast::Value::Type(t) => match t {
            ast::Type::FFI(name) => context::Location::FFI(
                *context.ffi_type_id(name.clone())),
            ast::Type::Local(name) => context::Location::Local(
                *context.local_type_id(name.clone()))
        }
        _ => panic!(format!("Expected type got {:?}", v))
    }
}

fn value_place(v : &ast::Value, context : &mut Context) -> ir::Place {
    match v {
        ast::Value::Place(p) => p.clone(),
        _ => panic!(format!("Expected place got {:?}", v))
    }
}

fn value_stage(v : &ast::Value, context : &mut Context) -> ir::ResourceQueueStage {
    match v {
        ast::Value::Stage(s) => s.clone(),
        _ => panic!(format!("Expected stage got {:?}", v))
    }
}

fn value_value_tag(v : &ast::Value, context : &mut Context) -> ir::ValueTag {
    match v {
        ast::Value::Tag(t) => match t {
            ast::Tag::ValueTag(v) => {
                match v {

                }
            }
        },
        _ => panic!(format!("Expected stage got {:?}", v))
    }
}

// Translation

fn ir_version(version : &ast::Version, context : &mut Context) -> (u32, u32, u32) {
    (version.major, version.minor, version.detailed)
}

fn ir_native_interface(program : &ast::Program, context : &mut Context) -> ffi::NativeInterface {
    todo!()
}

fn ir_types(types : &Vec<ast::LocalType>, context : &mut Context) -> Arena<ir::Type> {
    let mut result = Arena::new();
    for typ in types {
        let new_type = match typ.event { // only supported custom types atm
            true => {
                ir::Type::Event {
                    place: ,
                }
            },
            false => {

            }
        };
        result.create(new_type);
    }
    result
}

fn ir_funclets(funclets : &ast::FuncletDefs, context : &mut Context) -> Arena<ir::Funclet> {
    todo!()
}

fn ir_pipelines(pipelines : &ast::Pipelines, context : &mut Context) -> Vec<ir::Pipeline> {
    let mut result = Vec::new();
    for pipeline in pipelines.iter() {
        let new_pipeline = ir::Pipeline {
            name: pipeline.name.clone(),
            entry_funclet: *context.local_funclet_id(pipeline.funclet.clone()),
            yield_points: Default::default(),
        };
        result.push(new_pipeline);
    }
    result
}

fn ir_value_extras(extras : &ast::Extras, context : &mut Context)
-> HashMap<ir::FuncletId, ir::ValueFuncletExtra> {
    todo!()
}

fn ir_scheduling_extras(extras : &ast::Extras, context : &mut Context)
-> HashMap<ir::FuncletId, ir::SchedulingFuncletExtra> {
    todo!()
}

fn ir_program(program : ast::Program, context : &mut Context) -> ir::Program {
    ir::Program {
        native_interface: ir_native_interface(&program, context),
        types: ir_types(&program.types.local_types, context),
        funclets: ir_funclets(&program.funclets, context),
        value_functions: Arena::new(),
        pipelines: ir_pipelines(&program.pipelines, context),
        value_funclet_extras: ir_value_extras(&program.extras, context),
        scheduling_funclet_extras: ir_scheduling_extras(&program.extras, context),
    }
}

pub fn ast_to_ir(program : ast::Program, context : &mut Context) -> frontend::Definition {
    frontend::Definition {
        version: ir_version(&program.version, context),
        program: ir_program(program, context),
    }
}
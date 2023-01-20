
use std::collections::HashMap;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use clap::App;
use naga::SwizzleComponent::Z;
use pest::error::Error;
use pest::iterators::{Pairs, Pair};
use pest::Parser;
use pest_derive::Parser;
#[derive(Parser)]
#[grammar = "src/assembly/caimanir.pest"]
pub struct IRParser;
use crate::{ir, frontend};
use crate::ir::ffi;
use crate::arena::Arena;
use crate::assembly::ast;

// Stupid file structure way too much work to break up for some stupid reason

// Context

#[derive(Debug, Clone)]
struct TypeIndices {
    ffi_index : usize,
    ir_index : usize
}

#[derive(Debug, Clone)]
struct FuncletIndices {
    location : ir::RemoteNodeId
}

#[derive(Debug, Clone)]
enum Indices {
    TypeIndices(TypeIndices),
    FuncletIndices(FuncletIndices),
    None
}

#[derive(Debug)]
struct Context {
    external_type_map : HashMap<String, usize>,
    type_map : HashMap<String, ast::Type>,
    var_map : HashMap<String, ast::Var>,
    funclet_map : HashMap<String, ast::Funclet>,
    indices : Indices
}

impl Context {

    fn add_external_type(&mut self, ffi_type : String) {
        let index = match &mut self.indices {
            Indices::TypeIndices(t) => {
                let index = t.ffi_index;
                t.ffi_index += 1;
                index
            },
            _ => panic!("Invalid access attempt")
        };
        self.external_type_map.insert(ffi_type, index);
    }

    fn add_type(&mut self, name : String, typ : ast::Type) {
        self.type_map.insert(name, typ);
    }

    fn add_var(&mut self, name : String, var : ast::Var) {
        self.var_map.insert(name, var);
    }

    fn add_funclet(&mut self, name : String, funclet : ast::Funclet) {
        self.funclet_map.insert(name, funclet);
    }

    fn var_id(&mut self, name : String) -> usize {
        self.var_map.get(name.as_str()).unwrap().id
    }

    fn funclet_id(&mut self, name : String) -> usize {
        self.funclet_map.get(name.as_str()).unwrap().id
    }

    fn remote_id(&mut self, funclet : String, var : String) -> ir::RemoteNodeId {
        ir::RemoteNodeId {
            funclet_id: self.funclet_id(funclet),
            node_id: self.funclet_id(var)
        }
    }

    fn get_type_index(&mut self) -> &mut TypeIndices {
        match &mut self.indices {
            Indices::TypeIndices(t) => t,
            _ => panic!("Invalid access attempt")
        }
    }

    fn get_funclet_index(&mut self) -> &mut FuncletIndices {
        match &mut self.indices {
            Indices::FuncletIndices(t) => t,
            _ => panic!("Invalid access attempt")
        }
    }
}

// Fanciness

// Why this doesn't work in general is a bit of a mystery to me tbh, but here we are
// fn compose<'a, T, U, V, W, G, F>(f: F, g: G) -> Box<dyn Fn(T, U) -> W + 'a>
//     where
//         F: Fn(T, U) -> V + 'a,
//         G: Fn(V) -> W + 'a,
// {
//     Box::new(move |p, c| g(f(p, c)))
// }

fn compose_pair<'a, T, U, G, F>(f: F, g: G)
-> Box<dyn Fn(&mut Pairs<Rule>, &mut Context) -> U + 'a>
    where
        F: Fn(&mut Pairs<Rule>, &mut Context) -> T + 'a,
        G: Fn(T) -> U + 'a,
{
    Box::new(move |p, c| g(f(p, c)))
}

fn compose_str<'a, T, U, G, F>(f: F, g: G)
-> Box<dyn Fn(String, &mut Context) -> U + 'a>
    where
        F: Fn(String, &mut Context) -> T + 'a,
        G: Fn(T) -> U + 'a,
{
    Box::new(move |s, c| g(f(s, c)))
}

// Rule stuff

enum Application<'a, T> {
    P(Box<dyn Fn(&mut Pairs<Rule>, &mut Context) -> T + 'a>),
    S(Box<dyn Fn(String, &mut Context) -> T + 'a>)
}

struct RuleApp<'a, T> {
    rule : Rule,
    unwrap : usize,
    application : Application<'a, T>
}

fn rule_app_as_str<T>(rule : &RuleApp<T>) -> String {
    return format!("{:?} {:?}", rule.rule, rule.unwrap);
}

fn rule_app_vec_as_str<T>(rules : &Vec<RuleApp<T>>) -> String {
    let mut result = Vec::new();
    for rule in rules.iter() {
        result.push(rule_app_as_str(rule));
    }
    format!("{:?}", result)
}

fn rule_pair_unwrap<'a, T>(rule: Rule, unwrap : usize,
                      apply: Box<dyn Fn(&mut Pairs<Rule>, &mut Context) -> T + 'a>) -> RuleApp<'a, T> {
    let application = Application::P(apply);
    RuleApp { rule , unwrap, application }
}

fn rule_pair_boxed<'a, T>(rule: Rule, apply: Box<dyn Fn(&mut Pairs<Rule>, &mut Context) -> T + 'a>)
-> RuleApp<'a, T> {
    rule_pair_unwrap(rule, 0, apply)
}

fn rule_pair<'a, T : 'a>(rule: Rule, apply: fn(&mut Pairs<Rule>, &mut Context) -> T) -> RuleApp<'a, T> {
    rule_pair_unwrap(rule, 0, Box::new(apply))
}

fn rule_str_unwrap<'a, T>(rule: Rule, unwrap : usize,
                      apply: Box<dyn Fn(String, &mut Context) -> T + 'a>) -> RuleApp<'a, T> {
    let application = Application::S(apply);
    RuleApp { rule , unwrap, application }
}

fn rule_str_boxed<'a, T>(rule: Rule, apply: Box<dyn Fn(String, &mut Context) -> T + 'a>)
-> RuleApp<'a, T> {
    rule_str_unwrap(rule, 0, apply)
}

fn rule_str<'a, T : 'a>(rule: Rule, apply: fn(String, &mut Context) -> T) -> RuleApp<'a, T> {
    rule_str_unwrap(rule, 0, Box::new(apply))
}

fn dbg_pairs(pairs: &mut Pairs<Rule>, context : &mut Context) {
    for pair in pairs {
        dbg!(pair.as_rule());
    }
}

fn check_rule(potentials: Vec<Rule>, rule : Rule, context : &mut Context) -> bool {
    for potential in potentials {
        if rule == potential {
            return true
        }
    }
    false
}

fn is_rule(potentials: Vec<Rule>, pairs: &mut Pairs<Rule>, context : &mut Context) -> bool {
    check_rule(potentials, pairs.next().unwrap().as_rule(), context)
}

fn require_rule(potentials: Vec<Rule>, pairs: &mut Pairs<Rule>, context : &mut Context) {
    let rule = pairs.next().unwrap().as_rule();
    if !check_rule(potentials, rule, context) {
        panic!(format!("Unexpected parse rule {:?}", rule))
    }
}

fn apply_pair<T>(potentials: &Vec<RuleApp<T>>, pair : Pair<Rule>, context : &mut Context) -> Option<T> {
    for potential in potentials {
        if pair.as_rule() == potential.rule {
            return match &potential.application {
                Application::P(apply) => {
                    // duplicated cause this is faster for top-level stuff
                    let mut pairs = pair.into_inner();
                    for unwrap in 0..potential.unwrap {
                        let new_pair = pairs.next().unwrap();
                        pairs = new_pair.into_inner();
                    }
                    Some(apply(&mut pairs, context))
                },
                Application::S(apply) => {
                    // cloning is slow, but fixing takes work
                    let mut new_pair = pair.clone();
                    let mut pairs = pair.into_inner();
                    for unwrap in 0..potential.unwrap {
                        new_pair = pairs.next().unwrap();
                        pairs = new_pair.clone().into_inner(); // whatever, just whatever
                    }
                    let mut s = "".to_string();
                    for line in new_pair.as_span().lines() {
                        s += line;
                    }
                    Some(apply(s, context))
                },
            }
        }
    }
    None
}

fn optional<T>(potentials: Vec<RuleApp<T>>, pairs: &mut Pairs<Rule>, context : &mut Context) -> Option<T> {
    apply_pair(&potentials, pairs.next().unwrap(), context)
}

fn optional_all<T>(potentials: Vec<RuleApp<T>>, pairs: &mut Pairs<Rule>, context : &mut Context) -> Vec<T> {
    let mut result = Vec::new();
    for pair in pairs {
        match apply_pair(&potentials, pair, context) {
            None => {},
            Some(res) => { result.push(res); }
        }
    }
    result
}

fn expect_raw<T>(potentials: &Vec<RuleApp<T>>, pair: Pair<Rule>, context : &mut Context) -> T {
    let rule = pair.as_rule();
    match apply_pair(&potentials, pair, context)
    {
        Some(result) => result,
        None => panic!(format!("Expected rule {:?}, got {:?}", rule_app_vec_as_str(potentials), rule))
    }
}

fn expect_vec<T>(potentials: Vec<RuleApp<T>>, pairs: &mut Pairs<Rule>, context : &mut Context) -> T {
    let pair = pairs.next().unwrap();
    expect_raw(&potentials, pair, context)
}

fn expect<T>(potential: RuleApp<T>, pairs: &mut Pairs<Rule>, context : &mut Context) -> T {
    expect_vec(vec![potential], pairs, context)
}

fn expect_all_vec<T>(potentials: Vec<RuleApp<T>>, pairs: &mut Pairs<Rule>, context : &mut Context) -> Vec<T> {
    let mut result = Vec::new();
    for pair in pairs {
        let rule = pair.as_rule();
        result.push(expect_raw(&potentials, pair, context));
    }
    result
}

fn expect_all<T>(potential: RuleApp<T>, pairs: &mut Pairs<Rule>, context : &mut Context) -> Vec<T> {
    expect_all_vec(vec![potential], pairs, context)
}

// Core Reading

fn read_raw(s : String, context : &mut Context) -> String { s }

fn read_string(pairs : &mut Pairs<Rule>, context : &mut Context) -> String {
    let rule = rule_str(Rule::id, |s, c| s);
    expect(rule, pairs, context)
}

fn read_id(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Value {
    let rule = rule_str(Rule::id, |s, c| s);
    ast::Value::ID(expect(rule, pairs, context))
}

fn read_ffi_raw(s : String, context : &mut Context) -> ffi::Type {
    match s.as_str() {
        "i32" => { ffi::Type::I32 },
        s => panic!("Unknown type name {}", s)
    }
}

fn read_type(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Value {
    let mut rules = Vec::new();
    let ffi_fn = compose_str(read_ffi_raw, ast::Type::FFI);
    rules.push(rule_str_boxed(Rule::ffi_type, ffi_fn));
    let rule_ir= compose_pair(read_string, ast::Type::IR);
    rules.push(rule_pair_unwrap(Rule::var_name, 1, rule_ir));
    ast::Value::Type(expect_vec(rules, pairs, context))
}

fn read_var(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Value {
    let rule = rule_pair(Rule::id, read_string);
    ast::Value::FnName(expect(rule, pairs, context))
}

fn read_fn(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Value {
    let rule = rule_pair(Rule::id, read_string);
    ast::Value::Type(expect(rule, pairs, context))
}

fn read_function_loc(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Value {
    let rule_func = rule_pair(Rule::fn_name, read_string);
    let rule_var = rule_pair(Rule::var_name, read_string);
    let fun_name = expect(rule_func, pairs, context);
    let var_name = expect(rule_var, pairs, context);
    ast::Value::FunctionLoc(ast::RemoteNodeId { funclet_id : fun_name, node_id : var_name })
}

fn read_place(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Value {
    let rule = rule_pair(Rule::id, read_string);
    ast::Value::FnName(expect(rule, pairs, context));
    todo!()
}

fn read_stage(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Value {
    let rule = rule_pair(Rule::id, read_string);
    ast::Value::FnName(expect(rule, pairs, context));
    todo!()
}

fn read_tag(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Value {
    let rule = rule_pair(Rule::id, read_string);
    ast::Value::FnName(expect(rule, pairs, context));
    todo!()
}

fn read_value(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Value {
    // funclet_loc | var_name | fn_name | typ | place | stage | tag
    let rule_loc = rule_pair(Rule::funclet_loc, read_function_loc);
    let rule_var = rule_pair(Rule::var_name, read_var);
    let rule_fn = rule_pair(Rule::fn_name, read_fn);
    let rule_typ = rule_pair(Rule::typ, read_typ);
    let rule_place = rule_pair(Rule::place, read_place);
    let rule_place = rule_pair(Rule::stage, read_stage);
    let rule_place = rule_pair(Rule::tag, read_tag);
}

fn read_dict_value(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::DictValue {
    let value_map = compose_pair(read_value,ast::DictValue::Raw);
    let rule_value = rule_pair_boxed(Rule::value, value_map);
    expect_vec(vec![rule_value], pairs, context)
}

fn read_dict_key(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Value {
    let rule_id = rule_pair(Rule::id, read_id);
    let rule_var_name = rule_pair_unwrap(Rule::var_name_sep, 1,
                                        Box::new(read_var));
    expect_vec(vec![rule_id, rule_var_name], pairs, context)
}

fn read_dict_element(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::DictPair {
    let rule_key = rule_pair(Rule::dict_key, read_dict_key);
    let rule_value = rule_pair(Rule::dict_value, read_dict_value);
    let key = expect(rule_key, pairs, context);
    let value = expect(rule_value, pairs, context);
    ast::DictPair { key, value }
}

fn read_unchecked_dict(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::UncheckedDict {
    let rule = rule_pair(Rule::dict_element, read_dict_element);
    expect_all(rule, pairs, context)
}

// Readers

fn is_slot(s : String, context : &mut Context) -> bool {
    match s.as_str() {
        "slot" => true,
        "event" => false,
        _ => panic!(format!("Unexpected slot check {}", s))
    }
}

fn read_ir_type_decl(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::TypeDecl {
    let event_rule = rule_str_unwrap(
        Rule::type_decl_sep, 1, Box::new(is_slot));
    let event = expect(event_rule, pairs, context);
    let name_rule = rule_pair(Rule::type_name, read_string);
    let type_name = expect(name_rule, pairs, context);
    let dict_rule = rule_pair(Rule::unchecked_dict, read_unchecked_dict);
    let data = expect(dict_rule, pairs, context);
    let mut index = context.get_type_index();
    let result = ast::IRType {
        id : index.ir_index,
        event,
        type_name,
        data,
    };
    index.ir_index += 1;
    ast::TypeDecl::IR(result)
}

fn read_ffi_type_decl(s : String, context : &mut Context) -> ast::TypeDecl {
    context.add_external_type(s.to_string());

}

fn read_type_def(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::TypeDecl {
    let ffi_rule = rule_str(Rule::ffi_type, read_ffi_type);
    let ir_rule = rule_pair(Rule::type_name, read_ir_type_decl);
    expect_vec(vec![ffi_rule, ir_rule], pairs, context)
}

fn read_types(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Types {
    let rule = rule_pair(Rule::type_def,read_type_def);
    let mut result = ast::Types { ffi_types : Arena::new(), ir_types : Arena::new() };
    for next in expect_all(rule, pairs, context).drain(..) {
        match next {
            ast::TypeDecl::FFI(t) => { result.ffi_types.create(t); },
            ast::TypeDecl::IR(t) => { result.ir_types.create(t); }
        }
    }
    result
}

fn read_funclets(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Funclets {
    todo!()
}

fn read_extras(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Extras {
    todo!()
}

fn read_pipelines(pairs : &mut Pairs<Rule>, context : &mut Context) -> Vec<ir::Pipeline> {
    todo!()
}

fn read_program(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Program {
    context.indices = Indices::TypeIndices(TypeIndices {
        ffi_index: 0,
        ir_index: 0,
    });
    let found_types = expect(
        rule_pair(Rule::types, read_types), pairs, context);
    context.indices = Indices::FuncletIndices(FuncletIndices {
        location : ir::RemoteNodeId { funclet_id: 0, node_id: 0 }
    });
    let found_funclets = expect(
        rule_pair(Rule::funclets, read_funclets), pairs, context);
    context.indices = Indices::None;
    let found_extras = expect(
        rule_pair(Rule::extras, read_extras), pairs, context);
    let found_pipelines = expect(
        rule_pair(Rule::pipelines, read_pipelines), pairs, context);
    require_rule(vec![Rule::EOI], pairs, context);

    let native = ffi::NativeInterface {
        types: found_types.ffi_types,
        external_cpu_functions: found_funclets.external_cpu,
        external_gpu_functions: found_funclets.external_gpu,
    };

    todo!()
}


fn construct_program(program : ast::Program, context : &mut Context) -> ir::Program {

    todo!()

    // ir::Program {
    //     native_interface: native,
    //     types: found_types.ir_types,
    //     funclets: found_funclets.program,
    //     value_functions: Arena::new(),
    //     pipelines: found_pipelines,
    //     value_funclet_extras: found_extras.value_funclet_extras,
    //     scheduling_funclet_extras: found_extras.scheduling_funclet_extras,
    // }

}

fn read_definition(pairs : &mut Pairs<Rule>, context : &mut Context) -> frontend::Definition {
    let program = expect(
        rule_pair(Rule::program, read_program), pairs, context);
    frontend::Definition { version : (0, 0, 1), program : construct_program(program, context) }
}

pub fn parse(code : &str) ->
Result<frontend::Definition, frontend::CompileError> {
    let parsed = IRParser::parse(Rule::program, code);
    let mut context = Context {
        external_type_map: HashMap::new(),
        type_map: HashMap::new(),
        var_map: HashMap::new(),
        funclet_map: HashMap::new(),
        indices: Indices::None,
    };
    match parsed {
        Err(why) => Err(crate::frontend::CompileError{ message: (format!("{}", why)) }),
        Ok(mut parse_result) => Ok(read_definition(&mut parse_result, &mut context))
    }
}
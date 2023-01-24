
use std::collections::HashMap;
use pest::iterators::{Pairs, Pair};
use pest::Parser;
use pest_derive::Parser;
#[derive(Parser)]
#[grammar = "src/assembly/caimanir.pest"]
pub struct IRParser;
use crate::{ir, frontend};
use crate::ir::{ffi};
use crate::arena::Arena;
use crate::assembly::ast;
use crate::assembly::ast_to_ir::ast_to_ir;
use crate::assembly::context::{new_context, Context};

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

fn option_to_vec<T>(o : Option<Vec<T>>) -> Vec<T> {
    match o {
        None => Vec::new(),
        Some(v) => v
    }
}

// Rule stuff

fn unexpected(value : String) -> String {
    format!("Unexpected string {}", value)
}

fn unexpected_rule<T>(potentials : &Vec<RuleApp<T>>, rule : Rule) -> String {
    format!("Expected rule {:?}, got {:?}", rule_app_vec_as_str(potentials), rule)
}

fn unexpected_rule_raw(potentials : Vec<Rule>, rule : Rule) -> String {
    format!("Expected rule {:?}, got {:?}", potentials, rule)
}

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

fn check_rule(potentials: Vec<Rule>, rule : Rule, context : &mut Context) -> bool {
    for potential in potentials {
        if rule == potential {
            return true
        }
    }
    false
}

fn is_rule(potentials: Vec<Rule>, pairs: &mut Pairs<Rule>, context : &mut Context) -> bool {
    match pairs.peek() {
        None => false,
        Some(pair) => check_rule(potentials, pair.as_rule(), context)
    }
}

fn require_rules(potentials: Vec<Rule>, pairs: &mut Pairs<Rule>, context : &mut Context) {
    let rule = pairs.next().unwrap().as_rule();
    if !check_rule(potentials, rule, context) {
        panic!(format!("Unexpected parse rule {:?}", rule))
    }
}

fn require_rule(potential: Rule, pairs: &mut Pairs<Rule>, context : &mut Context) {
    require_rules(vec![potential], pairs, context)
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
                    let mut new_pair= pair.clone();
                    let mut pairs = pair.into_inner();
                    for unwrap in 0..potential.unwrap {
                        new_pair = pairs.next().unwrap();
                        pairs = new_pair.clone().into_inner(); // whatever, just whatever
                    }
                    Some(apply(new_pair.as_span().as_str().to_string(), context))
                },
            }
        }
    }
    None
}

fn optional_vec<T>(potentials: Vec<RuleApp<T>>, pairs: &mut Pairs<Rule>, context : &mut Context) -> Option<T> {
    match pairs.peek() {
        None => None,
        Some(pair) => {
            match apply_pair(&potentials, pair, context) {
                None => None,
                t => { pairs.next(); t }
            }
        }
    }
}

fn optional<T>(potentials: RuleApp<T>, pairs: &mut Pairs<Rule>, context : &mut Context) -> Option<T> {
    optional_vec(vec![potentials], pairs, context)
}

fn expect_raw<T>(potentials: &Vec<RuleApp<T>>, pair: Pair<Rule>, context : &mut Context) -> T {
    let rule = pair.as_rule();
    let span = pair.as_span();
    match apply_pair(&potentials, pair, context)
    {
        Some(result) => result,
        None =>
            {
                println!("{:?}", span);
                panic!(unexpected_rule(potentials, rule))
            }
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
        result.push(expect_raw(&potentials, pair, context));
    }
    result
}

fn expect_all<T>(potential: RuleApp<T>, pairs: &mut Pairs<Rule>, context : &mut Context) -> Vec<T> {
    expect_all_vec(vec![potential], pairs, context)
}

// Core Reading

fn read_n(s : String, context : &mut Context) -> usize {
    s.parse::<usize>().unwrap()
}

fn rule_n<'a>() -> RuleApp<'a, usize> {
    rule_str(Rule::n, read_n)
}

fn read_string(s : String, context : &mut Context) -> String { s }

fn rule_string<'a>(rule : Rule) -> RuleApp<'a, String> {
    rule_str(rule, read_string)
}

fn rule_id_raw<'a>() -> RuleApp<'a, String> {
    rule_str(Rule::id, read_string)
}

fn rule_n_raw<'a>() -> RuleApp<'a, String> {
    rule_str(Rule::n, read_string)
}

fn read_string_clean(s : String, context : &mut Context) -> String {
    (&s[1..s.len()-1]).to_string()
}

fn rule_string_clean<'a>() -> RuleApp<'a, String> {
    rule_str(Rule::str, read_string_clean)
}

fn read_type_raw(pairs : &mut Pairs<Rule>, context : &mut Context) -> String {
    let mut rules = Vec::new();
    rules.push(rule_str(Rule::ffi_type, read_string));
    rules.push(rule_str_unwrap(Rule::type_name, 1, Box::new(read_string)));
    expect_vec(rules, pairs, context)
}

fn rule_type_raw<'a>() -> RuleApp<'a, String> {
    rule_pair(Rule::typ, read_type_raw)
}

fn read_id(s : String, context : &mut Context) -> ast::Value {
    ast::Value::ID(s)
}

fn rule_id<'a>() -> RuleApp<'a, ast::Value> {
    rule_str(Rule::id, read_id)
}

fn read_ffi_raw(s : String, context : &mut Context) -> ffi::Type {
    match s.as_str() {
        "i32" => { ffi::Type::I32 },
        s => panic!("Unknown type name {}", s)
    }
}

fn rule_ffi_type<'a>() -> RuleApp<'a, ffi::Type> {
    rule_str(Rule::ffi_type, read_ffi_raw)
}

fn rule_ffi_typ_sep<'a>() -> RuleApp<'a, ffi::Type> {
    rule_str_unwrap(Rule::ffi_type_sep, 1, Box::new(read_ffi_raw))
}

fn read_type(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Type {
    let mut rules = Vec::new();
    let ffi_fn = compose_str(read_string, ast::Type::FFI);
    rules.push(rule_str_boxed(Rule::ffi_type, ffi_fn));
    let rule_ir= compose_str(read_string, ast::Type::Local);
    rules.push(rule_str_unwrap(Rule::type_name, 1, rule_ir));
    expect_vec(rules, pairs, context)
}

fn rule_type<'a>() -> RuleApp<'a, ast::Type> {
    rule_pair(Rule::typ, read_type)
}

fn rule_type_sep<'a>() -> RuleApp<'a, ast::Type> {
    rule_pair_unwrap(Rule::typ_sep, 1, Box::new(read_type))
}

fn read_var_name(pairs : &mut Pairs<Rule>, context : &mut Context) -> String {
    expect_vec(vec![rule_id_raw(), rule_n_raw()], pairs, context)
}

fn rule_var_name<'a>() -> RuleApp<'a, String> {
    rule_pair(Rule::var_name, read_var_name)
}

fn read_fn_name(pairs : &mut Pairs<Rule>, context : &mut Context) -> String {
    expect(rule_id_raw(), pairs, context)
}

fn rule_fn_name<'a>() -> RuleApp<'a, String> {
    rule_pair(Rule::fn_name, read_fn_name)
}

fn rule_fn_name_sep<'a>() -> RuleApp<'a, String> {
    rule_pair_unwrap(Rule::fn_name_sep, 1, Box::new(read_fn_name))
}

fn read_funclet_loc(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::RemoteNodeId {
    let rule_func = rule_str(Rule::fn_name, read_string);
    let rule_var = rule_str(Rule::var_name, read_string);
    let fun_name = expect(rule_func, pairs, context);
    let var_name = expect(rule_var, pairs, context);
    ast::RemoteNodeId { funclet_id : fun_name, node_id : var_name }
}

fn rule_funclet_loc<'a>() -> RuleApp<'a, ast::RemoteNodeId> {
    rule_pair(Rule::funclet_loc, read_funclet_loc)
}

fn read_place(s : String, context : &mut Context) -> ir::Place {
    match s.as_str() {
        "local" => ir::Place::Local,
        "cpu" => ir::Place::Cpu,
        "gpu" => ir::Place::Gpu,
        _ => panic!(unexpected(s))
    }
}

fn rule_place<'a>() -> RuleApp<'a, ir::Place> {
    rule_str(Rule::place, read_place)
}

fn read_stage(s : String, context : &mut Context) -> ir::ResourceQueueStage {
    match s.as_str() {
        "unbound" => ir::ResourceQueueStage::Unbound,
        "bound" => ir::ResourceQueueStage::Bound,
        "encoded" => ir::ResourceQueueStage::Encoded,
        "submitted" => ir::ResourceQueueStage::Submitted,
        "ready" => ir::ResourceQueueStage::Ready,
        "dead" => ir::ResourceQueueStage::Dead,
        _ => panic!(unexpected((s)))
    }
}

fn rule_stage<'a>() -> RuleApp<'a, ir::ResourceQueueStage> {
    rule_str(Rule::stage, read_stage)
}

fn read_tag_core_op(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::TagCore {
    let op_type = expect(rule_string(Rule::tag_core_op), pairs, context);
    let funclet_loc = expect(rule_funclet_loc(), pairs, context);
    match op_type.as_str() {
        // "operation" | "input" | "output"
        "operation" => ast::TagCore::Operation(funclet_loc),
        "input" =>  ast::TagCore::Input(funclet_loc),
        "output" => ast::TagCore::Output(funclet_loc),
        _ => panic!(unexpected(op_type))
    }
}

fn read_tag_core(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::TagCore {
    let pair = pairs.peek().unwrap();
    let rule = pair.as_rule();
    match rule {
        Rule::tag_none => ast::TagCore::None,
        Rule::tag_core_op => read_tag_core_op(pairs, context),
        _ => panic!(unexpected_rule_raw(vec![Rule::tag_none, Rule::tag_core_op], rule))
    }
}

fn rule_tag_core<'a>() -> RuleApp<'a, ast::TagCore> {
    rule_pair(Rule::tag_core, read_tag_core)
}

fn read_timeline_tag(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::TimelineTag {
    ast::TimelineTag::Core(expect(rule_tag_core(), pairs, context))
}

fn read_spatial_tag(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::SpatialTag {
    ast::SpatialTag::Core(expect(rule_tag_core(), pairs, context))
}

fn read_value_tag_loc(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::ValueTag {
    let op_type = expect(rule_string(Rule::value_tag_op), pairs, context);
    let funclet_loc = expect(rule_funclet_loc(), pairs, context);
    match op_type.as_str() {
        // "function_input" | "function_output"
        "function_input" => ast::ValueTag::FunctionInput(funclet_loc),
        "function_output" =>  ast::ValueTag::FunctionOutput(funclet_loc),
        _ => panic!(unexpected(op_type))
    }
}

fn read_value_tag(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::ValueTag {
    let mut rules = vec![];
    let app = compose_pair(read_tag_core, ast::ValueTag::Core);
    let rule = rule_pair_boxed(Rule::tag_core, app);
    rules.push(rule);

    let rule = rule_pair(Rule::value_tag_loc, read_value_tag_loc);
    rules.push(rule);

    let app = compose_pair(read_var_name, ast::ValueTag::Halt);
    let rule = rule_pair_unwrap(Rule::tag_halt, 1, app);
    rules.push(rule);
    expect_vec(rules, pairs, context)
}

fn read_tag(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Tag {
    let mut rules = vec![];
    let value_app = compose_pair(read_value_tag, ast::Tag::ValueTag);
    rules.push(rule_pair_boxed(Rule::value_tag, value_app));
    let timeline_app = compose_pair(read_timeline_tag, ast::Tag::TimelineTag);
    rules.push(rule_pair_boxed(Rule::timeline_tag, timeline_app));
    let spatial_app = compose_pair(read_spatial_tag, ast::Tag::SpatialTag);
    rules.push(rule_pair_boxed(Rule::spatial_tag, spatial_app));
    expect_vec(rules, pairs, context)
}

fn read_value(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Value {
    // funclet_loc | var_name | fn_name | typ | place | stage | tag
    let mut rules = Vec::new();

    let rule_function_loc = compose_pair(read_funclet_loc, ast::Value::FunctionLoc);
    rules.push(rule_pair_boxed(Rule::funclet_loc, rule_function_loc));
    let rule_fn_name = compose_pair(read_fn_name, ast::Value::FnName);
    rules.push(rule_pair_boxed(Rule::fn_name, rule_fn_name));
    let rule_type = compose_pair(read_type, ast::Value::Type);
    rules.push(rule_pair_boxed(Rule::typ, rule_type));
    let rule_place = compose_str(read_place, ast::Value::Place);
    rules.push(rule_str_boxed(Rule::place, rule_place));
    let rule_stage = compose_str(read_stage, ast::Value::Stage);
    rules.push(rule_str_boxed(Rule::stage, rule_stage));
    let rule_tag = compose_pair(read_tag, ast::Value::Tag);
    rules.push(rule_pair_boxed(Rule::tag, rule_tag));

    expect_vec(rules, pairs, context)
}

fn rule_value<'a>() -> RuleApp<'a, ast::Value> {
    rule_pair(Rule::value, read_value)
}

fn read_list_values(pairs : &mut Pairs<Rule>, context : &mut Context) -> Vec<ast::Value> {
    expect_all(rule_value(), pairs, context)
}

fn read_list(pairs : &mut Pairs<Rule>, context : &mut Context) -> Vec<ast::Value> {
    let rule = rule_pair(Rule::list_values, read_list_values);
    option_to_vec(optional(rule, pairs, context))
}

fn read_dict_value(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::DictValue {
    let mut rules = Vec::new();
    let value_map = compose_pair(read_value,ast::DictValue::Raw);
    rules.push(rule_pair_boxed(Rule::value, value_map));

    let list_map = compose_pair(read_list, ast::DictValue::List);
    rules.push(rule_pair_boxed(Rule::list, list_map));

    let dict_map = compose_pair(read_unchecked_dict, ast::DictValue::Dict);
    rules.push(rule_pair_boxed(Rule::unchecked_dict, dict_map));

    expect_vec(rules, pairs, context)
}

fn read_dict_key(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Value {
    let value_var_name = compose_pair(read_var_name, ast::Value::VarName);
    let rule_var_name = rule_pair_boxed(Rule::var_name, value_var_name);
    expect_vec(vec![rule_id(), rule_var_name], pairs, context)
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

fn rule_unchecked_dict<'a>() -> RuleApp<'a, ast::UncheckedDict> {
    rule_pair(Rule::unchecked_dict, read_unchecked_dict)
}

// Readers

fn read_version(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Version {
    require_rule(Rule::version_keyword, pairs, context);
    let major_s = expect(rule_n_raw(), pairs, context);
    let minor_s = expect(rule_n_raw(), pairs, context);
    let detailed_s = expect(rule_n_raw(), pairs, context);

    let major = major_s.parse::<u32>().unwrap();
    let minor = minor_s.parse::<u32>().unwrap();
    let detailed = detailed_s.parse::<u32>().unwrap();

    ast::Version { major, minor, detailed }
}

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
    let name_rule = rule_str_unwrap(Rule::type_name, 1, Box::new(read_string));
    let name = expect(name_rule, pairs, context);
    let data = expect(rule_unchecked_dict(), pairs, context);
    context.add_local_type(name.clone());
    let result = ast::LocalType { event, name, data };
    ast::TypeDecl::Local(result)
}

fn read_ffi_type_decl(s : String, context : &mut Context) -> ast::TypeDecl {
    context.add_ffi_type(s.to_string());
    ast::TypeDecl::FFI(read_ffi_raw(s, context))
}

fn read_type_def(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::TypeDecl {
    let ffi_rule = rule_str(Rule::ffi_type, read_ffi_type_decl);
    let ir_rule = rule_pair(Rule::ir_type_decl, read_ir_type_decl);
    expect_vec(vec![ffi_rule, ir_rule], pairs, context)
}

fn read_types(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Types {
    let rule = rule_pair(Rule::type_def,read_type_def);
    let mut result = ast::Types { ffi_types : Vec::new(), local_types : Vec::new() };
    for next in expect_all(rule, pairs, context).drain(..) {
        match next {
            ast::TypeDecl::FFI(t) => { result.ffi_types.push(t); },
            ast::TypeDecl::Local(t) => { result.local_types.push(t); }
        }
    }
    result
}

fn read_external_args(pairs : &mut Pairs<Rule>, context : &mut Context) -> Vec<ffi::Type> {
    expect_all(rule_ffi_type(), pairs, context)
}

fn read_external_cpu_funclet(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::ExternalCpuFunction {
    let output_type = expect(rule_ffi_typ_sep(), pairs, context);
    let name = expect(rule_fn_name(), pairs, context);
    let rule_extern_args = rule_pair(Rule::external_args, read_external_args);
    let input_types = expect(rule_extern_args, pairs, context);
    context.add_ffi_funclet(name.clone());
    ast::ExternalCpuFunction {
        name,
        input_types,
        output_types : vec![output_type]
    }
}

fn read_external_gpu_funclet(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::ExternalGpuFunction {
    // TODO: Don't forget to add the ffi_funclet!
    todo!()
}

fn read_external_funclet(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::FuncletDef {
    let extern_name = expect(rule_string(Rule::external_name), pairs, context);
    match extern_name.trim() {
        "external_cpu" => ast::FuncletDef::ExternalCPU(read_external_cpu_funclet(pairs, context)),
        "external_gpu" => ast::FuncletDef::ExternalGPU(read_external_gpu_funclet(pairs, context)),
        _ => panic!(unexpected(extern_name))
    }
}

fn read_funclet_arg(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Argument {
    let typ = expect(rule_type_sep(), pairs, context);
    let var = expect(rule_var_name(), pairs, context);
    ast::Argument { typ, var }
}

fn read_funclet_args(pairs : &mut Pairs<Rule>, context : &mut Context) -> Vec<ast::Argument> {
    expect_all(rule_pair(Rule::funclet_arg, read_funclet_arg), pairs, context)
}

fn read_funclet_header(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::FuncletHeader {
    let ret = expect(rule_type_sep(), pairs, context);
    let name = expect(rule_fn_name(), pairs, context);
    let rule_args = rule_pair(Rule::funclet_args, read_funclet_args);
    let args = option_to_vec(optional(rule_args, pairs, context));
    context.add_local_funclet(name.clone());
    ast::FuncletHeader { ret, name, args }
}

fn rule_funclet_header<'a>() -> RuleApp<'a, ast::FuncletHeader> {
    rule_pair(Rule::funclet_header, read_funclet_header)
}

fn read_var_assign(pairs : &mut Pairs<Rule>, context : &mut Context) {
    let var = expect(rule_var_name(), pairs, context);
    context.add_node(var);
}

fn read_phi_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Command {
    read_var_assign(pairs, context);
    let index_rule = rule_str_unwrap(Rule::phi_right, 1,Box::new(read_n));
    let index = expect(index_rule, pairs, context);
    ast::Command::IRNode(ast::Node::Phi { index })
}

fn rule_phi_command<'a>() -> RuleApp<'a, ast::Command> {
    rule_pair(Rule::phi_command, read_phi_command)
}

fn read_return_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Command {
    let var = expect(rule_var_name(), pairs, context);
    ast::Command::Return { var }
}

fn rule_return_command<'a>() -> RuleApp<'a, ast::Command> {
    rule_pair(Rule::return_command, read_return_command)
}

fn read_constant(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let rule_value = rule_str(Rule::n, read_string);
    let value_string = expect(rule_value, pairs, context);
    let value = value_string.parse::<i64>().unwrap();

    let ffi_app = compose_str(read_string, ast::Type::FFI);
    let rule_ffi = rule_str_boxed(Rule::ffi_type, ffi_app);
    let type_id = expect(rule_ffi, pairs, context);

    ast::Node::ConstantInteger { value, type_id }
}

fn read_constant_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Command {
    read_var_assign(pairs, context);
    require_rule(Rule::constant_sep, pairs, context);
    let rule_constant = rule_pair(Rule::constant, read_constant);
    let node = expect(rule_constant, pairs, context);
    ast::Command::IRNode(node)
}

fn rule_constant_command<'a>() -> RuleApp<'a, ast::Command> {
    rule_pair(Rule::constant_command, read_constant_command)
}

fn read_value_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Command {
    let mut rules = Vec::new();
    rules.push(rule_phi_command());
    rules.push(rule_return_command());
    rules.push(rule_constant_command());
    expect_vec(rules, pairs, context)
}

fn read_alloc_right(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Command {
    let place = expect(rule_place(), pairs, context);
    let storage_type = expect(rule_type(), pairs, context);
    let operation = expect(rule_funclet_loc(), pairs, context);
    ast::Command::IRNode(ast::Node::AllocTemporary {
        place,
        storage_type,
        operation,
    })
}

fn read_alloc_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Command {
    read_var_assign(pairs, context);
    let rule_alloc_right = rule_pair(Rule::alloc_right, read_alloc_right);
    expect(rule_alloc_right, pairs, context)
}

fn rule_alloc_command<'a>() -> RuleApp<'a, ast::Command> {
    rule_pair(Rule::alloc_command, read_alloc_command)
}

fn read_do_args(pairs : &mut Pairs<Rule>, context : &mut Context) -> Vec<String> {
    expect_all(rule_var_name(), pairs, context)
}

fn read_do_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Command {
    let rule_place = rule_str_unwrap(Rule::do_sep, 1, Box::new(read_place));
    let place = expect(rule_place, pairs, context);
    let operation = expect(rule_funclet_loc(), pairs, context);
    let rule_args = rule_pair(Rule::do_args, read_do_args);
    let inputs = option_to_vec(optional(rule_args, pairs, context));
    let output = expect(rule_var_name(), pairs, context);
    ast::Command::IRNode(ast::Node::EncodeDo {
        place,
        operation,
        inputs: inputs.into_boxed_slice(),
        outputs: Box::new([output]),
    })
}

fn rule_do_command<'a>() -> RuleApp<'a, ast::Command> {
    rule_pair(Rule::do_command, read_do_command)
}

fn read_schedule_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Command {
    let mut rules = Vec::new();
    rules.push(rule_phi_command());
    rules.push(rule_return_command());
    rules.push(rule_alloc_command());
    rules.push(rule_do_command());
    expect_vec(rules, pairs, context)
}

fn read_timeline_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Command {
    let mut rules = Vec::new();
    rules.push(rule_phi_command());
    rules.push(rule_return_command());
    expect_vec(rules, pairs, context)
}

fn read_funclet_blob(kind : ir::FuncletKind, rule_command : RuleApp<ast::Command>,
pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Funclet {
    let header = expect(rule_funclet_header(), pairs, context);
    let commands = expect_all(rule_command, pairs, context);
    ast::Funclet {
        kind,
        header,
        commands,
    }
}

fn read_value_funclet(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Funclet {
    let rule_command = rule_pair(Rule::value_command, read_value_command);
    read_funclet_blob(ir::FuncletKind::Value, rule_command, pairs, context)
}

fn read_schedule_funclet(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Funclet {
    let rule_command = rule_pair(Rule::schedule_command, read_schedule_command);
    read_funclet_blob(ir::FuncletKind::ScheduleExplicit, rule_command, pairs, context)
}

fn read_timeline_funclet(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Funclet {
    let rule_command = rule_pair(Rule::timeline_command, read_timeline_command);
    read_funclet_blob(ir::FuncletKind::Timeline, rule_command, pairs, context)
}

fn read_funclet(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Funclet {
    let rule = pairs.next().unwrap().as_rule();
    let vrule = rule_pair(Rule::value_funclet, read_value_funclet);
    let srule = rule_pair(Rule::schedule_funclet, read_schedule_funclet);
    let trule = rule_pair(Rule::timeline_funclet, read_timeline_funclet);
    match rule {
        Rule::value_sep => expect(vrule, pairs, context),
        Rule::schedule_sep => expect(srule, pairs, context),
        Rule::timeline_sep => expect(trule, pairs, context),
        _ => panic!(unexpected_rule(&vec![vrule, srule, trule], rule))
    }
}

fn read_funclet_def(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::FuncletDef {
    let mut rules = Vec::new();
    rules.push(rule_pair(Rule::external_funclet, read_external_funclet));
    let local_rule = compose_pair(read_funclet, ast::FuncletDef::Local);
    rules.push(rule_pair_boxed(Rule::funclet, local_rule));
    expect_vec(rules, pairs, context)
}

fn read_funclets(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::FuncletDefs {
    let rule = rule_pair(Rule::funclet_def, read_funclet_def);
    expect_all(rule, pairs, context)
}

fn read_extra(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Extra {
    let name = expect(rule_fn_name_sep(), pairs, context);
    let data = expect(rule_unchecked_dict(), pairs, context);
    ast::Extra { name, data }
}

fn read_extras(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Extras {
    expect_all(rule_pair(Rule::extra, read_extra), pairs, context)
}

fn read_pipeline(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Pipeline {
    require_rule(Rule::pipeline_sep, pairs, context);
    let name = expect(rule_string_clean(), pairs, context);
    let funclet = expect(rule_fn_name(), pairs, context);
    ast::Pipeline { name, funclet }
}

fn read_pipelines(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Pipelines {
    expect_all(rule_pair(Rule::pipeline, read_pipeline), pairs, context)
}

fn read_program(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Program {

    let version = expect(rule_pair(Rule::version, read_version),
                         pairs, context);

    context.initiate_type_indices();
    let types = expect(
        rule_pair(Rule::types, read_types), pairs, context);


    context.initiate_funclet_indices();
    let funclets = expect(
        rule_pair(Rule::funclets, read_funclets), pairs, context);


    context.clear_indices();
    let extras = expect(
        rule_pair(Rule::extras, read_extras), pairs, context);

    let pipelines = expect(
        rule_pair(Rule::pipelines, read_pipelines), pairs, context);

    ast::Program {
        version,
        types,
        funclets,
        extras,
        pipelines,
    }
}

fn read_definition(pairs : &mut Pairs<Rule>, context : &mut Context) -> frontend::Definition {
    let program = expect(
        rule_pair(Rule::program, read_program), pairs, context);

    ast_to_ir(program, context)
}

pub fn parse(code : &str) ->
Result<frontend::Definition, frontend::CompileError> {
    let parsed = IRParser::parse(Rule::program, code);
    let mut context = new_context();
    match parsed {
        Err(why) => Err(crate::frontend::CompileError{ message: (format!("{}", why)) }),
        Ok(mut parse_result) => Ok(read_definition(&mut parse_result, &mut context))
    }
}
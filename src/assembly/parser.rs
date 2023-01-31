
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
use crate::ast;
use crate::assembly::ast_to_ir;
use crate::assembly::context::{new_context, Context};
use crate::ast::UncheckedDict;

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

fn read_ffi_type_base(s : String, context : &mut Context) -> ast::FFIType {
    match s.as_str() {
        "f32" => { ast::FFIType::F32 },
        "f64" => { ast::FFIType::F64 },
        "u8" => { ast::FFIType::U8 },
        "u16" => { ast::FFIType::U16 },
        "u32" => { ast::FFIType::U32 },
        "u64" => { ast::FFIType::U64 },
        "i8" => { ast::FFIType::I8 },
        "i16" => { ast::FFIType::I16 },
        "i32" => { ast::FFIType::I32 },
        "i64" => { ast::FFIType::I64 },
        "usize" => { ast::FFIType::USize },
        "gpu_buffer_allocator" => { ast::FFIType::GpuBufferAllocator },
        _ => panic!("Unknown type name {}", s)
    }
}

fn read_ffi_ref_parameter(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::FFIType {
    expect(rule_ffi_type(), pairs, context)
}

fn read_ffi_array_params(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::FFIType {
    let element_type = Box::new(expect(rule_ffi_type(), pairs, context));
    let length = expect(rule_n(), pairs, context);
    ast::FFIType::Array { element_type, length }
}

fn read_ffi_tuple_params(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::FFIType {
    let elements = expect_all(rule_ffi_type(), pairs, context);
    ast::FFIType::Tuple(elements)
}

fn read_ffi_parameterized_ref_name(s : String, context : &mut Context)
-> Box<dyn Fn(ast::FFIType) -> ast::FFIType> {
    fn box_up<F>(f : &'static F) -> Box<dyn Fn(ast::FFIType) -> ast::FFIType>
        where
        F: Fn(Box<ast::FFIType>) -> ast::FFIType,
    {
        Box::new(move |x| f(Box::new(x)))
    }
    match s.as_str() {
        "erased_length_array" => box_up(&ast::FFIType::ErasedLengthArray),
        "const_ref" => box_up(&ast::FFIType::ConstRef),
        "mut_ref" => box_up(&ast::FFIType::MutRef),
        "const_slice" => box_up(&ast::FFIType::ConstSlice),
        "mut_slice" => box_up(&ast::FFIType::MutSlice),
        "gpu_buffer_ref" => box_up(&ast::FFIType::GpuBufferRef),
        "gpu_buffer_slice" => box_up(&ast::FFIType::GpuBufferSlice),
        _ => panic!("Unknown type name {}", s)
    }
}

fn read_ffi_parameterized_ref(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::FFIType {
    let rule = rule_str(Rule::ffi_parameterized_ref_name,
                        read_ffi_parameterized_ref_name);
    let kind = expect(rule, pairs, context);
    let rule = rule_pair(Rule::ffi_ref_parameter, read_ffi_ref_parameter);
    let value = expect(rule, pairs, context);
    kind(value)
}

fn read_ffi_parameterized_type(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::FFIType {
    let mut rules = Vec::new();
    let func = Box::new(read_ffi_array_params);
    rules.push(rule_pair_unwrap(Rule::ffi_parameterized_array, 1, func));

    rules.push(rule_pair(Rule::ffi_parameterized_ref, read_ffi_parameterized_ref));
    let func = Box::new(read_ffi_tuple_params);
    rules.push(rule_pair_unwrap(Rule::ffi_parameterized_tuple, 1, func));

    expect_vec(rules, pairs, context)
}

fn read_ffi_type(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::FFIType {
    let mut rules = Vec::new();
    rules.push(rule_str(Rule::ffi_type_base, read_ffi_type_base));
    rules.push(rule_pair(Rule::ffi_parameterized_type, read_ffi_parameterized_type));
    expect_vec(rules, pairs, context)
}

fn rule_ffi_type<'a>() -> RuleApp<'a, ast::FFIType> {
    rule_pair(Rule::ffi_type, read_ffi_type)
}

fn rule_ffi_type_sep<'a>() -> RuleApp<'a, ast::FFIType> {
    rule_pair_unwrap(Rule::ffi_type_sep, 1, Box::new(read_ffi_type))
}

fn read_type(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Type {
    let mut rules = Vec::new();
    let ffi_fn = compose_pair(read_ffi_type, ast::Type::FFI);
    rules.push(rule_pair_boxed(Rule::ffi_type, ffi_fn));
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
    let rule_func = rule_str_unwrap(Rule::fn_name, 1,Box::new(read_string));
    let rule_var = rule_str_unwrap(Rule::var_name, 1,Box::new(read_string));
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

fn read_value_tag_data(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::ValueTag {
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

fn read_value_tag(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::ValueTag {
    require_rule(Rule::value_tag_sep, pairs, context);
    let rule = rule_pair(Rule::value_tag_data, read_value_tag_data);
    expect(rule, pairs, context)
}

fn rule_value_tag<'a>() -> RuleApp<'a, ast::ValueTag> {
    rule_pair(Rule::value_tag, read_value_tag)
}

fn read_timeline_tag_data(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::TimelineTag {
    ast::TimelineTag::Core(expect(rule_tag_core(), pairs, context))
}

fn read_timeline_tag(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::TimelineTag {
    require_rule(Rule::timeline_tag_sep, pairs, context);
    let rule = rule_pair(Rule::timeline_tag_data, read_timeline_tag_data);
    expect(rule, pairs, context)
}

fn rule_timeline_tag<'a>() -> RuleApp<'a, ast::TimelineTag> {
    rule_pair(Rule::timeline_tag, read_timeline_tag)
}

fn read_spatial_tag_data(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::SpatialTag {
    ast::SpatialTag::Core(expect(rule_tag_core(), pairs, context))
}

fn read_spatial_tag(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::SpatialTag {
    require_rule(Rule::spatial_tag_sep, pairs, context);
    let rule = rule_pair(Rule::spatial_tag_data, read_spatial_tag_data);
    expect(rule, pairs, context)
}

fn rule_spatial_tag<'a>() -> RuleApp<'a, ast::SpatialTag> {
    rule_pair(Rule::spatial_tag, read_spatial_tag)
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

fn read_slot_info(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::SlotInfo {
    let mut rules = vec![];
    rules.push(rule_pair(Rule::tag, read_tag));
    let tags = expect_all_vec(rules, pairs, context);
    let mut value_tag = ast::ValueTag::Core(ast::TagCore::None);
    let mut timeline_tag = ast::TimelineTag::Core(ast::TagCore::None);
    let mut spatial_tag = ast::SpatialTag::Core(ast::TagCore::None);
    for tag in tags.iter() {
        match tag { // duplicates are whatever
            ast::Tag::ValueTag(t) => { value_tag = t.clone() }
            ast::Tag::TimelineTag(t) => { timeline_tag = t.clone() }
            ast::Tag::SpatialTag(t) => { spatial_tag = t.clone() }
        }
    }
    ast::SlotInfo { value_tag, timeline_tag, spatial_tag }
}

fn read_fence_info(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::FenceInfo {
    let rule = rule_pair(Rule::timeline_tag, read_timeline_tag);
    match pairs.peek() {
        None => ast::FenceInfo { timeline_tag : ast::TimelineTag::Core(ast::TagCore::None) },
        Some(_) => ast::FenceInfo { timeline_tag : expect(rule, pairs, context) }
    }

}

fn read_buffer_info(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::BufferInfo {
    let rule = rule_pair(Rule::spatial_tag, read_spatial_tag);
    match pairs.peek() {
        None => ast::BufferInfo { spatial_tag: expect(rule, pairs, context) },
        Some(_) => ast::BufferInfo { spatial_tag: expect(rule, pairs, context) }
    }
}

fn read_value(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Value {
    // funclet_loc | var_name | fn_name | typ | place | stage | tag
    let mut rules = Vec::new();

    let rule = compose_str(read_n, ast::Value::Num);
    rules.push(rule_str_boxed(Rule::n, rule));
    let rule = compose_pair(read_fn_name, ast::Value::VarName);
    rules.push(rule_pair_boxed(Rule::var_name, rule));
    let rule = compose_pair(read_funclet_loc, ast::Value::FunctionLoc);
    rules.push(rule_pair_boxed(Rule::funclet_loc, rule));
    let rule = compose_pair(read_fn_name, ast::Value::FnName);
    rules.push(rule_pair_boxed(Rule::fn_name, rule));
    let rule = compose_pair(read_type, ast::Value::Type);
    rules.push(rule_pair_boxed(Rule::typ, rule));
    let rule = compose_str(read_place, ast::Value::Place);
    rules.push(rule_str_boxed(Rule::place, rule));
    let rule = compose_str(read_stage, ast::Value::Stage);
    rules.push(rule_str_boxed(Rule::stage, rule));
    let rule = compose_pair(read_tag, ast::Value::Tag);
    rules.push(rule_pair_boxed(Rule::tag, rule));
    let rule = compose_pair(read_slot_info, ast::Value::SlotInfo);
    rules.push(rule_pair_boxed(Rule::slot_info, rule));
    let rule = compose_pair(read_fence_info, ast::Value::FenceInfo);
    rules.push(rule_pair_boxed(Rule::fence_info, rule));
    let rule = compose_pair(read_buffer_info, ast::Value::BufferInfo);
    rules.push(rule_pair_boxed(Rule::buffer_info, rule));

    expect_vec(rules, pairs, context)
}

fn rule_value<'a>() -> RuleApp<'a, ast::Value> {
    rule_pair(Rule::value, read_value)
}

fn read_list_values(pairs : &mut Pairs<Rule>, context : &mut Context) -> Vec<ast::DictValue> {
    let rule = rule_pair(Rule::dict_value, read_dict_value);
    expect_all(rule, pairs, context)
}

fn read_list(pairs : &mut Pairs<Rule>, context : &mut Context) -> Vec<ast::DictValue> {
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

struct DictPair {
    key: ast::Value,
    value: ast::DictValue
}

fn read_dict_element(pairs : &mut Pairs<Rule>, context : &mut Context) -> DictPair {
    let rule_key = rule_pair(Rule::dict_key, read_dict_key);
    let rule_value = rule_pair(Rule::dict_value, read_dict_value);
    let key = expect(rule_key, pairs, context);
    let value = expect(rule_value, pairs, context);
    DictPair { key, value }
}

fn read_unchecked_dict(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::UncheckedDict {
    let rule = rule_pair(Rule::dict_element, read_dict_element);
    let mut result = HashMap::new();
    for pair in expect_all(rule, pairs, context) {
        result.insert(pair.key, pair.value);
    }
    result
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

fn is_event(s : String, context : &mut Context) -> bool {
    match s.as_str() {
        "event" => true,
        "slot" => false,
        _ => panic!(format!("Unexpected slot check {}", s))
    }
}

fn read_ir_type_decl(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::TypeDecl {
    let event_rule = rule_str_unwrap(
        Rule::type_decl_sep, 1, Box::new(is_event));
    let event = expect(event_rule, pairs, context);
    let name_rule = rule_str_unwrap(Rule::type_name, 1, Box::new(read_string));
    let name = expect(name_rule, pairs, context);
    let data = expect(rule_unchecked_dict(), pairs, context);
    context.add_local_type(name.clone());
    let result = ast::LocalType { event, name, data };
    ast::TypeDecl::Local(result)
}

fn read_ffi_type_decl(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::TypeDecl {
    let ffi_typ = expect(rule_ffi_type(), pairs, context);
    context.add_ffi_type(ffi_typ.clone());
    ast::TypeDecl::FFI(ffi_typ)
}

fn read_type_def(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::TypeDecl {
    let mut rules = Vec::new();
    rules.push(rule_pair(Rule::ffi_type_decl, read_ffi_type_decl));
    rules.push(rule_pair(Rule::ir_type_decl, read_ir_type_decl));
    expect_vec(rules, pairs, context)
}

fn read_types(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Types {
    let rule = rule_pair(Rule::type_def,read_type_def);
    expect_all(rule, pairs, context)
}

fn read_external_cpu_args(pairs : &mut Pairs<Rule>, context : &mut Context) -> Vec<ast::FFIType> {
    expect_all(rule_ffi_type(), pairs, context)
}

fn read_external_cpu_funclet(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::ExternalCpuFunction {
    require_rule(Rule::external_cpu_sep, pairs, context);

    let output_types = vec![expect(rule_ffi_type_sep(), pairs, context)];
    let name = expect(rule_fn_name(), pairs, context);

    let rule_extern_args = rule_pair(Rule::external_cpu_args, read_external_cpu_args);
    let input_types = expect(rule_extern_args, pairs, context);
    context.add_cpu_funclet(name.clone());
    ast::ExternalCpuFunction {
        name,
        input_types,
        output_types
    }
}

fn read_external_gpu_arg(pairs : &mut Pairs<Rule>, context : &mut Context) -> (ast::FFIType, String) {
    let typ = expect(rule_ffi_type_sep(), pairs, context);
    let name = expect(rule_var_name(), pairs, context);
    (typ, name)
}

fn read_external_gpu_args(pairs : &mut Pairs<Rule>, context : &mut Context) -> Vec<(ast::FFIType, String)> {
    let rule = rule_pair(Rule::external_gpu_arg, read_external_gpu_arg);
    expect_all(rule, pairs, context)
}

fn read_external_gpu_body(pairs : &mut Pairs<Rule>, context : &mut Context) -> Vec<UncheckedDict> {
    let rule = rule_pair_unwrap(Rule::external_gpu_resource, 1,
                                Box::new(read_unchecked_dict));
    expect_all(rule, pairs, context)
}

fn read_external_gpu_funclet(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::ExternalGpuFunction {
    require_rule(Rule::external_gpu_sep, pairs, context);

    let output_types = vec![expect(rule_ffi_type_sep(), pairs, context)];

    let name = expect(rule_fn_name(), pairs, context);

    let rule_extern_args = rule_pair(Rule::external_gpu_args, read_external_gpu_args);
    let input_args = expect(rule_extern_args, pairs, context);

    let shader_module = expect(rule_string_clean(), pairs, context);

    let rule_binding = rule_pair(Rule::external_gpu_body, read_external_gpu_body);
    let resource_bindings = expect(rule_binding, pairs, context);

    context.add_gpu_funclet(name.clone());
    ast::ExternalGpuFunction {
        name,
        input_args,
        output_types,
        shader_module,
        entry_point: "main".to_string(), // todo: uhhhh, allow syntax perhaps
        resource_bindings
    }
}

fn read_external_funclet(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::FuncletDef {
    let mut rules = Vec::new();
    let comp = compose_pair(read_external_cpu_funclet, ast::FuncletDef::ExternalCPU);
    rules.push(rule_pair_boxed(Rule::external_cpu, comp));
    let comp = compose_pair(read_external_gpu_funclet, ast::FuncletDef::ExternalGPU);
    rules.push(rule_pair_boxed(Rule::external_gpu, comp));
    expect_vec(rules, pairs, context)
}

fn read_funclet_args(pairs : &mut Pairs<Rule>, context : &mut Context) -> Vec<ast::Type> {
    expect_all(rule_type(), pairs, context)
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

fn read_var_assign(pairs : &mut Pairs<Rule>, context : &mut Context) -> String {
    let var = expect(rule_var_name(), pairs, context);
    context.add_node(var.clone());
    var
}

fn read_return_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::TailEdge {
    let var = expect(rule_var_name(), pairs, context);
    ast::TailEdge::Return { var }
}

fn rule_return_command<'a>() -> RuleApp<'a, ast::TailEdge> {
    rule_pair(Rule::return_command, read_return_command)
}

fn read_tail_edge(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::TailEdge {
    let mut rules = Vec::new();
    rules.push(rule_return_command());
    expect_vec(rules, pairs, context)
}

fn read_phi_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let index = expect(rule_n(), pairs, context);
    ast::Node::Phi { index }
}

fn rule_phi_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::phi_command, read_phi_command)
}

fn read_constant(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let rule_value = rule_str(Rule::n, read_string);
    let value_string = expect(rule_value, pairs, context);
    let value = value_string.parse::<i64>().unwrap(); // BAD

    // let ffi_app = compose_pair(read_ffi_typ, ast::Type::FFI);
    // let rule_ffi = rule_pair_boxed(Rule::ffi_type, ffi_app);
    // let type_id = expect(rule_ffi, pairs, context);

    let check_id = expect(rule_ffi_type(), pairs, context);

    if check_id != ast::FFIType::I64 {
        panic!("Constant can only be i64 for now");
    }

    let type_id = ast::Type::FFI(ast::FFIType::I64);

    ast::Node::ConstantInteger { value, type_id }
}

fn read_constant_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let rule = rule_pair(Rule::constant, read_constant);
    expect(rule, pairs, context)
}

fn rule_constant_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::constant_command, read_constant_command)
}

fn read_constant_unsigned(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
     // todo: remove this function
    let rule_value = rule_str(Rule::n, read_string);
    let value_string = expect(rule_value, pairs, context);
    let value = value_string.parse::<u64>().unwrap(); // BAD

    let check_id = expect(rule_ffi_type(), pairs, context);

    if check_id != ast::FFIType::U64 {
        panic!("Unsigned constant can only be u64 for now");
    }

    let type_id = ast::Type::FFI(ast::FFIType::U64);

    ast::Node::ConstantUnsignedInteger { value, type_id }
}

fn read_constant_unsigned_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let rule = rule_pair(Rule::constant, read_constant_unsigned);
    expect(rule, pairs, context)
}

fn rule_constant_unsigned_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::constant_unsigned_command, read_constant_unsigned_command)
}

fn read_extract_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    require_rule(Rule::extract_sep, pairs, context);
    let node_id = expect(rule_var_name(), pairs, context);
    let index = expect(rule_n(), pairs, context);
    ast::Node::ExtractResult { node_id, index }
}

fn rule_extract_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::extract_command, read_extract_command)
}

fn read_call_args(pairs : &mut Pairs<Rule>, context : &mut Context) -> Vec<String> {
    expect_all(rule_var_name(), pairs, context)
}

fn read_call_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    require_rule(Rule::call_sep, pairs, context);
    let external_function_id = expect(rule_fn_name(), pairs, context);
    let rule = rule_pair(Rule::call_args, read_call_args);
    let args = expect(rule, pairs, context).into_boxed_slice();
    match pairs.peek() {
        None => {
            // NOTE: semi-arbitrary choice for unification
            ast::Node::CallExternalCpu { external_function_id, arguments : args }
        }
        Some(_) => {
            let rule = rule_pair(Rule::call_args, read_call_args);
            let arguments = expect(rule, pairs, context).into_boxed_slice();
            ast::Node::CallExternalGpuCompute { external_function_id, arguments, dimensions : args }
        }
    }
}

fn rule_call_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::call_command, read_call_command)
}

fn read_select_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    require_rule(Rule::select_sep, pairs, context);
    let condition = expect(rule_var_name(), pairs, context);
    let true_case = expect(rule_var_name(), pairs, context);
    let false_case = expect(rule_var_name(), pairs, context);
    ast::Node::Select { condition, true_case, false_case }
}

fn rule_select_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::select_command, read_select_command)
}

fn read_value_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let mut rules = Vec::new();
    rules.push(rule_phi_command());
    rules.push(rule_constant_command());
    rules.push(rule_constant_unsigned_command());
    rules.push(rule_extract_command());
    rules.push(rule_call_command());
    rules.push(rule_select_command());
    expect_vec(rules, pairs, context)
}

fn read_value_assign(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let name = expect(rule_var_name(), pairs, context);
    context.add_node(name);
    let rule = rule_pair(Rule::value_command, read_value_command);
    expect(rule, pairs, context)
}

fn read_alloc_temporary_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let place = expect(rule_place(), pairs, context);
    let storage_type = expect(rule_type(), pairs, context);
    let operation = expect(rule_funclet_loc(), pairs, context);
    ast::Node::AllocTemporary { place, storage_type, operation }
}

fn rule_alloc_temporary_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::alloc_temporary_command, read_alloc_temporary_command)
}

fn read_do_args(pairs : &mut Pairs<Rule>, context : &mut Context) -> Vec<String> {
    expect_all(rule_var_name(), pairs, context)
}

fn read_do_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let rule_place = rule_str_unwrap(Rule::do_sep, 1, Box::new(read_place));
    let place = expect(rule_place, pairs, context);
    let operation = expect(rule_funclet_loc(), pairs, context);
    let rule_args = rule_pair(Rule::do_args, read_do_args);
    let inputs = option_to_vec(optional(rule_args, pairs, context));
    let output = expect(rule_var_name(), pairs, context);
    ast::Node::EncodeDo {
        place,
        operation,
        inputs: inputs.into_boxed_slice(),
        outputs: Box::new([output]),
    }
}

fn rule_do_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::do_command, read_do_command)
}

fn read_create_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let place = expect(rule_place(), pairs, context);
    let storage_type = expect(rule_type(), pairs, context);
    let operation = expect(rule_funclet_loc(), pairs, context);
    ast::Node::UnboundSlot { place, storage_type, operation }
}

fn rule_create_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::create_command, read_create_command)
}

fn read_drop_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let node = expect(rule_var_name(), pairs, context);
    ast::Node::Drop { node }
}

fn rule_drop_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::drop_command, read_drop_command)
}

fn read_alloc_sep(pairs : &mut Pairs<Rule>, context : &mut Context) -> (ir::Place, ast::Type) {
    let place = expect(rule_place(), pairs, context);
    let storage_type = expect(rule_type(), pairs, context);
    (place, storage_type)
}

fn read_alloc_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let rule = rule_pair(Rule::alloc_sep, read_alloc_sep);
    let (place, storage_type) = expect(rule, pairs, context);
    let buffer = expect(rule_var_name(), pairs, context);
    let operation = expect(rule_funclet_loc(), pairs, context);
    ast::Node::StaticAllocFromStaticBuffer { buffer, place, storage_type, operation }
}

fn rule_alloc_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::alloc_command, read_alloc_command)
}

fn read_encode_copy_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let rule = rule_str_unwrap(Rule::encode_copy_sep, 1, Box::new(read_place));
    let place = expect(rule, pairs, context);
    let input = expect(rule_var_name(), pairs, context);
    let output = expect(rule_var_name(), pairs, context);
    ast::Node::EncodeCopy { place, input, output }
}

fn rule_encode_copy_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::encode_copy_command, read_encode_copy_command)
}

fn read_encode_fence_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let place = expect(rule_place(), pairs, context);
    let event = expect(rule_funclet_loc(), pairs, context);
    ast::Node::EncodeFence { place, event }
}

fn rule_encode_fence_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::encode_fence_command, read_encode_fence_command)
}

fn read_submit_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let place = expect(rule_place(), pairs, context);
    let event = expect(rule_funclet_loc(), pairs, context);
    ast::Node::Submit { place, event }
}

fn rule_submit_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::submit_command, read_submit_command)
}

fn read_sync_fence_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let rule = rule_str_unwrap(Rule::sync_fence_sep, 1, Box::new(read_place));
    let place = expect(rule, pairs, context);
    let fence = expect(rule_var_name(), pairs, context);
    let event = expect(rule_funclet_loc(), pairs, context);
    ast::Node::SyncFence { place, fence, event }
}

fn rule_sync_fence_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::sync_fence_command, read_sync_fence_command)
}

fn read_inline_join_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    require_rule(Rule::inline_join_sep, pairs, context);
    let funclet = expect(rule_fn_name(), pairs, context);
    let continuation = expect(rule_var_name(), pairs, context);
    // empty captures re conversation
    ast::Node::InlineJoin { funclet, captures: Box::new([]), continuation }
}

fn rule_inline_join_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::inline_join_command, read_inline_join_command)
}

fn read_serialized_join_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    require_rule(Rule::serialized_join_sep, pairs, context);
    let funclet = expect(rule_fn_name(), pairs, context);
    let continuation = expect(rule_var_name(), pairs, context);
    ast::Node::InlineJoin { funclet, captures: Box::new([]), continuation }
}

fn rule_serialized_join_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::serialized_join_command, read_serialized_join_command)
}

fn read_default_join_command(s : String, context : &mut Context) -> ast::Node {
    ast::Node::DefaultJoin
}

fn rule_default_join_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_str(Rule::default_join_command, read_default_join_command)
}

fn read_schedule_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let mut rules = Vec::new();
    rules.push(rule_phi_command());
    rules.push(rule_alloc_command());
    rules.push(rule_do_command());
    rules.push(rule_create_command());
    rules.push(rule_drop_command());
    rules.push(rule_alloc_temporary_command());
    rules.push(rule_encode_copy_command());
    rules.push(rule_encode_fence_command());
    rules.push(rule_submit_command());
    rules.push(rule_sync_fence_command());
    rules.push(rule_inline_join_command());
    rules.push(rule_serialized_join_command());
    rules.push(rule_default_join_command());
    expect_vec(rules, pairs, context)
}

fn read_schedule_assign(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let name = expect(rule_var_name(), pairs, context);
    context.add_node(name);
    let rule = rule_pair(Rule::schedule_command, read_schedule_command);
    expect(rule, pairs, context)
}

fn read_sync_sep(pairs : &mut Pairs<Rule>, context : &mut Context) -> (ir::Place, ir::Place) {
    let place1 = expect(rule_place(), pairs, context);
    let place2 = expect(rule_place(), pairs, context);
    (place1, place2)
}

fn read_sync_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let rule = rule_pair(Rule::sync_sep, read_sync_sep);
    let (here_place, there_place) = expect(rule, pairs, context);
    let local_past = expect(rule_var_name(), pairs, context);
    let remote_local_past = expect(rule_var_name(), pairs, context);
    ast::Node::SynchronizationEvent {
        here_place,
        there_place,
        local_past,
        remote_local_past
    }
}

fn rule_sync_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::sync_command, read_sync_command)
}

fn read_submission_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let here_place= expect(rule_place(), pairs, context);
    let there_place = expect(rule_place(), pairs, context);
    let local_past = expect(rule_var_name(), pairs, context);
    ast::Node::SubmissionEvent {
        here_place,
        there_place,
        local_past,
    }
}

fn rule_submission_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::submission_command, read_submission_command)
}

fn read_timeline_command(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let mut rules = Vec::new();
    rules.push(rule_phi_command());
    rules.push(rule_sync_command());
    rules.push(rule_submission_command());
    expect_vec(rules, pairs, context)
}

fn read_timeline_assign(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Node {
    let name = expect(rule_var_name(), pairs, context);
    context.add_node(name);
    let rule = rule_pair(Rule::timeline_command, read_timeline_command);
    expect(rule, pairs, context)
}

fn read_funclet_blob(kind : ir::FuncletKind, rule_command : RuleApp<ast::Node>,
pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Funclet {
    let header = expect(rule_funclet_header(), pairs, context);
    let mut commands = Vec::new();
    for pair in pairs {
        let rule = pair.as_rule();
        if rule == Rule::tail_edge {
            let tail_edge = read_tail_edge(&mut pair.into_inner(), context);
            return ast::Funclet {
                kind,
                header,
                commands,
                tail_edge
            };
        }
        else if rule == rule_command.rule {
            commands.push(match &rule_command.application {
                Application::P(f) => f(&mut pair.into_inner(), context),
                _ => panic!("Internal error with rules")
            });
        }
        else {
            panic!(unexpected_rule(&vec![rule_command], rule));
        }
    }
    panic!(format!("No tail edge found for funclet {}", context.funclet_name()))
}

fn read_value_funclet(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Funclet {
    let rule_command = rule_pair(Rule::value_assign, read_value_assign);
    read_funclet_blob(ir::FuncletKind::Value, rule_command, pairs, context)
}

fn read_schedule_funclet(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Funclet {
    let rule_command = rule_pair(Rule::schedule_assign, read_schedule_assign);
    read_funclet_blob(ir::FuncletKind::ScheduleExplicit, rule_command, pairs, context)
}

fn read_timeline_funclet(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::Funclet {
    let rule_command = rule_pair(Rule::timeline_assign, read_timeline_assign);
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
    let rule = compose_pair(read_funclet, ast::FuncletDef::Local);
    rules.push(rule_pair_boxed(Rule::funclet, rule));
    let rule = compose_pair(read_value_function, ast::FuncletDef::ValueFunction);
    rules.push(rule_pair_boxed(Rule::value_function, rule));
    expect_vec(rules, pairs, context)
}

fn read_value_function_funclets(pairs : &mut Pairs<Rule>, context : &mut Context) -> Vec<String> {
    expect_all(rule_fn_name(), pairs, context)
}

fn read_value_function(pairs : &mut Pairs<Rule>, context : &mut Context) -> ast::ValueFunction {
    require_rule(Rule::value_function_sep, pairs, context);
    let output_types = vec![expect(rule_type_sep(), pairs, context)];
    let name = expect(rule_fn_name(), pairs, context);

    let rule = rule_pair(Rule::value_function_args, read_funclet_args);
    let input_types = expect(rule, pairs, context);

    let rule = rule_pair(Rule::value_function_funclets, read_value_function_funclets);
    let allowed_funclets = expect(rule, pairs, context);

    context.add_value_function(name.clone());

    ast::ValueFunction { name, input_types, output_types, allowed_funclets } // todo add syntax
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

    ast_to_ir::transform(program, context)
    // dbg!(ast_to_ir::transform(program, context));
    // todo!()
}

pub fn parse(code : &str) ->
Result<frontend::Definition, frontend::CompileError> {
    std::env::set_var("RUST_BACKTRACE", "1"); // help with debugging I guess
    let parsed = IRParser::parse(Rule::program, code);
    let mut context = new_context();
    let result = match parsed {
        Err(why) => Err(crate::frontend::CompileError{ message: (format!("{}", why)) }),
        Ok(mut parse_result) => Ok(read_definition(&mut parse_result, &mut context))
    };
    std::env::set_var("RUST_BACKTRACE", "0"); // help with debugging I guess
    result
}
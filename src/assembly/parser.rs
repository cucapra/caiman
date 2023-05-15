use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "src/assembly/caimanir.pest"]
pub struct IRParser;

use crate::assembly::ast::Hole;
use crate::assembly::ast::{
    ExternalFunctionId, FFIType, FuncletId, OperationId, RemoteNodeId, StorageTypeId, TypeId,
    ValueFunctionId,
};
use crate::ir::ffi;
use crate::{assembly, frontend, ir};

// Meta stuff

// Why this doesn't work in general is a bit of a mystery to me tbh, but here we are
// fn compose<'a, T, U, V, W, G, F>(f: F, g: G) -> Box<dyn Fn(T, U) -> W + 'a>
//     where
//         F: Fn(T, U) -> V + 'a,
//         G: Fn(V) -> W + 'a,
// {
//     Box::new(move |p, c| g(f(p, c)))
// }

#[derive(Debug)]
struct Context {
    // keeping this here in case we need it later
}

impl Context {
    pub fn new() -> Context {
        Context {}
    }
}

fn compose_pair<'a, T, U, G, F>(f: F, g: G) -> Box<dyn Fn(&mut Pairs<Rule>, &mut Context) -> U + 'a>
where
    F: Fn(&mut Pairs<Rule>, &mut Context) -> T + 'a,
    G: Fn(T) -> U + 'a,
{
    Box::new(move |p, c| g(f(p, c)))
}

fn compose_str<'a, T, U, G, F>(f: F, g: G) -> Box<dyn Fn(String, &mut Context) -> U + 'a>
where
    F: Fn(String, &mut Context) -> T + 'a,
    G: Fn(T) -> U + 'a,
{
    Box::new(move |s, c| g(f(s, c)))
}

fn option_to_vec<T>(o: Option<Vec<T>>) -> Vec<T> {
    match o {
        None => Vec::new(),
        Some(v) => v,
    }
}

fn reject_hole<T>(h: Hole<T>) -> T {
    match h {
        Some(v) => v,
        None => panic!("Invalid hole"),
    }
}

fn compose_pair_reject<'a, T, U, G, F>(
    f: F,
    g: G,
) -> Box<dyn Fn(&mut Pairs<Rule>, &mut Context) -> U + 'a>
where
    F: Fn(&mut Pairs<Rule>, &mut Context) -> Hole<T> + 'a,
    G: Fn(T) -> U + 'a,
{
    Box::new(move |p, c| g(reject_hole(f(p, c))))
}

fn compose_str_reject<'a, T, U, G, F>(f: F, g: G) -> Box<dyn Fn(String, &mut Context) -> U + 'a>
where
    F: Fn(String, &mut Context) -> Hole<T> + 'a,
    G: Fn(T) -> U + 'a,
{
    Box::new(move |s, c| g(reject_hole(f(s, c))))
}

fn unchecked_value(
    s: &assembly::ast::Value,
    data: &assembly::ast::UncheckedDict,
) -> assembly::ast::Value {
    let err = format!("Expected an entry for for {:?}", s);
    match data.get(s).unwrap_or_else(|| panic!(err.clone())) {
        assembly::ast::DictValue::Raw(v) => v.clone(),
        _ => panic!(err),
    }
}

// Rule stuff

fn unexpected(value: String) -> String {
    format!("Unexpected string {}", value)
}

fn unexpected_rule<T>(potentials: &Vec<RuleApp<T>>, rule: Rule) -> String {
    format!(
        "Expected rule {:?}, got {:?}",
        rule_app_vec_as_str(potentials),
        rule
    )
}

fn unexpected_rule_raw(potentials: Vec<Rule>, rule: Rule) -> String {
    format!("Expected rule {:?}, got {:?}", potentials, rule)
}

enum Application<'a, T> {
    P(Box<dyn Fn(&mut Pairs<Rule>, &mut Context) -> T + 'a>),
    S(Box<dyn Fn(String, &mut Context) -> T + 'a>),
}

struct RuleApp<'a, T> {
    rule: Rule,
    unwrap: usize,
    application: Application<'a, T>,
}

fn rule_app_as_str<T>(rule: &RuleApp<T>) -> String {
    return format!("{:?} {:?}", rule.rule, rule.unwrap);
}

fn rule_app_vec_as_str<T>(rules: &Vec<RuleApp<T>>) -> String {
    let mut result = Vec::new();
    for rule in rules.iter() {
        result.push(rule_app_as_str(rule));
    }
    format!("{:?}", result)
}

fn rule_pair_unwrap<'a, T>(
    rule: Rule,
    unwrap: usize,
    apply: Box<dyn Fn(&mut Pairs<Rule>, &mut Context) -> T + 'a>,
) -> RuleApp<'a, T> {
    let application = Application::P(apply);
    RuleApp {
        rule,
        unwrap,
        application,
    }
}

fn rule_pair_boxed<'a, T>(
    rule: Rule,
    apply: Box<dyn Fn(&mut Pairs<Rule>, &mut Context) -> T + 'a>,
) -> RuleApp<'a, T> {
    rule_pair_unwrap(rule, 0, apply)
}

fn rule_pair<'a, T: 'a>(
    rule: Rule,
    apply: fn(&mut Pairs<Rule>, &mut Context) -> T,
) -> RuleApp<'a, T> {
    rule_pair_unwrap(rule, 0, Box::new(apply))
}

fn rule_str_unwrap<'a, T>(
    rule: Rule,
    unwrap: usize,
    apply: Box<dyn Fn(String, &mut Context) -> T + 'a>,
) -> RuleApp<'a, T> {
    let application = Application::S(apply);
    RuleApp {
        rule,
        unwrap,
        application,
    }
}

fn rule_str_boxed<'a, T>(
    rule: Rule,
    apply: Box<dyn Fn(String, &mut Context) -> T + 'a>,
) -> RuleApp<'a, T> {
    rule_str_unwrap(rule, 0, apply)
}

fn rule_str<'a, T: 'a>(rule: Rule, apply: fn(String, &mut Context) -> T) -> RuleApp<'a, T> {
    rule_str_unwrap(rule, 0, Box::new(apply))
}

fn check_rule(potentials: Vec<Rule>, rule: Rule, context: &mut Context) -> bool {
    for potential in potentials {
        if rule == potential {
            return true;
        }
    }
    false
}

fn is_rule(potentials: Vec<Rule>, pairs: &mut Pairs<Rule>, context: &mut Context) -> bool {
    match pairs.peek() {
        None => false,
        Some(pair) => check_rule(potentials, pair.as_rule(), context),
    }
}

fn require_rules(potentials: Vec<Rule>, pairs: &mut Pairs<Rule>, context: &mut Context) {
    let rule = pairs.next().unwrap().as_rule();
    if !check_rule(potentials, rule, context) {
        panic!("Unexpected parse rule {:?}", rule)
    }
}

fn require_rule(potential: Rule, pairs: &mut Pairs<Rule>, context: &mut Context) {
    require_rules(vec![potential], pairs, context)
}

fn apply_pair<T>(
    potentials: &Vec<RuleApp<T>>,
    pair: Pair<Rule>,
    context: &mut Context,
) -> Option<T> {
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
                }
                Application::S(apply) => {
                    // cloning is slow, but fixing takes work
                    let mut new_pair = pair.clone();
                    let mut pairs = pair.into_inner();
                    for unwrap in 0..potential.unwrap {
                        new_pair = pairs.next().unwrap();
                        pairs = new_pair.clone().into_inner(); // whatever, just whatever
                    }
                    Some(apply(new_pair.as_span().as_str().to_string(), context))
                }
            };
        }
    }
    None
}

fn optional_vec<T>(
    potentials: Vec<RuleApp<T>>,
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Option<T> {
    match pairs.peek() {
        None => None,
        Some(pair) => match apply_pair(&potentials, pair, context) {
            None => None,
            t => {
                pairs.next();
                t
            }
        },
    }
}

fn optional<T>(
    potentials: RuleApp<T>,
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Option<T> {
    optional_vec(vec![potentials], pairs, context)
}

fn expect_raw<T>(potentials: &Vec<RuleApp<T>>, pair: Pair<Rule>, context: &mut Context) -> T {
    let rule = pair.as_rule();
    let span = pair.as_span();
    match apply_pair(&potentials, pair, context) {
        Some(result) => result,
        None => {
            println!("{:?}", span);
            panic!(unexpected_rule(potentials, rule))
        }
    }
}

fn expect_vec<T>(potentials: Vec<RuleApp<T>>, pairs: &mut Pairs<Rule>, context: &mut Context) -> T {
    let pair = pairs.next().unwrap();
    expect_raw(&potentials, pair, context)
}

fn expect<T>(potential: RuleApp<T>, pairs: &mut Pairs<Rule>, context: &mut Context) -> T {
    expect_vec(vec![potential], pairs, context)
}

fn expect_hole<T>(
    potential: RuleApp<T>,
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Hole<T> {
    let mut rules = Vec::new();
    let some_rule = match potential.application {
        Application::P(f) => rules.push(rule_pair_unwrap(
            potential.rule,
            potential.unwrap,
            compose_pair(f, Some),
        )),
        Application::S(f) => rules.push(rule_str_unwrap(
            potential.rule,
            potential.unwrap,
            compose_str(f, Some),
        )),
    };
    rules.push(rule_hole());
    expect_vec(rules, pairs, context)
}

fn expect_node_hole<T>(
    potential: RuleApp<T>,
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Hole<T> {
    let mut rules = Vec::new();
    let some_rule = match potential.application {
        Application::P(f) => rules.push(rule_pair_unwrap(
            potential.rule,
            potential.unwrap,
            compose_pair(f, Some),
        )),
        Application::S(f) => rules.push(rule_str_unwrap(
            potential.rule,
            potential.unwrap,
            compose_str(f, Some),
        )),
    };
    rules.push(rule_node_hole());
    expect_vec(rules, pairs, context)
}

fn expect_all_vec<T>(
    potentials: Vec<RuleApp<T>>,
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<T> {
    let mut result = Vec::new();
    for pair in pairs {
        result.push(expect_raw(&potentials, pair, context));
    }
    result
}

fn expect_all<T>(potential: RuleApp<T>, pairs: &mut Pairs<Rule>, context: &mut Context) -> Vec<T> {
    expect_all_vec(vec![potential], pairs, context)
}

// Core Reading

fn read_n(s: String, context: &mut Context) -> usize {
    s.parse::<usize>().unwrap()
}

fn rule_n<'a>() -> RuleApp<'a, usize> {
    rule_str(Rule::n, read_n)
}

fn read_string(s: String, context: &mut Context) -> String {
    s
}

fn read_string_as<T>(s: String, context: &mut Context) -> T
where T {
    s
}

fn rule_id_raw<'a>() -> RuleApp<'a, String> {
    rule_str(Rule::id, read_string)
}

fn rule_n_raw<'a>() -> RuleApp<'a, String> {
    rule_str(Rule::n, read_string)
}

fn rule_string<'a>(rule: Rule) -> RuleApp<'a, String> {
    rule_str(rule, read_string)
}

fn rule_funclet_id<'a>(rule: Rule) -> RuleApp<'a, FuncletId> {
    rule_str(rule, read_funclet_id)
}

fn rule_operation_id<'a>(rule: Rule) -> RuleApp<'a, OperationId> {
    rule_str(rule, read_operation_id)
}

fn read_string_clean(s: String, context: &mut Context) -> String {
    (&s[1..s.len() - 1]).to_string()
}

fn rule_string_clean<'a>() -> RuleApp<'a, String> {
    rule_str(Rule::str, read_string_clean)
}

fn read_type_raw(pairs: &mut Pairs<Rule>, context: &mut Context) -> TypeId {
    let mut rules = Vec::new();
    rules.push(rule_str(Rule::ffi_type, read_string));
    rules.push(rule_str_unwrap(Rule::type_name, 1, Box::new(read_string)));
    expect_vec(rules, pairs, context)
}

fn rule_type_raw<'a>() -> RuleApp<'a, String> {
    rule_pair(Rule::typ, read_type_raw)
}

fn read_hole<T>(s: String, context: &mut Context) -> Option<T> {
    None
}

fn rule_hole<'a, T: 'a>() -> RuleApp<'a, Option<T>> {
    rule_str(Rule::hole, read_hole)
}

fn rule_node_hole<'a, T: 'a>() -> RuleApp<'a, Option<T>> {
    rule_str(Rule::node_hole, read_hole)
}

fn read_id(s: String, context: &mut Context) -> assembly::ast::Value {
    assembly::ast::Value::ID(s)
}

fn rule_id<'a>() -> RuleApp<'a, assembly::ast::Value> {
    rule_str(Rule::id, read_id)
}

fn read_none_value(_: String, _: &mut Context) -> assembly::ast::Value {
    assembly::ast::Value::None
}

fn read_ffi_type_base(s: String, context: &mut Context) -> assembly::ast::FFIType {
    match s.as_str() {
        "f32" => assembly::ast::FFIType::F32,
        "f64" => assembly::ast::FFIType::F64,
        "u8" => assembly::ast::FFIType::U8,
        "u16" => assembly::ast::FFIType::U16,
        "u32" => assembly::ast::FFIType::U32,
        "u64" => assembly::ast::FFIType::U64,
        "i8" => assembly::ast::FFIType::I8,
        "i16" => assembly::ast::FFIType::I16,
        "i32" => assembly::ast::FFIType::I32,
        "i64" => assembly::ast::FFIType::I64,
        "usize" => assembly::ast::FFIType::USize,
        "gpu_buffer_allocator" => assembly::ast::FFIType::GpuBufferAllocator,
        "cpu_buffer_allocator" => assembly::ast::FFIType::CpuBufferAllocator,
        _ => panic!("Unknown type name {}", s),
    }
}

fn read_ffi_ref_parameter(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> assembly::ast::FFIType {
    expect(rule_ffi_type(), pairs, context)
}

fn read_ffi_array_params(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::FFIType {
    let element_type = Box::new(expect(rule_ffi_type(), pairs, context));
    let length = expect(rule_n(), pairs, context);
    assembly::ast::FFIType::Array {
        element_type,
        length,
    }
}

fn read_ffi_tuple_params(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::FFIType {
    let elements = expect_all(rule_ffi_type(), pairs, context);
    assembly::ast::FFIType::Tuple(elements)
}

fn read_ffi_parameterized_ref_name(
    s: String,
    context: &mut Context,
) -> Box<dyn Fn(assembly::ast::FFIType) -> assembly::ast::FFIType> {
    fn box_up<F>(f: &'static F) -> Box<dyn Fn(assembly::ast::FFIType) -> assembly::ast::FFIType>
    where
        F: Fn(Box<assembly::ast::FFIType>) -> assembly::ast::FFIType,
    {
        Box::new(move |x| f(Box::new(x)))
    }
    match s.as_str() {
        "erased_length_array" => box_up(&assembly::ast::FFIType::ErasedLengthArray),
        "const_ref" => box_up(&assembly::ast::FFIType::ConstRef),
        "mut_ref" => box_up(&assembly::ast::FFIType::MutRef),
        "const_slice" => box_up(&assembly::ast::FFIType::ConstSlice),
        "mut_slice" => box_up(&assembly::ast::FFIType::MutSlice),
        "gpu_buffer_ref" => box_up(&assembly::ast::FFIType::GpuBufferRef),
        "gpu_buffer_slice" => box_up(&assembly::ast::FFIType::GpuBufferSlice),
        "cpu_buffer_ref" => box_up(&assembly::ast::FFIType::CpuBufferRef),
        _ => panic!("Unknown type name {}", s),
    }
}

fn read_ffi_parameterized_ref(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> assembly::ast::FFIType {
    let rule = rule_str(
        Rule::ffi_parameterized_ref_name,
        read_ffi_parameterized_ref_name,
    );
    let kind = expect(rule, pairs, context);
    let rule = rule_pair(Rule::ffi_ref_parameter, read_ffi_ref_parameter);
    let value = expect(rule, pairs, context);
    kind(value)
}

fn read_ffi_parameterized_type(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> assembly::ast::FFIType {
    let mut rules = Vec::new();
    let func = Box::new(read_ffi_array_params);
    rules.push(rule_pair_unwrap(Rule::ffi_parameterized_array, 1, func));

    rules.push(rule_pair(
        Rule::ffi_parameterized_ref,
        read_ffi_parameterized_ref,
    ));
    let func = Box::new(read_ffi_tuple_params);
    rules.push(rule_pair_unwrap(Rule::ffi_parameterized_tuple, 1, func));

    expect_vec(rules, pairs, context)
}

fn read_ffi_type(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::FFIType {
    let mut rules = Vec::new();
    rules.push(rule_str(Rule::ffi_type_base, read_ffi_type_base));
    rules.push(rule_pair(
        Rule::ffi_parameterized_type,
        read_ffi_parameterized_type,
    ));
    expect_vec(rules, pairs, context)
}

fn rule_ffi_type<'a>() -> RuleApp<'a, assembly::ast::FFIType> {
    rule_pair(Rule::ffi_type, read_ffi_type)
}

fn rule_ffi_type_sep<'a>() -> RuleApp<'a, assembly::ast::FFIType> {
    rule_pair_unwrap(Rule::ffi_type_sep, 1, Box::new(read_ffi_type))
}

fn read_type(pairs: &mut Pairs<Rule>, context: &mut Context) -> Hole<assembly::ast::Type> {
    let mut rules = Vec::new();
    let ffi_fn = compose_pair(read_ffi_type, assembly::ast::Type::FFI);
    let ffi_fn_wrap = compose_pair(ffi_fn, Some);
    rules.push(rule_pair_boxed(Rule::ffi_type, ffi_fn_wrap));
    let rule_ir = compose_str(read_string, assembly::ast::Type::Local);
    let rule_ir_wrap = compose_str(rule_ir, Some);
    rules.push(rule_str_unwrap(Rule::type_name, 1, rule_ir_wrap));
    rules.push(rule_hole());
    expect_vec(rules, pairs, context)
}

fn rule_type<'a>() -> RuleApp<'a, Hole<assembly::ast::Type>> {
    rule_pair(Rule::typ, read_type)
}

fn rule_type_sep<'a>() -> RuleApp<'a, Hole<assembly::ast::Type>> {
    rule_pair_unwrap(Rule::typ_sep, 1, Box::new(read_type))
}

fn read_throwaway(_: String, context: &mut Context) -> String {
    "_".to_string()
}

fn rule_throwaway<'a>() -> RuleApp<'a, String> {
    rule_str(Rule::throwaway, read_throwaway)
}

fn read_var_name(pairs: &mut Pairs<Rule>, context: &mut Context) -> Hole<String> {
    let mut rules = Vec::new();
    let rule = compose_str(read_string, Some);
    rules.push(rule_str_boxed(Rule::id, rule));
    let rule = compose_str(read_string, Some);
    rules.push(rule_str_boxed(Rule::n, rule));
    let rule = compose_str(read_throwaway, Some);
    rules.push(rule_str_boxed(Rule::throwaway, rule));
    rules.push(rule_hole());
    expect_vec(rules, pairs, context)
}

fn rule_var_name<'a>() -> RuleApp<'a, Hole<String>> {
    rule_pair(Rule::var_name, read_var_name)
}

fn read_fn_name(pairs: &mut Pairs<Rule>, context: &mut Context) -> Hole<FuncletId> {
    let mut rules = Vec::new();
    let rule = compose_str(read_funclet_id, Some);
    rules.push(rule_str_boxed(Rule::id, rule));
    rules.push(rule_hole());
    expect_vec(rules, pairs, context)
}

fn rule_fn_name<'a>() -> RuleApp<'a, Hole<FuncletId>> {
    rule_pair(Rule::fn_name, read_fn_name)
}

fn rule_fn_name_sep<'a>() -> RuleApp<'a, Hole<String>> {
    rule_pair_unwrap(Rule::fn_name_sep, 1, Box::new(read_fn_name))
}

fn read_funclet_loc_filled(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> assembly::ast::RemoteNodeId {
    let rule_func = rule_str_unwrap(Rule::fn_name, 1, Box::new(read_string));
    let rule_var = rule_str_unwrap(Rule::var_name, 1, Box::new(read_string));
    let fun_name = expect(rule_func, pairs, context);
    let var_name = expect(rule_var, pairs, context);
    assembly::ast::RemoteNodeId {
        funclet_name: fun_name,
        node_name: var_name,
    }
}

fn read_funclet_loc(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Hole<assembly::ast::RemoteNodeId> {
    expect_hole(
        rule_pair(Rule::funclet_loc_filled, read_funclet_loc_filled),
        pairs,
        context,
    )
}

fn rule_funclet_loc<'a>() -> RuleApp<'a, Hole<assembly::ast::RemoteNodeId>> {
    rule_pair(Rule::funclet_loc, read_funclet_loc)
}

fn read_place(s: String, context: &mut Context) -> Hole<ir::Place> {
    match s.as_str() {
        "local" => Some(ir::Place::Local),
        "cpu" => Some(ir::Place::Cpu),
        "gpu" => Some(ir::Place::Gpu),
        "?" => None,
        _ => panic!(unexpected(s)),
    }
}

fn rule_place<'a>() -> RuleApp<'a, Hole<ir::Place>> {
    rule_str(Rule::place, read_place)
}

fn read_stage(s: String, context: &mut Context) -> Hole<ir::ResourceQueueStage> {
    match s.as_str() {
        "unbound" => Some(ir::ResourceQueueStage::Unbound),
        "bound" => Some(ir::ResourceQueueStage::Bound),
        "encoded" => Some(ir::ResourceQueueStage::Encoded),
        "submitted" => Some(ir::ResourceQueueStage::Submitted),
        "ready" => Some(ir::ResourceQueueStage::Ready),
        "dead" => Some(ir::ResourceQueueStage::Dead),
        "?" => None,
        _ => panic!(unexpected((s))),
    }
}

fn rule_stage<'a>() -> RuleApp<'a, Hole<ir::ResourceQueueStage>> {
    rule_str(Rule::stage, read_stage)
}

fn read_tag_core_op(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::TagCore {
    let op_type = expect(rule_string(Rule::tag_core_op), pairs, context);
    let funclet_loc = expect(rule_funclet_loc(), pairs, context);
    match op_type.as_str() {
        // "operation" | "input" | "output"
        "operation" => assembly::ast::TagCore::Operation(reject_hole(funclet_loc)),
        "input" => assembly::ast::TagCore::Input(reject_hole(funclet_loc)),
        "output" => assembly::ast::TagCore::Output(reject_hole(funclet_loc)),
        _ => panic!(unexpected(op_type)),
    }
}

fn read_tag_core(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::TagCore {
    let pair = pairs.peek().unwrap();
    let rule = pair.as_rule();
    match rule {
        Rule::none => assembly::ast::TagCore::None,
        Rule::tag_core_op => read_tag_core_op(pairs, context),
        _ => panic!(unexpected_rule_raw(
            vec![Rule::none, Rule::tag_core_op],
            rule,
        )),
    }
}

fn rule_tag_core<'a>() -> RuleApp<'a, assembly::ast::TagCore> {
    rule_pair(Rule::tag_core, read_tag_core)
}

fn read_value_tag_loc(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::ValueTag {
    let op_type = expect(rule_string(Rule::value_tag_op), pairs, context);
    let funclet_loc = expect(rule_funclet_loc(), pairs, context);
    match op_type.as_str() {
        // "function_input" | "function_output"
        "function_input" => assembly::ast::ValueTag::FunctionInput(reject_hole(funclet_loc)),
        "function_output" => assembly::ast::ValueTag::FunctionOutput(reject_hole(funclet_loc)),
        _ => panic!(unexpected(op_type)),
    }
}

fn read_value_tag_data(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::ValueTag {
    let mut rules = vec![];
    let app = compose_pair(read_tag_core, assembly::ast::ValueTag::Core);
    let rule = rule_pair_boxed(Rule::tag_core, app);
    rules.push(rule);

    let rule = rule_pair(Rule::value_tag_loc, read_value_tag_loc);
    rules.push(rule);

    let app = compose_pair_reject(read_var_name, assembly::ast::ValueTag::Halt);
    let rule = rule_pair_unwrap(Rule::tag_halt, 1, app);
    rules.push(rule);
    expect_vec(rules, pairs, context)
}

fn read_value_tag(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::ValueTag {
    require_rule(Rule::value_tag_sep, pairs, context);
    let rule = rule_pair(Rule::value_tag_data, read_value_tag_data);
    expect(rule, pairs, context)
}

fn rule_value_tag<'a>() -> RuleApp<'a, assembly::ast::ValueTag> {
    rule_pair(Rule::value_tag, read_value_tag)
}

fn read_timeline_tag_data(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> assembly::ast::TimelineTag {
    assembly::ast::TimelineTag::Core(expect(rule_tag_core(), pairs, context))
}

fn read_timeline_tag(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::TimelineTag {
    require_rule(Rule::timeline_tag_sep, pairs, context);
    let rule = rule_pair(Rule::timeline_tag_data, read_timeline_tag_data);
    expect(rule, pairs, context)
}

fn rule_timeline_tag<'a>() -> RuleApp<'a, assembly::ast::TimelineTag> {
    rule_pair(Rule::timeline_tag, read_timeline_tag)
}

fn read_spatial_tag_data(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> assembly::ast::SpatialTag {
    assembly::ast::SpatialTag::Core(expect(rule_tag_core(), pairs, context))
}

fn read_spatial_tag(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::SpatialTag {
    require_rule(Rule::spatial_tag_sep, pairs, context);
    let rule = rule_pair(Rule::spatial_tag_data, read_spatial_tag_data);
    expect(rule, pairs, context)
}

fn rule_spatial_tag<'a>() -> RuleApp<'a, assembly::ast::SpatialTag> {
    rule_pair(Rule::spatial_tag, read_spatial_tag)
}

fn read_tag(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Tag {
    let mut rules = vec![];
    let value_app = compose_pair(read_value_tag, assembly::ast::Tag::ValueTag);
    rules.push(rule_pair_boxed(Rule::value_tag, value_app));
    let timeline_app = compose_pair(read_timeline_tag, assembly::ast::Tag::TimelineTag);
    rules.push(rule_pair_boxed(Rule::timeline_tag, timeline_app));
    let spatial_app = compose_pair(read_spatial_tag, assembly::ast::Tag::SpatialTag);
    rules.push(rule_pair_boxed(Rule::spatial_tag, spatial_app));
    expect_vec(rules, pairs, context)
}

fn read_slot_info(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::SlotInfo {
    let mut rules = vec![];
    rules.push(rule_pair(Rule::tag, read_tag));
    let tags = expect_all_vec(rules, pairs, context);
    let mut value_tag = assembly::ast::ValueTag::Core(assembly::ast::TagCore::None);
    let mut timeline_tag = assembly::ast::TimelineTag::Core(assembly::ast::TagCore::None);
    let mut spatial_tag = assembly::ast::SpatialTag::Core(assembly::ast::TagCore::None);
    for tag in tags.iter() {
        match tag {
            // duplicates are whatever
            assembly::ast::Tag::ValueTag(t) => value_tag = t.clone(),
            assembly::ast::Tag::TimelineTag(t) => timeline_tag = t.clone(),
            assembly::ast::Tag::SpatialTag(t) => spatial_tag = t.clone(),
        }
    }
    assembly::ast::SlotInfo {
        value_tag,
        timeline_tag,
        spatial_tag,
    }
}

fn read_fence_info(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::FenceInfo {
    let rule = rule_pair(Rule::timeline_tag, read_timeline_tag);
    match pairs.peek() {
        None => assembly::ast::FenceInfo {
            timeline_tag: assembly::ast::TimelineTag::Core(assembly::ast::TagCore::None),
        },
        Some(_) => assembly::ast::FenceInfo {
            timeline_tag: expect(rule, pairs, context),
        },
    }
}

fn read_buffer_info(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::BufferInfo {
    let rule = rule_pair(Rule::spatial_tag, read_spatial_tag);
    match pairs.peek() {
        None => assembly::ast::BufferInfo {
            spatial_tag: expect(rule, pairs, context),
        },
        Some(_) => assembly::ast::BufferInfo {
            spatial_tag: expect(rule, pairs, context),
        },
    }
}

fn read_value(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Value {
    let mut rules = Vec::new();

    rules.push(rule_str(Rule::none, read_none_value));
    let rule = compose_str(read_n, assembly::ast::Value::Num);
    rules.push(rule_str_boxed(Rule::n, rule));
    let rule = compose_pair_reject(read_fn_name, assembly::ast::Value::VarName);
    rules.push(rule_pair_boxed(Rule::var_name, rule));
    let rule = compose_pair_reject(read_funclet_loc, assembly::ast::Value::FunctionLoc);
    rules.push(rule_pair_boxed(Rule::funclet_loc, rule));
    let rule = compose_pair_reject(read_fn_name, assembly::ast::Value::FnName);
    rules.push(rule_pair_boxed(Rule::fn_name, rule));
    let rule = compose_pair_reject(read_type, assembly::ast::Value::Type);
    rules.push(rule_pair_boxed(Rule::typ, rule));
    let rule = compose_str_reject(read_place, assembly::ast::Value::Place);
    rules.push(rule_str_boxed(Rule::place, rule));
    let rule = compose_str_reject(read_stage, assembly::ast::Value::Stage);
    rules.push(rule_str_boxed(Rule::stage, rule));
    let rule = compose_pair(read_tag, assembly::ast::Value::Tag);
    rules.push(rule_pair_boxed(Rule::tag, rule));
    let rule = compose_pair(read_slot_info, assembly::ast::Value::SlotInfo);
    rules.push(rule_pair_boxed(Rule::slot_info, rule));
    let rule = compose_pair(read_fence_info, assembly::ast::Value::FenceInfo);
    rules.push(rule_pair_boxed(Rule::fence_info, rule));
    let rule = compose_pair(read_buffer_info, assembly::ast::Value::BufferInfo);
    rules.push(rule_pair_boxed(Rule::buffer_info, rule));

    expect_vec(rules, pairs, context)
}

fn rule_value<'a>() -> RuleApp<'a, assembly::ast::Value> {
    rule_pair(Rule::value, read_value)
}

fn read_list_values(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<assembly::ast::DictValue> {
    let rule = rule_pair(Rule::dict_value, read_dict_value);
    expect_all(rule, pairs, context)
}

fn read_list(pairs: &mut Pairs<Rule>, context: &mut Context) -> Vec<assembly::ast::DictValue> {
    let rule = rule_pair(Rule::list_values, read_list_values);
    option_to_vec(optional(rule, pairs, context))
}

fn read_dict_value(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::DictValue {
    let mut rules = Vec::new();
    let value_map = compose_pair(read_value, assembly::ast::DictValue::Raw);
    rules.push(rule_pair_boxed(Rule::value, value_map));

    let list_map = compose_pair(read_list, assembly::ast::DictValue::List);
    rules.push(rule_pair_boxed(Rule::list, list_map));

    let dict_map = compose_pair(read_unchecked_dict, assembly::ast::DictValue::Dict);
    rules.push(rule_pair_boxed(Rule::unchecked_dict, dict_map));

    expect_vec(rules, pairs, context)
}

fn read_dict_key(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Value {
    let value_var_name = compose_pair_reject(read_var_name, assembly::ast::Value::VarName);
    let rule_var_name = rule_pair_boxed(Rule::var_name, value_var_name);
    expect_vec(vec![rule_id(), rule_var_name], pairs, context)
}

struct DictPair {
    key: assembly::ast::Value,
    value: assembly::ast::DictValue,
}

fn read_dict_element(pairs: &mut Pairs<Rule>, context: &mut Context) -> DictPair {
    let rule_key = rule_pair(Rule::dict_key, read_dict_key);
    let rule_value = rule_pair(Rule::dict_value, read_dict_value);
    let key = expect(rule_key, pairs, context);
    let value = expect(rule_value, pairs, context);
    DictPair { key, value }
}

fn read_unchecked_dict(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> assembly::ast::UncheckedDict {
    let rule = rule_pair(Rule::dict_element, read_dict_element);
    let mut result = HashMap::new();
    for pair in expect_all(rule, pairs, context) {
        result.insert(pair.key, pair.value);
    }
    result
}

fn rule_unchecked_dict<'a>() -> RuleApp<'a, assembly::ast::UncheckedDict> {
    rule_pair(Rule::unchecked_dict, read_unchecked_dict)
}

// Readers

fn read_version(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Version {
    require_rule(Rule::version_keyword, pairs, context);
    let major_s = expect(rule_n_raw(), pairs, context);
    let minor_s = expect(rule_n_raw(), pairs, context);
    let detailed_s = expect(rule_n_raw(), pairs, context);

    let major = major_s.parse::<u32>().unwrap();
    let minor = minor_s.parse::<u32>().unwrap();
    let detailed = detailed_s.parse::<u32>().unwrap();

    assembly::ast::Version {
        major,
        minor,
        detailed,
    }
}

fn interpret_ir_dict(
    s: String,
    data: assembly::ast::UncheckedDict,
) -> assembly::ast::LocalTypeInfo {
    match s.as_str() {
        "native_value" => {
            let storage_type =
                match unchecked_value(&assembly::ast::Value::ID("type".to_string()), &data) {
                    assembly::ast::Value::Type(t) => t,
                    v => panic!("Unsupported storage type {:?}", v),
                };
            assembly::ast::LocalTypeInfo::NativeValue { storage_type }
        }
        "slot" => {
            let storage_type =
                match unchecked_value(&assembly::ast::Value::ID("type".to_string()), &data) {
                    assembly::ast::Value::Type(t) => t,
                    v => panic!("Unsupported storage type {:?}", v),
                };
            let queue_stage =
                match unchecked_value(&assembly::ast::Value::ID("stage".to_string()), &data) {
                    assembly::ast::Value::Stage(t) => t,
                    v => panic!("Unsupported queue stage {:?}", v),
                };
            let queue_place =
                match unchecked_value(&assembly::ast::Value::ID("place".to_string()), &data) {
                    assembly::ast::Value::Place(t) => t,
                    v => panic!("Unsupported queue place {:?}", v),
                };
            assembly::ast::LocalTypeInfo::Slot {
                storage_type,
                queue_stage,
                queue_place,
            }
        }
        "fence" => {
            let queue_place =
                match unchecked_value(&assembly::ast::Value::ID("place".to_string()), &data) {
                    assembly::ast::Value::Place(t) => t,
                    v => panic!("Unsupported queue place {:?}", v),
                };
            assembly::ast::LocalTypeInfo::Fence { queue_place }
        }
        "buffer" => {
            let storage_place =
                match unchecked_value(&assembly::ast::Value::ID("place".to_string()), &data) {
                    assembly::ast::Value::Place(t) => t,
                    v => panic!("Unsupported storage place {:?}", v),
                };
            let static_layout_opt = data
                .get(&assembly::ast::Value::ID("static_layout_opt".to_string()))
                .and_then(|v| match v {
                    assembly::ast::DictValue::Dict(d) => {
                        let alignment_bits = match unchecked_value(
                            &assembly::ast::Value::ID("alignment_bits".to_string()),
                            &d,
                        ) {
                            assembly::ast::Value::Num(n) => n,
                            v => panic!("Unsupported alignment bits {:?}", v),
                        };
                        let byte_size = match unchecked_value(
                            &assembly::ast::Value::ID("byte_size".to_string()),
                            &d,
                        ) {
                            assembly::ast::Value::Num(n) => n,
                            v => panic!("Unsupported byte size {:?}", v),
                        };
                        Some(ir::StaticBufferLayout {
                            alignment_bits,
                            byte_size,
                        })
                    }
                    _ => panic!("Unsupported static layout opt {:?}", v),
                });
            assembly::ast::LocalTypeInfo::Buffer {
                storage_place,
                static_layout_opt,
            }
        }
        "space_buffer" => assembly::ast::LocalTypeInfo::BufferSpace,
        "event" => {
            let place = match unchecked_value(&assembly::ast::Value::ID("place".to_string()), &data)
            {
                assembly::ast::Value::Place(t) => t,
                v => panic!("Unsupported place {:?}", v),
            };
            assembly::ast::LocalTypeInfo::Event { place }
        }
        _ => panic!("Unexpected slot check {:?}", s),
    }
}

fn read_ir_type_decl(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::TypeDecl {
    let event_rule = rule_str_unwrap(Rule::ir_type_decl_key_sep, 1, Box::new(read_string));
    let type_kind = expect(event_rule, pairs, context);
    let name_rule = rule_str_unwrap(Rule::type_name, 1, Box::new(read_string));
    let name = expect(name_rule, pairs, context);
    let unchecked_dict = expect(rule_unchecked_dict(), pairs, context);
    let data = interpret_ir_dict(type_kind, unchecked_dict);
    let result = assembly::ast::LocalType { name, data };
    assembly::ast::TypeDecl::Local(result)
}

fn read_ffi_type_decl(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::TypeDecl {
    let ffi_typ = expect(rule_ffi_type(), pairs, context);
    assembly::ast::TypeDecl::FFI(ffi_typ)
}

fn read_type_def(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::TypeDecl {
    let mut rules = Vec::new();
    rules.push(rule_pair(Rule::ffi_type_decl, read_ffi_type_decl));
    rules.push(rule_pair(Rule::ir_type_decl, read_ir_type_decl));
    expect_vec(rules, pairs, context)
}

fn read_types(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Types {
    let rule = rule_pair(Rule::type_def, read_type_def);
    expect_all(rule, pairs, context)
}

fn read_external_cpu_args(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<assembly::ast::FFIType> {
    expect_all(rule_ffi_type(), pairs, context)
}

fn read_external_cpu_return_args(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<assembly::ast::FFIType> {
    let mut rules = Vec::new();
    rules.push(rule_pair(Rule::external_cpu_args, read_external_cpu_args));
    rules.push(rule_pair_boxed(
        Rule::ffi_type,
        compose_pair(read_ffi_type, |t| vec![t]),
    ));
    expect_vec(rules, pairs, context)
}

fn read_external_cpu_funclet(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> assembly::ast::ExternalCpuFunction {
    require_rule(Rule::external_cpu_sep, pairs, context);

    let name = reject_hole(expect(rule_fn_name(), pairs, context));

    let rule_extern_args = rule_pair(Rule::external_cpu_args, read_external_cpu_args);
    let input_types = expect(rule_extern_args, pairs, context);

    let rule_extern_return_args = rule_pair(
        Rule::external_cpu_return_args,
        read_external_cpu_return_args,
    );
    let output_types = expect(rule_extern_return_args, pairs, context);
    assembly::ast::ExternalCpuFunction {
        name,
        input_types,
        output_types,
    }
}

fn read_external_gpu_arg(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> (assembly::ast::FFIType, String) {
    let name = reject_hole(expect(rule_var_name(), pairs, context));
    let typ = expect(rule_ffi_type(), pairs, context);
    (typ, name)
}

fn read_external_gpu_args(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<(assembly::ast::FFIType, String)> {
    let rule = rule_pair(Rule::external_gpu_arg, read_external_gpu_arg);
    expect_all(rule, pairs, context)
}

fn read_external_gpu_body(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<assembly::ast::UncheckedDict> {
    let rule = rule_pair_unwrap(
        Rule::external_gpu_resource,
        1,
        Box::new(read_unchecked_dict),
    );
    expect_all(rule, pairs, context)
}

fn interpret_external_gpu_dict(
    data: assembly::ast::UncheckedDict,
) -> assembly::ast::ExternalGpuFunctionResourceBinding {
    let group = match unchecked_value(&assembly::ast::Value::ID("group".to_string()), &data) {
        assembly::ast::Value::Num(n) => n,
        v => panic!("Unsupported group {:?}", v),
    };
    let binding = match unchecked_value(&assembly::ast::Value::ID("binding".to_string()), &data) {
        assembly::ast::Value::Num(n) => n,
        v => panic!("Unsupported binding {:?}", v),
    };
    let input = data
        .get(&assembly::ast::Value::ID("input".to_string()))
        .and_then(|v| match v {
            assembly::ast::DictValue::Raw(assembly::ast::Value::VarName(t)) => Some(t.clone()),
            v => panic!("Unsupported input {:?}", v),
        });
    let output = data
        .get(&assembly::ast::Value::ID("output".to_string()))
        .and_then(|v| match v {
            assembly::ast::DictValue::Raw(assembly::ast::Value::VarName(t)) => Some(t.clone()),
            v => panic!("Unsupported output {:?}", v),
        });
    assembly::ast::ExternalGpuFunctionResourceBinding {
        group,
        binding,
        input,
        output,
    }
}

fn read_external_gpu_funclet(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> assembly::ast::ExternalGpuFunction {
    require_rule(Rule::external_gpu_sep, pairs, context);

    let name = reject_hole(expect(rule_fn_name(), pairs, context));

    let rule_extern_args = rule_pair(Rule::external_gpu_args, read_external_gpu_args);
    let input_args = expect(rule_extern_args, pairs, context);

    let rule_extern_args = rule_pair(Rule::external_gpu_args, read_external_gpu_args);
    let output_types = expect(rule_extern_args, pairs, context);

    let shader_module = expect(rule_string_clean(), pairs, context);

    let rule_binding = rule_pair(Rule::external_gpu_body, read_external_gpu_body);
    let unchecked_bindings = expect(rule_binding, pairs, context);

    let resource_bindings = unchecked_bindings
        .into_iter()
        .map(|d| interpret_external_gpu_dict(d))
        .collect();

    assembly::ast::ExternalGpuFunction {
        name,
        input_args,
        output_types,
        shader_module,
        entry_point: "main".to_string(), // todo: uhhhh, allow syntax perhaps
        resource_bindings,
    }
}

fn read_external_funclet(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> assembly::ast::FuncletDef {
    let mut rules = Vec::new();
    let comp = compose_pair(
        read_external_cpu_funclet,
        assembly::ast::FuncletDef::ExternalCPU,
    );
    rules.push(rule_pair_boxed(Rule::external_cpu, comp));
    let comp = compose_pair(
        read_external_gpu_funclet,
        assembly::ast::FuncletDef::ExternalGPU,
    );
    rules.push(rule_pair_boxed(Rule::external_gpu, comp));
    expect_vec(rules, pairs, context)
}

fn read_funclet_arg(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> (Option<String>, assembly::ast::Type) {
    let pair = pairs.next().unwrap();
    let rule = pair.as_rule();
    match rule {
        Rule::var_name => {
            // You gotta add the phi node when translating IRs when you do this!
            let var = reject_hole(read_var_name(&mut pair.into_inner(), context));
            let typ = reject_hole(expect(rule_type(), pairs, context));
            (Some(var), typ)
        }
        Rule::typ => (
            None,
            reject_hole(read_type(&mut pair.into_inner(), context)),
        ),
        _ => panic!(unexpected_rule_raw(vec![Rule::var_name, Rule::typ], rule)),
    }
}

fn read_funclet_args(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<(Option<String>, assembly::ast::Type)> {
    let rule = rule_pair(Rule::funclet_arg, read_funclet_arg);
    expect_all(rule, pairs, context)
}

fn read_funclet_return_arg(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> (Option<String>, assembly::ast::Type) {
    let pair = pairs.next().unwrap();
    let rule = pair.as_rule();
    match rule {
        Rule::var_name => {
            // You gotta add the phi node when translating IRs when you do this!
            let var = reject_hole(read_var_name(&mut pair.into_inner(), context));
            let typ = reject_hole(expect(rule_type(), pairs, context));
            (Some(var), typ)
        }
        Rule::typ => (
            None,
            reject_hole(read_type(&mut pair.into_inner(), context)),
        ),
        _ => panic!(unexpected_rule_raw(vec![Rule::var_name, Rule::typ], rule)),
    }
}

fn read_funclet_return_args(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<(Option<String>, assembly::ast::Type)> {
    let rule = rule_pair(Rule::funclet_arg, read_funclet_return_arg);
    expect_all(rule, pairs, context)
}

fn read_funclet_return(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<(Option<String>, assembly::ast::Type)> {
    let mut rules = Vec::new();
    rules.push(rule_pair(Rule::funclet_args, read_funclet_return_args));
    rules.push(rule_pair_boxed(
        Rule::typ,
        compose_pair_reject(read_type, |t| vec![(None, t)]),
    ));
    expect_vec(rules, pairs, context)
}

fn read_funclet_header(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> (assembly::ast::FuncletHeader, Vec<String>) {
    let name = reject_hole(expect(rule_fn_name(), pairs, context));

    let rule_args = rule_pair(Rule::funclet_args, read_funclet_args);
    let named_args = option_to_vec(optional(rule_args, pairs, context));
    let args = named_args.clone().into_iter().map(|x| x.1).collect();
    let names = named_args
        .into_iter()
        .map(|x| x.0.unwrap_or("_".to_string()))
        .collect();

    let rule_return = rule_pair(Rule::funclet_return, read_funclet_return);
    let ret = expect(rule_return, pairs, context);
    (assembly::ast::FuncletHeader { ret, name, args }, names)
}

fn rule_funclet_header<'a>() -> RuleApp<'a, (assembly::ast::FuncletHeader, Vec<String>)> {
    rule_pair(Rule::funclet_header, read_funclet_header)
}

fn read_var_assign(pairs: &mut Pairs<Rule>, context: &mut Context) -> String {
    let var = reject_hole(expect(rule_var_name(), pairs, context));
    var
}

fn read_node_list(pairs: &mut Pairs<Rule>, context: &mut Context) -> Vec<Hole<String>> {
    expect_all(rule_var_name(), pairs, context)
}

fn read_node_box_raw(pairs: &mut Pairs<Rule>, context: &mut Context) -> Vec<Hole<String>> {
    match pairs.peek() {
        None => {
            vec![]
        }
        Some(_) => {
            let rule = rule_pair(Rule::node_list, read_node_list);
            expect(rule, pairs, context)
        }
    }
}

fn read_node_box(pairs: &mut Pairs<Rule>, context: &mut Context) -> Hole<Vec<Hole<String>>> {
    expect_hole(
        rule_pair(Rule::node_box_raw, read_node_box_raw),
        pairs,
        context,
    )
}

fn rule_node_box<'a>() -> RuleApp<'a, Hole<Vec<Hole<String>>>> {
    rule_pair(Rule::node_box, read_node_box)
}

fn read_return_args(pairs: &mut Pairs<Rule>, context: &mut Context) -> Vec<Hole<String>> {
    expect_all(rule_var_name(), pairs, context)
}

fn read_return_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::TailEdge {
    require_rule(Rule::return_sep, pairs, context);
    let rule = rule_pair(Rule::return_args, read_return_args);
    let return_values = expect_node_hole(rule, pairs, context);
    assembly::ast::TailEdge::Return { return_values }
}

fn rule_return_command<'a>() -> RuleApp<'a, assembly::ast::TailEdge> {
    rule_pair(Rule::return_command, read_return_command)
}

fn read_yield_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::TailEdge {
    require_rule(Rule::yield_sep, pairs, context);
    let point_id_hole = expect_hole(rule_n(), pairs, context);
    let pipeline_yield_point_id = point_id_hole.map(ir::PipelineYieldPointId);
    let yielded_nodes = expect(rule_node_box(), pairs, context);
    let next_funclet = expect(rule_fn_name(), pairs, context);
    let continuation_join = expect(rule_var_name(), pairs, context);
    let arguments = expect(rule_node_box(), pairs, context);
    assembly::ast::TailEdge::Yield {
        pipeline_yield_point_id,
        yielded_nodes,
        next_funclet,
        continuation_join,
        arguments,
    }
}

fn rule_yield_command<'a>() -> RuleApp<'a, assembly::ast::TailEdge> {
    rule_pair(Rule::yield_command, read_yield_command)
}

fn read_jump_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::TailEdge {
    require_rule(Rule::jump_sep, pairs, context);
    let join = expect(rule_var_name(), pairs, context);
    let arguments = expect(rule_node_box(), pairs, context);
    assembly::ast::TailEdge::Jump { join, arguments }
}

fn rule_jump_command<'a>() -> RuleApp<'a, assembly::ast::TailEdge> {
    rule_pair(Rule::jump_command, read_jump_command)
}

fn read_schedule_call_command(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> assembly::ast::TailEdge {
    require_rule(Rule::schedule_call_sep, pairs, context);
    let value_operation = expect(rule_funclet_loc(), pairs, context);
    let callee_funclet_id = expect(rule_fn_name(), pairs, context);
    let callee_arguments = expect(rule_node_box(), pairs, context);
    let continuation_join = expect(rule_var_name(), pairs, context);
    assembly::ast::TailEdge::ScheduleCall {
        value_operation,
        callee_funclet_id,
        callee_arguments,
        continuation_join,
    }
}

fn rule_schedule_call_command<'a>() -> RuleApp<'a, assembly::ast::TailEdge> {
    rule_pair(Rule::schedule_call_command, read_schedule_call_command)
}

fn read_tail_fn_nodes(pairs: &mut Pairs<Rule>, context: &mut Context) -> Vec<Hole<String>> {
    expect_all(rule_fn_name(), pairs, context)
}

fn read_tail_fn_box_raw(pairs: &mut Pairs<Rule>, context: &mut Context) -> Vec<Hole<String>> {
    match pairs.peek() {
        None => {
            vec![]
        }
        Some(_) => {
            let rule = rule_pair(Rule::tail_fn_nodes, read_tail_fn_nodes);
            expect(rule, pairs, context)
        }
    }
}

fn read_tail_fn_box(pairs: &mut Pairs<Rule>, context: &mut Context) -> Hole<Vec<Hole<String>>> {
    expect_hole(
        rule_pair(Rule::tail_fn_box_raw, read_tail_fn_box_raw),
        pairs,
        context,
    )
}

fn rule_tail_fn_box<'a>() -> RuleApp<'a, Hole<Vec<Hole<String>>>> {
    rule_pair(Rule::tail_fn_box, read_tail_fn_box)
}

fn read_schedule_select_command(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> assembly::ast::TailEdge {
    require_rule(Rule::schedule_select_sep, pairs, context);
    let value_operation = expect(rule_funclet_loc(), pairs, context);
    let condition = expect(rule_var_name(), pairs, context);
    let callee_funclet_ids = expect(rule_tail_fn_box(), pairs, context);
    let callee_arguments = expect(rule_node_box(), pairs, context);
    let continuation_join = expect(rule_var_name(), pairs, context);
    assembly::ast::TailEdge::ScheduleSelect {
        value_operation,
        condition,
        callee_funclet_ids,
        callee_arguments,
        continuation_join,
    }
}

fn rule_schedule_select_command<'a>() -> RuleApp<'a, assembly::ast::TailEdge> {
    rule_pair(Rule::schedule_select_command, read_schedule_select_command)
}

fn read_tail_none(_: String, _: &mut Context) -> Option<Hole<String>> {
    None
}

fn read_tail_option_node(pairs: &mut Pairs<Rule>, context: &mut Context) -> Option<Hole<String>> {
    let mut rules = Vec::new();
    let apply_some = compose_pair(read_var_name, Some);
    rules.push(rule_pair_boxed(Rule::var_name, apply_some));
    rules.push(rule_str(Rule::none, read_tail_none));
    expect_vec(rules, pairs, context)
}

fn read_tail_option_nodes(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<Option<Hole<String>>> {
    let rule = rule_pair(Rule::tail_option_node, read_tail_option_node);
    expect_all(rule, pairs, context)
}

fn read_tail_option_box_raw(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<Option<Hole<String>>> {
    match pairs.peek() {
        None => {
            vec![]
        }
        Some(_) => {
            let rule = rule_pair(Rule::tail_option_nodes, read_tail_option_nodes);
            expect(rule, pairs, context)
        }
    }
}

fn read_tail_option_box(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Hole<Vec<Option<Hole<String>>>> {
    expect_hole(
        rule_pair(Rule::tail_option_box_raw, read_tail_option_box_raw),
        pairs,
        context,
    )
}

fn rule_tail_option_box<'a>() -> RuleApp<'a, Hole<Vec<Option<Hole<String>>>>> {
    rule_pair(Rule::tail_option_box, read_tail_option_box)
}

fn read_dynamic_alloc_command(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> assembly::ast::TailEdge {
    require_rule(Rule::dynamic_alloc_sep, pairs, context);
    let buffer = expect(rule_var_name(), pairs, context);
    let arguments = expect(rule_node_box(), pairs, context);
    let dynamic_allocation_size_slots = expect(rule_tail_option_box(), pairs, context);
    let success_funclet_id = expect(rule_fn_name(), pairs, context);
    let failure_funclet_id = expect(rule_fn_name(), pairs, context);
    let continuation_join = expect(rule_var_name(), pairs, context);
    assembly::ast::TailEdge::DynamicAllocFromBuffer {
        buffer,
        arguments,
        dynamic_allocation_size_slots,
        success_funclet_id,
        failure_funclet_id,
        continuation_join,
    }
}

fn rule_dynamic_alloc_command<'a>() -> RuleApp<'a, assembly::ast::TailEdge> {
    rule_pair(Rule::dynamic_alloc_command, read_dynamic_alloc_command)
}

fn read_tail_edge(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::TailEdge {
    let mut rules = Vec::new();
    rules.push(rule_return_command());
    rules.push(rule_yield_command());
    rules.push(rule_jump_command());
    rules.push(rule_schedule_call_command());
    rules.push(rule_schedule_select_command());
    rules.push(rule_dynamic_alloc_command());
    expect_vec(rules, pairs, context)
}

fn read_phi_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    let index = expect_hole(rule_n(), pairs, context);
    assembly::ast::Node::Phi { index }
}

fn rule_phi_command<'a>() -> RuleApp<'a, assembly::ast::Node> {
    rule_pair(Rule::phi_command, read_phi_command)
}

fn read_constant_raw(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    let value = Some(expect(rule_n_raw(), pairs, context));
    let type_id = Some(assembly::ast::Type::FFI(expect(
        rule_ffi_type(),
        pairs,
        context,
    )));

    assembly::ast::Node::Constant { value, type_id }
}

fn read_constant_hole(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    assembly::ast::Node::Constant {
        value: None,
        type_id: None,
    }
}

fn read_constant(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    let mut rules = Vec::new();
    rules.push(rule_pair(Rule::constant_raw, read_constant_raw));
    rules.push(rule_pair(Rule::hole, read_constant_hole));
    expect_vec(rules, pairs, context)
}

fn read_constant_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    let rule = rule_pair(Rule::constant, read_constant);
    expect(rule, pairs, context)
}

fn rule_constant_command<'a>() -> RuleApp<'a, assembly::ast::Node> {
    rule_pair(Rule::constant_command, read_constant_command)
}

fn read_extract_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    require_rule(Rule::extract_sep, pairs, context);
    let node_id = expect(rule_var_name(), pairs, context);
    let index = Some(expect(rule_n(), pairs, context));
    assembly::ast::Node::ExtractResult { node_id, index }
}

fn rule_extract_command<'a>() -> RuleApp<'a, assembly::ast::Node> {
    rule_pair(Rule::extract_command, read_extract_command)
}

fn read_call_args(pairs: &mut Pairs<Rule>, context: &mut Context) -> Vec<Hole<String>> {
    expect_all(rule_var_name(), pairs, context)
}

fn read_call_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    require_rule(Rule::call_sep, pairs, context);
    let external_function_id = expect(rule_fn_name(), pairs, context);
    let rule = rule_pair(Rule::call_args, read_call_args);
    let args = expect_hole(rule, pairs, context).map(|x| x.into_boxed_slice());
    match pairs.peek() {
        None => {
            // NOTE: semi-arbitrary choice for unification
            assembly::ast::Node::CallExternalCpu {
                external_function_id,
                arguments: args,
            }
        }
        Some(_) => {
            let rule = rule_pair(Rule::call_args, read_call_args);
            let arguments = expect_hole(rule, pairs, context).map(|x| x.into_boxed_slice());
            assembly::ast::Node::CallExternalGpuCompute {
                external_function_id,
                arguments,
                dimensions: args,
            }
        }
    }
}

fn rule_call_command<'a>() -> RuleApp<'a, assembly::ast::Node> {
    rule_pair(Rule::call_command, read_call_command)
}

fn read_select_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    require_rule(Rule::select_sep, pairs, context);
    let condition = expect(rule_var_name(), pairs, context);
    let true_case = expect(rule_var_name(), pairs, context);
    let false_case = expect(rule_var_name(), pairs, context);
    assembly::ast::Node::Select {
        condition,
        true_case,
        false_case,
    }
}

fn rule_select_command<'a>() -> RuleApp<'a, assembly::ast::Node> {
    rule_pair(Rule::select_command, read_select_command)
}

fn read_value_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    let mut rules = Vec::new();
    rules.push(rule_phi_command());
    rules.push(rule_constant_command());
    rules.push(rule_extract_command());
    rules.push(rule_call_command());
    rules.push(rule_select_command());
    expect_vec(rules, pairs, context)
}

fn read_value_assign(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::NamedNode {
    let name = reject_hole(expect(rule_var_name(), pairs, context));
    let rule = rule_pair(Rule::value_command, read_value_command);
    let node = expect(rule, pairs, context);
    assembly::ast::NamedNode { name, node }
}

fn read_alloc_temporary_command(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> assembly::ast::Node {
    let place = expect(rule_place(), pairs, context);
    let storage_type = expect(rule_type(), pairs, context);
    let operation = expect(rule_funclet_loc(), pairs, context);
    assembly::ast::Node::AllocTemporary {
        place,
        storage_type,
        operation,
    }
}

fn rule_alloc_temporary_command<'a>() -> RuleApp<'a, assembly::ast::Node> {
    rule_pair(Rule::alloc_temporary_command, read_alloc_temporary_command)
}

fn read_encode_do_args(pairs: &mut Pairs<Rule>, context: &mut Context) -> Vec<Hole<String>> {
    expect_all(rule_var_name(), pairs, context)
}

fn read_encode_do_params(pairs: &mut Pairs<Rule>, context: &mut Context) -> Box<[Hole<String>]> {
    option_to_vec(optional(
        rule_pair(Rule::encode_do_args, read_encode_do_args),
        pairs,
        context,
    ))
    .into_boxed_slice()
}

fn read_encode_do_call(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> (Hole<assembly::ast::RemoteNodeId>, Hole<Box<[Hole<String>]>>) {
    let operation = expect(rule_funclet_loc(), pairs, context);
    let inputs = expect_hole(
        rule_pair(Rule::encode_do_params, read_encode_do_params),
        pairs,
        context,
    );
    (operation, inputs)
}

fn read_encode_do_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    let rule_place = rule_str_unwrap(Rule::encode_do_sep, 1, Box::new(read_place));
    let place = expect(rule_place, pairs, context);
    let rule_call = rule_pair(Rule::encode_do_call, read_encode_do_call);
    let call = expect_hole(rule_call, pairs, context);
    let output = expect(rule_var_name(), pairs, context);
    let operation = call.clone().map(|x| x.0.unwrap()); // this unwrap is safe by parser definition
    let inputs = call.map(|x| x.1.unwrap()); // also safe since no empty input vec for now
    assembly::ast::Node::EncodeDo {
        place,
        operation,
        inputs,
        outputs: Some(Box::new([output])),
    }
}

fn rule_encode_do_command<'a>() -> RuleApp<'a, assembly::ast::Node> {
    rule_pair(Rule::encode_do_command, read_encode_do_command)
}

fn read_create_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    let place = expect(rule_place(), pairs, context);
    let storage_type = expect(rule_type(), pairs, context);
    let operation = expect(rule_funclet_loc(), pairs, context);
    assembly::ast::Node::UnboundSlot {
        place,
        storage_type,
        operation,
    }
}

fn rule_create_command<'a>() -> RuleApp<'a, assembly::ast::Node> {
    rule_pair(Rule::create_command, read_create_command)
}

fn read_drop_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    let node = expect(rule_var_name(), pairs, context);
    assembly::ast::Node::Drop { node }
}

fn rule_drop_command<'a>() -> RuleApp<'a, assembly::ast::Node> {
    rule_pair(Rule::drop_command, read_drop_command)
}

fn read_alloc_sep(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> (Hole<ir::Place>, Hole<assembly::ast::Type>) {
    let place = expect(rule_place(), pairs, context);
    let storage_type = expect(rule_type(), pairs, context);
    (place, storage_type)
}

fn read_alloc_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    let rule = rule_pair(Rule::alloc_sep, read_alloc_sep);
    let (place, storage_type) = expect(rule, pairs, context);
    let buffer = expect(rule_var_name(), pairs, context);
    let operation = expect(rule_funclet_loc(), pairs, context);
    assembly::ast::Node::StaticAllocFromStaticBuffer {
        buffer,
        place,
        storage_type,
        operation,
    }
}

fn rule_alloc_command<'a>() -> RuleApp<'a, assembly::ast::Node> {
    rule_pair(Rule::alloc_command, read_alloc_command)
}

fn read_encode_copy_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    let rule = rule_str_unwrap(Rule::encode_copy_sep, 1, Box::new(read_place));
    let place = expect(rule, pairs, context);
    let input = expect(rule_var_name(), pairs, context);
    let output = expect(rule_var_name(), pairs, context);
    assembly::ast::Node::EncodeCopy {
        place,
        input,
        output,
    }
}

fn rule_encode_copy_command<'a>() -> RuleApp<'a, assembly::ast::Node> {
    rule_pair(Rule::encode_copy_command, read_encode_copy_command)
}

fn read_encode_fence_command(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> assembly::ast::Node {
    let place = expect(rule_place(), pairs, context);
    let event = expect(rule_funclet_loc(), pairs, context);
    assembly::ast::Node::EncodeFence { place, event }
}

fn rule_encode_fence_command<'a>() -> RuleApp<'a, assembly::ast::Node> {
    rule_pair(Rule::encode_fence_command, read_encode_fence_command)
}

fn read_submit_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    let place = expect(rule_place(), pairs, context);
    let event = expect(rule_funclet_loc(), pairs, context);
    assembly::ast::Node::Submit { place, event }
}

fn rule_submit_command<'a>() -> RuleApp<'a, assembly::ast::Node> {
    rule_pair(Rule::submit_command, read_submit_command)
}

fn read_sync_fence_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    let rule = rule_str_unwrap(Rule::sync_fence_sep, 1, Box::new(read_place));
    let place = expect(rule, pairs, context);
    let fence = expect(rule_var_name(), pairs, context);
    let event = expect(rule_funclet_loc(), pairs, context);
    assembly::ast::Node::SyncFence {
        place,
        fence,
        event,
    }
}

fn rule_sync_fence_command<'a>() -> RuleApp<'a, assembly::ast::Node> {
    rule_pair(Rule::sync_fence_command, read_sync_fence_command)
}

fn read_inline_join_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    require_rule(Rule::inline_join_sep, pairs, context);
    let funclet = expect(rule_fn_name(), pairs, context);
    let captures = expect(rule_node_box(), pairs, context).map(|v| v.into_boxed_slice());
    let continuation = expect(rule_var_name(), pairs, context);
    // empty captures re conversation
    assembly::ast::Node::InlineJoin {
        funclet,
        captures,
        continuation,
    }
}

fn rule_inline_join_command<'a>() -> RuleApp<'a, assembly::ast::Node> {
    rule_pair(Rule::inline_join_command, read_inline_join_command)
}

fn read_serialized_join_command(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> assembly::ast::Node {
    require_rule(Rule::serialized_join_sep, pairs, context);
    let funclet = expect(rule_fn_name(), pairs, context);
    let captures = expect(rule_node_box(), pairs, context).map(|v| v.into_boxed_slice());
    let continuation = expect(rule_var_name(), pairs, context);
    assembly::ast::Node::InlineJoin {
        funclet,
        captures,
        continuation,
    }
}

fn rule_serialized_join_command<'a>() -> RuleApp<'a, assembly::ast::Node> {
    rule_pair(Rule::serialized_join_command, read_serialized_join_command)
}

fn read_default_join_command(s: String, context: &mut Context) -> assembly::ast::Node {
    assembly::ast::Node::DefaultJoin
}

fn rule_default_join_command<'a>() -> RuleApp<'a, assembly::ast::Node> {
    rule_str(Rule::default_join_command, read_default_join_command)
}

fn read_schedule_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    let mut rules = Vec::new();
    rules.push(rule_phi_command());
    rules.push(rule_alloc_command());
    rules.push(rule_encode_do_command());
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

fn read_schedule_assign(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> assembly::ast::NamedNode {
    let name = reject_hole(expect(rule_var_name(), pairs, context));
    let rule = rule_pair(Rule::schedule_command, read_schedule_command);
    let node = expect(rule, pairs, context);
    assembly::ast::NamedNode { name, node }
}

fn read_sync_sep(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> (Hole<ir::Place>, Hole<ir::Place>) {
    let place1 = expect(rule_place(), pairs, context);
    let place2 = expect(rule_place(), pairs, context);
    (place1, place2)
}

fn read_sync_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    let rule = rule_pair(Rule::sync_sep, read_sync_sep);
    let (here_place, there_place) = expect(rule, pairs, context);
    let local_past = expect(rule_var_name(), pairs, context);
    let remote_local_past = expect(rule_var_name(), pairs, context);
    assembly::ast::Node::SynchronizationEvent {
        here_place,
        there_place,
        local_past,
        remote_local_past,
    }
}

fn rule_sync_command<'a>() -> RuleApp<'a, assembly::ast::Node> {
    rule_pair(Rule::sync_command, read_sync_command)
}

fn read_submission_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    let here_place = expect(rule_place(), pairs, context);
    let there_place = expect(rule_place(), pairs, context);
    let local_past = expect(rule_var_name(), pairs, context);
    assembly::ast::Node::SubmissionEvent {
        here_place,
        there_place,
        local_past,
    }
}

fn rule_submission_command<'a>() -> RuleApp<'a, assembly::ast::Node> {
    rule_pair(Rule::submission_command, read_submission_command)
}

fn read_timeline_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Node {
    let mut rules = Vec::new();
    rules.push(rule_phi_command());
    rules.push(rule_sync_command());
    rules.push(rule_submission_command());
    expect_vec(rules, pairs, context)
}

fn read_spatial_command(_: &mut Pairs<Rule>, _: &mut Context) -> assembly::ast::Node {
    unimplemented!() // currently invalid
}

fn read_timeline_assign(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> assembly::ast::NamedNode {
    let name = reject_hole(expect(rule_var_name(), pairs, context));
    let rule = rule_pair(Rule::timeline_command, read_timeline_command);
    let node = expect(rule, pairs, context);
    assembly::ast::NamedNode { name, node }
}

fn read_spatial_assign(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::NamedNode {
    let name = reject_hole(expect(rule_var_name(), pairs, context));
    let rule = rule_pair(Rule::spatial_command, read_spatial_command);
    let node = expect(rule, pairs, context);
    assembly::ast::NamedNode { name, node }
}

fn read_funclet_blob(
    kind: ir::FuncletKind,
    rule_command: RuleApp<assembly::ast::NamedNode>,
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> assembly::ast::Funclet {
    let mut commands: Vec<Hole<assembly::ast::NamedNode>> = Vec::new();
    let (header, in_names) = expect(rule_funclet_header(), pairs, context);
    for (index, name) in itertools::enumerate(in_names) {
        let node = assembly::ast::Node::Phi { index: Some(index) };
        commands.push(Some(assembly::ast::NamedNode { name, node }));
    }
    // this gets very silly for checking reasons
    // we both want to check for if we have a tail edge,
    //   _and_ if the last node hole could be a tail edge
    let mut tail: Option<Hole<assembly::ast::TailEdge>> = None;
    for pair in pairs {
        let rule = pair.as_rule();
        if rule == Rule::tail_edge {
            tail = match tail {
                None => Some(Some(read_tail_edge(&mut pair.into_inner(), context))),
                Some(None) => {
                    commands.push(None); // push the "tail edge hole" into the commands list
                    Some(Some(read_tail_edge(&mut pair.into_inner(), context)))
                }
                _ => panic!("Multiple tail edges found for funclet",),
            }
        } else if rule == Rule::node_hole {
            tail = Some(None) // currently the hole
        } else if rule == rule_command.rule {
            tail = match tail {
                None => None,
                // push the "tail edge hole" into the commands list
                Some(None) => {
                    commands.push(None);
                    None
                }
                _ => panic!("Command after tail edge found for funclet",),
            };
            commands.push(match &rule_command.application {
                Application::P(f) => Some(f(&mut pair.into_inner(), context)),
                _ => panic!("Internal error with rules"),
            });
        } else {
            panic!(unexpected_rule(&vec![rule_command], rule));
        }
    }
    match tail {
        Some(tail_edge) => {
            // note that tail_edge can be None (as a hole)
            assembly::ast::Funclet {
                kind,
                header,
                commands,
                tail_edge,
            }
        }
        None => panic!(format!("No tail edge found for funclet",)),
    }
}

fn read_value_funclet(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Funclet {
    let rule_command = rule_pair(Rule::value_assign, read_value_assign);
    read_funclet_blob(ir::FuncletKind::Value, rule_command, pairs, context)
}

fn read_schedule_funclet(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Funclet {
    let rule_command = rule_pair(Rule::schedule_assign, read_schedule_assign);
    read_funclet_blob(
        ir::FuncletKind::ScheduleExplicit,
        rule_command,
        pairs,
        context,
    )
}

fn read_timeline_funclet(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Funclet {
    let rule_command = rule_pair(Rule::timeline_assign, read_timeline_assign);
    read_funclet_blob(ir::FuncletKind::Timeline, rule_command, pairs, context)
}

fn read_spatial_funclet(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Funclet {
    let rule_command = rule_pair(Rule::spatial_assign, read_spatial_assign);
    read_funclet_blob(ir::FuncletKind::Spatial, rule_command, pairs, context)
}

fn read_funclet(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Funclet {
    let rule = pairs.next().unwrap().as_rule();
    let vrule = rule_pair(Rule::value_funclet, read_value_funclet);
    let srule = rule_pair(Rule::schedule_funclet, read_schedule_funclet);
    let trule = rule_pair(Rule::timeline_funclet, read_timeline_funclet);
    let sprule = rule_pair(Rule::spatial_funclet, read_spatial_funclet);
    match rule {
        Rule::value_sep => expect(vrule, pairs, context),
        Rule::schedule_sep => expect(srule, pairs, context),
        Rule::timeline_sep => expect(trule, pairs, context),
        Rule::spatial_sep => expect(sprule, pairs, context),
        _ => panic!(unexpected_rule(&vec![vrule, srule, trule], rule)),
    }
}

fn read_funclet_def(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::FuncletDef {
    let mut rules = Vec::new();
    rules.push(rule_pair(Rule::external_funclet, read_external_funclet));
    let rule = compose_pair(read_funclet, assembly::ast::FuncletDef::Local);
    rules.push(rule_pair_boxed(Rule::funclet, rule));
    let rule = compose_pair(
        read_value_function,
        assembly::ast::FuncletDef::ValueFunction,
    );
    rules.push(rule_pair_boxed(Rule::value_function, rule));
    expect_vec(rules, pairs, context)
}

fn read_value_function_args(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<assembly::ast::Type> {
    let rule = compose_pair(read_type, reject_hole);
    expect_all(rule_pair_boxed(Rule::typ, rule), pairs, context)
}

fn read_value_function_funclets(pairs: &mut Pairs<Rule>, context: &mut Context) -> Vec<String> {
    let rule = compose_pair(read_fn_name, reject_hole);
    expect_all(rule_pair_boxed(Rule::fn_name, rule), pairs, context)
}

fn read_value_function(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> assembly::ast::ValueFunction {
    require_rule(Rule::value_function_sep, pairs, context);
    let name = reject_hole(expect(rule_fn_name(), pairs, context));

    let rule = rule_pair(Rule::value_function_args, read_value_function_args);
    let input_types = expect(rule, pairs, context);

    let output_types = vec![reject_hole(expect(rule_type(), pairs, context))];

    let rule = rule_pair(Rule::value_function_funclets, read_value_function_funclets);
    let allowed_funclets = expect(rule, pairs, context);

    assembly::ast::ValueFunction {
        name,
        input_types,
        output_types,
        allowed_funclets,
    } // todo add syntax
}

fn read_funclets(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::FuncletDefs {
    let rule = rule_pair(Rule::funclet_def, read_funclet_def);
    expect_all(rule, pairs, context)
}

fn read_extra(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> (String, assembly::ast::UncheckedDict) {
    let name = reject_hole(expect(rule_fn_name_sep(), pairs, context));
    let data = expect(rule_unchecked_dict(), pairs, context);
    (name, data)
}

fn read_extras(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Extras {
    let mut result = HashMap::new();
    let extras = expect_all(rule_pair(Rule::extra, read_extra), pairs, context);
    for extra in extras.into_iter() {
        result.insert(extra.0, extra.1);
    }
    result
}

fn read_pipeline(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> (String, assembly::ast::FuncletId) {
    require_rule(Rule::pipeline_sep, pairs, context);
    let name = expect(rule_string_clean(), pairs, context);
    let funclet = reject_hole(expect(rule_fn_name(), pairs, context));
    (name, funclet)
}

fn read_pipelines(pairs: &mut Pairs<Rule>, context: &mut Context) -> assembly::ast::Pipelines {
    let mut result = HashMap::new();
    let extras = expect_all(rule_pair(Rule::pipeline, read_pipeline), pairs, context);
    for extra in extras.into_iter() {
        result.insert(extra.0, extra.1);
    }
    result
}

fn read_program(parsed: &mut Pairs<Rule>) -> assembly::ast::Program {
    let head = parsed.next().unwrap();
    let mut pairs = match head.as_rule() {
        Rule::program => head.into_inner(),
        _ => panic!("CAIR must start with a program"),
    };

    let mut context = Context::new();

    let version = expect(
        rule_pair(Rule::version, read_version),
        &mut pairs,
        &mut context,
    );
    let types = expect(rule_pair(Rule::types, read_types), &mut pairs, &mut context);
    let funclets = expect(
        rule_pair(Rule::funclets, read_funclets),
        &mut pairs,
        &mut context,
    );
    let extras = expect(
        rule_pair(Rule::extras, read_extras),
        &mut pairs,
        &mut context,
    );
    let pipelines = expect(
        rule_pair(Rule::pipelines, read_pipelines),
        &mut pairs,
        &mut context,
    );

    assembly::ast::Program {
        version,
        types,
        funclets,
        extras,
        pipelines,
    }
}

pub fn parse(code: &str) -> assembly::ast::Program {
    let parsed = IRParser::parse(Rule::program, code);
    match parsed {
        Err(why) => panic!("{:?}", why),
        Ok(mut parse_result) => read_program(&mut parse_result),
    }
}

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
    s: &ast::Value,
    data: &ast::UncheckedDict,
) -> ast::Value {
    let err = format!("Expected an entry for for {:?}", s);
    match data.get(s).unwrap_or_else(|| panic!(err.clone())) {
        ast::DictValue::Raw(v) => v.clone(),
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

fn n(s: String, context: &mut Context) -> usize {
    s.parse::<usize>().unwrap()
}

fn rule_n<'a>() -> RuleApp<'a, usize> {
    rule_str(Rule::n, n)
}

fn string(s: String, context: &mut Context) -> String {
    s
}

fn rule_id_raw<'a>() -> RuleApp<'a, String> {
    rule_str(Rule::id, string)
}

fn rule_n_raw<'a>() -> RuleApp<'a, String> {
    rule_str(Rule::n, string)
}

fn rule_string<'a>(rule: Rule) -> RuleApp<'a, String> {
    rule_str(rule, string)
}

fn rule_funclet_id<'a>(rule: Rule) -> RuleApp<'a, FuncletId> {
    rule_str(rule, funclet_id)
}

fn rule_operation_id<'a>(rule: Rule) -> RuleApp<'a, OperationId> {
    rule_str(rule, operation_id)
}

fn string_clean(s: String, context: &mut Context) -> String {
    (&s[1..s.len() - 1]).to_string()
}

fn rule_string_clean<'a>() -> RuleApp<'a, String> {
    rule_str(Rule::str, string_clean)
}

fn type_raw(pairs: &mut Pairs<Rule>, context: &mut Context) -> TypeId {
    let mut rules = Vec::new();
    rules.push(rule_str(Rule::ffi_type, string));
    rules.push(rule_str_unwrap(Rule::type_name, 1, Box::new(string)));
    expect_vec(rules, pairs, context)
}

fn rule_type_raw<'a>() -> RuleApp<'a, String> {
    rule_pair(Rule::typ, type_raw)
}

fn hole<T>(s: String, context: &mut Context) -> Option<T> {
    None
}

fn rule_hole<'a, T: 'a>() -> RuleApp<'a, Option<T>> {
    rule_str(Rule::hole, hole)
}

fn rule_node_hole<'a, T: 'a>() -> RuleApp<'a, Option<T>> {
    rule_str(Rule::node_hole, hole)
}

fn id(s: String, context: &mut Context) -> ast::Value {
    ast::Value::ID(s)
}

fn rule_id<'a>() -> RuleApp<'a, ast::Value> {
    rule_str(Rule::id, id)
}

fn none_value(_: String, _: &mut Context) -> ast::Value {
    ast::Value::None
}

fn ffi_type_base(s: String, context: &mut Context) -> ast::FFIType {
    match s.as_str() {
        "f32" => ast::FFIType::F32,
        "f64" => ast::FFIType::F64,
        "u8" => ast::FFIType::U8,
        "u16" => ast::FFIType::U16,
        "u32" => ast::FFIType::U32,
        "u64" => ast::FFIType::U64,
        "i8" => ast::FFIType::I8,
        "i16" => ast::FFIType::I16,
        "i32" => ast::FFIType::I32,
        "i64" => ast::FFIType::I64,
        "usize" => ast::FFIType::USize,
        "gpu_buffer_allocator" => ast::FFIType::GpuBufferAllocator,
        "cpu_buffer_allocator" => ast::FFIType::CpuBufferAllocator,
        _ => panic!("Unknown type name {}", s),
    }
}

fn ffi_ref_parameter(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> ast::FFIType {
    expect(rule_ffi_type(), pairs, context)
}

fn ffi_array_params(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::FFIType {
    let element_type = Box::new(expect(rule_ffi_type(), pairs, context));
    let length = expect(rule_n(), pairs, context);
    ast::FFIType::Array {
        element_type,
        length,
    }
}

fn ffi_tuple_params(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::FFIType {
    let elements = expect_all(rule_ffi_type(), pairs, context);
    ast::FFIType::Tuple(elements)
}

fn ffi_parameterized_ref_name(
    s: String,
    context: &mut Context,
) -> Box<dyn Fn(ast::FFIType) -> ast::FFIType> {
    fn box_up<F>(f: &'static F) -> Box<dyn Fn(ast::FFIType) -> ast::FFIType>
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
        "cpu_buffer_ref" => box_up(&ast::FFIType::CpuBufferRef),
        _ => panic!("Unknown type name {}", s),
    }
}

fn ffi_parameterized_ref(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> ast::FFIType {
    let rule = rule_str(
        Rule::ffi_parameterized_ref_name,
        ffi_parameterized_ref_name,
    );
    let kind = expect(rule, pairs, context);
    let rule = rule_pair(Rule::ffi_ref_parameter, ffi_ref_parameter);
    let value = expect(rule, pairs, context);
    kind(value)
}

fn ffi_parameterized_type(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> ast::FFIType {
    let mut rules = Vec::new();
    let func = Box::new(ffi_array_params);
    rules.push(rule_pair_unwrap(Rule::ffi_parameterized_array, 1, func));

    rules.push(rule_pair(
        Rule::ffi_parameterized_ref,
        ffi_parameterized_ref,
    ));
    let func = Box::new(ffi_tuple_params);
    rules.push(rule_pair_unwrap(Rule::ffi_parameterized_tuple, 1, func));

    expect_vec(rules, pairs, context)
}

fn ffi_type(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::FFIType {
    let mut rules = Vec::new();
    rules.push(rule_str(Rule::ffi_type_base, ffi_type_base));
    rules.push(rule_pair(
        Rule::ffi_parameterized_type,
        ffi_parameterized_type,
    ));
    expect_vec(rules, pairs, context)
}

fn rule_ffi_type<'a>() -> RuleApp<'a, ast::FFIType> {
    rule_pair(Rule::ffi_type, ffi_type)
}

fn rule_ffi_type_sep<'a>() -> RuleApp<'a, ast::FFIType> {
    rule_pair_unwrap(Rule::ffi_type_sep, 1, Box::new(ffi_type))
}

fn typ(pairs: &mut Pairs<Rule>, context: &mut Context) -> Hole<ast::Type> {
    let mut rules = Vec::new();
    let ffi_fn = compose_pair(ffi_type, ast::Type::FFI);
    let ffi_fn_wrap = compose_pair(ffi_fn, Some);
    rules.push(rule_pair_boxed(Rule::ffi_type, ffi_fn_wrap));
    let rule_ir = compose_str(string, ast::Type::Local);
    let rule_ir_wrap = compose_str(rule_ir, Some);
    rules.push(rule_str_unwrap(Rule::type_name, 1, rule_ir_wrap));
    rules.push(rule_hole());
    expect_vec(rules, pairs, context)
}

fn rule_type<'a>() -> RuleApp<'a, Hole<ast::Type>> {
    rule_pair(Rule::typ, type)
}

fn rule_type_sep<'a>() -> RuleApp<'a, Hole<ast::Type>> {
    rule_pair_unwrap(Rule::typ_sep, 1, Box::new(type))
}

fn throwaway(_: String, context: &mut Context) -> String {
    "_".to_string()
}

fn rule_throwaway<'a>() -> RuleApp<'a, String> {
    rule_str(Rule::throwaway, throwaway)
}

fn var_name(pairs: &mut Pairs<Rule>, context: &mut Context) -> Hole<String> {
    let mut rules = Vec::new();
    let rule = compose_str(string, Some);
    rules.push(rule_str_boxed(Rule::id, rule));
    let rule = compose_str(string, Some);
    rules.push(rule_str_boxed(Rule::n, rule));
    let rule = compose_str(throwaway, Some);
    rules.push(rule_str_boxed(Rule::throwaway, rule));
    rules.push(rule_hole());
    expect_vec(rules, pairs, context)
}

fn rule_var_name<'a>() -> RuleApp<'a, Hole<String>> {
    rule_pair(Rule::var_name, var_name)
}

fn fn_name(pairs: &mut Pairs<Rule>, context: &mut Context) -> Hole<FuncletId> {
    let mut rules = Vec::new();
    let rule = compose_str(funclet_id, Some);
    rules.push(rule_str_boxed(Rule::id, rule));
    rules.push(rule_hole());
    expect_vec(rules, pairs, context)
}

fn rule_fn_name<'a>() -> RuleApp<'a, Hole<FuncletId>> {
    rule_pair(Rule::fn_name, fn_name)
}

fn rule_fn_name_sep<'a>() -> RuleApp<'a, Hole<String>> {
    rule_pair_unwrap(Rule::fn_name_sep, 1, Box::new(fn_name))
}

fn funclet_loc_filled(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> ast::RemoteNodeId {
    let rule_func = rule_str_unwrap(Rule::fn_name, 1, Box::new(string));
    let rule_var = rule_str_unwrap(Rule::var_name, 1, Box::new(string));
    let fun_name = expect(rule_func, pairs, context);
    let var_name = expect(rule_var, pairs, context);
    ast::RemoteNodeId {
        funclet_name: fun_name,
        node_name: var_name,
    }
}

fn funclet_loc(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Hole<ast::RemoteNodeId> {
    expect_hole(
        rule_pair(Rule::funclet_loc_filled, funclet_loc_filled),
        pairs,
        context,
    )
}

fn rule_funclet_loc<'a>() -> RuleApp<'a, Hole<ast::RemoteNodeId>> {
    rule_pair(Rule::funclet_loc, funclet_loc)
}

fn place(s: String, context: &mut Context) -> Hole<ir::Place> {
    match s.as_str() {
        "local" => Some(ir::Place::Local),
        "cpu" => Some(ir::Place::Cpu),
        "gpu" => Some(ir::Place::Gpu),
        "?" => None,
        _ => panic!(unexpected(s)),
    }
}

fn rule_place<'a>() -> RuleApp<'a, Hole<ir::Place>> {
    rule_str(Rule::place, place)
}

fn stage(s: String, context: &mut Context) -> Hole<ir::ResourceQueueStage> {
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
    rule_str(Rule::stage, stage)
}

fn tag_core_op(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::TagCore {
    let op_type = expect(rule_string(Rule::tag_core_op), pairs, context);
    let funclet_loc = expect(rule_funclet_loc(), pairs, context);
    match op_type.as_str() {
        // "operation" | "input" | "output"
        "operation" => ast::TagCore::Operation(reject_hole(funclet_loc)),
        "input" => ast::TagCore::Input(reject_hole(funclet_loc)),
        "output" => ast::TagCore::Output(reject_hole(funclet_loc)),
        _ => panic!(unexpected(op_type)),
    }
}

fn tag_core(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::TagCore {
    let pair = pairs.peek().unwrap();
    let rule = pair.as_rule();
    match rule {
        Rule::none => ast::TagCore::None,
        Rule::tag_core_op => tag_core_op(pairs, context),
        _ => panic!(unexpected_rule_raw(
            vec![Rule::none, Rule::tag_core_op],
            rule,
        )),
    }
}

fn rule_tag_core<'a>() -> RuleApp<'a, ast::TagCore> {
    rule_pair(Rule::tag_core, tag_core)
}

fn value_tag_loc(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::ValueTag {
    let op_type = expect(rule_string(Rule::value_tag_op), pairs, context);
    let funclet_loc = expect(rule_funclet_loc(), pairs, context);
    match op_type.as_str() {
        // "function_input" | "function_output"
        "function_input" => ast::ValueTag::FunctionInput(reject_hole(funclet_loc)),
        "function_output" => ast::ValueTag::FunctionOutput(reject_hole(funclet_loc)),
        _ => panic!(unexpected(op_type)),
    }
}

fn value_tag_data(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::ValueTag {
    let mut rules = vec![];
    let app = compose_pair(tag_core, ast::ValueTag::Core);
    let rule = rule_pair_boxed(Rule::tag_core, app);
    rules.push(rule);

    let rule = rule_pair(Rule::value_tag_loc, value_tag_loc);
    rules.push(rule);

    let app = compose_pair_reject(var_name, ast::ValueTag::Halt);
    let rule = rule_pair_unwrap(Rule::tag_halt, 1, app);
    rules.push(rule);
    expect_vec(rules, pairs, context)
}

fn value_tag(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::ValueTag {
    require_rule(Rule::value_tag_sep, pairs, context);
    let rule = rule_pair(Rule::value_tag_data, value_tag_data);
    expect(rule, pairs, context)
}

fn rule_value_tag<'a>() -> RuleApp<'a, ast::ValueTag> {
    rule_pair(Rule::value_tag, value_tag)
}

fn timeline_tag_data(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> ast::TimelineTag {
    ast::TimelineTag::Core(expect(rule_tag_core(), pairs, context))
}

fn timeline_tag(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::TimelineTag {
    require_rule(Rule::timeline_tag_sep, pairs, context);
    let rule = rule_pair(Rule::timeline_tag_data, timeline_tag_data);
    expect(rule, pairs, context)
}

fn rule_timeline_tag<'a>() -> RuleApp<'a, ast::TimelineTag> {
    rule_pair(Rule::timeline_tag, timeline_tag)
}

fn spatial_tag_data(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> ast::SpatialTag {
    ast::SpatialTag::Core(expect(rule_tag_core(), pairs, context))
}

fn spatial_tag(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::SpatialTag {
    require_rule(Rule::spatial_tag_sep, pairs, context);
    let rule = rule_pair(Rule::spatial_tag_data, spatial_tag_data);
    expect(rule, pairs, context)
}

fn rule_spatial_tag<'a>() -> RuleApp<'a, ast::SpatialTag> {
    rule_pair(Rule::spatial_tag, spatial_tag)
}

fn tag(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Tag {
    let mut rules = vec![];
    let value_app = compose_pair(value_tag, ast::Tag::ValueTag);
    rules.push(rule_pair_boxed(Rule::value_tag, value_app));
    let timeline_app = compose_pair(timeline_tag, ast::Tag::TimelineTag);
    rules.push(rule_pair_boxed(Rule::timeline_tag, timeline_app));
    let spatial_app = compose_pair(spatial_tag, ast::Tag::SpatialTag);
    rules.push(rule_pair_boxed(Rule::spatial_tag, spatial_app));
    expect_vec(rules, pairs, context)
}

fn slot_info(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::SlotInfo {
    let mut rules = vec![];
    rules.push(rule_pair(Rule::tag, tag));
    let tags = expect_all_vec(rules, pairs, context);
    let mut value_tag = ast::ValueTag::Core(ast::TagCore::None);
    let mut timeline_tag = ast::TimelineTag::Core(ast::TagCore::None);
    let mut spatial_tag = ast::SpatialTag::Core(ast::TagCore::None);
    for tag in tags.iter() {
        match tag {
            // duplicates are whatever
            ast::Tag::ValueTag(t) => value_tag = t.clone(),
            ast::Tag::TimelineTag(t) => timeline_tag = t.clone(),
            ast::Tag::SpatialTag(t) => spatial_tag = t.clone(),
        }
    }
    ast::SlotInfo {
        value_tag,
        timeline_tag,
        spatial_tag,
    }
}

fn fence_info(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::FenceInfo {
    let rule = rule_pair(Rule::timeline_tag, timeline_tag);
    match pairs.peek() {
        None => ast::FenceInfo {
            timeline_tag: ast::TimelineTag::Core(ast::TagCore::None),
        },
        Some(_) => ast::FenceInfo {
            timeline_tag: expect(rule, pairs, context),
        },
    }
}

fn buffer_info(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::BufferInfo {
    let rule = rule_pair(Rule::spatial_tag, spatial_tag);
    match pairs.peek() {
        None => ast::BufferInfo {
            spatial_tag: expect(rule, pairs, context),
        },
        Some(_) => ast::BufferInfo {
            spatial_tag: expect(rule, pairs, context),
        },
    }
}

fn value(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Value {
    let mut rules = Vec::new();

    rules.push(rule_str(Rule::none, none_value));
    let rule = compose_str(n, ast::Value::Num);
    rules.push(rule_str_boxed(Rule::n, rule));
    let rule = compose_pair_reject(fn_name, ast::Value::VarName);
    rules.push(rule_pair_boxed(Rule::var_name, rule));
    let rule = compose_pair_reject(funclet_loc, ast::Value::FunctionLoc);
    rules.push(rule_pair_boxed(Rule::funclet_loc, rule));
    let rule = compose_pair_reject(fn_name, ast::Value::FnName);
    rules.push(rule_pair_boxed(Rule::fn_name, rule));
    let rule = compose_pair_reject(type, ast::Value::Type);
    rules.push(rule_pair_boxed(Rule::typ, rule));
    let rule = compose_str_reject(place, ast::Value::Place);
    rules.push(rule_str_boxed(Rule::place, rule));
    let rule = compose_str_reject(stage, ast::Value::Stage);
    rules.push(rule_str_boxed(Rule::stage, rule));
    let rule = compose_pair(tag, ast::Value::Tag);
    rules.push(rule_pair_boxed(Rule::tag, rule));
    let rule = compose_pair(slot_info, ast::Value::SlotInfo);
    rules.push(rule_pair_boxed(Rule::slot_info, rule));
    let rule = compose_pair(fence_info, ast::Value::FenceInfo);
    rules.push(rule_pair_boxed(Rule::fence_info, rule));
    let rule = compose_pair(buffer_info, ast::Value::BufferInfo);
    rules.push(rule_pair_boxed(Rule::buffer_info, rule));

    expect_vec(rules, pairs, context)
}

fn rule_value<'a>() -> RuleApp<'a, ast::Value> {
    rule_pair(Rule::value, value)
}

fn list_values(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<ast::DictValue> {
    let rule = rule_pair(Rule::dict_value, dict_value);
    expect_all(rule, pairs, context)
}

fn list(pairs: &mut Pairs<Rule>, context: &mut Context) -> Vec<ast::DictValue> {
    let rule = rule_pair(Rule::list_values, list_values);
    option_to_vec(optional(rule, pairs, context))
}

fn dict_value(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::DictValue {
    let mut rules = Vec::new();
    let value_map = compose_pair(value, ast::DictValue::Raw);
    rules.push(rule_pair_boxed(Rule::value, value_map));

    let list_map = compose_pair(list, ast::DictValue::List);
    rules.push(rule_pair_boxed(Rule::list, list_map));

    let dict_map = compose_pair(unchecked_dict, ast::DictValue::Dict);
    rules.push(rule_pair_boxed(Rule::unchecked_dict, dict_map));

    expect_vec(rules, pairs, context)
}

fn dict_key(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Value {
    let value_var_name = compose_pair_reject(var_name, ast::Value::VarName);
    let rule_var_name = rule_pair_boxed(Rule::var_name, value_var_name);
    expect_vec(vec![rule_id(), rule_var_name], pairs, context)
}

struct DictPair {
    key: ast::Value,
    value: ast::DictValue,
}

fn dict_element(pairs: &mut Pairs<Rule>, context: &mut Context) -> DictPair {
    let rule_key = rule_pair(Rule::dict_key, dict_key);
    let rule_value = rule_pair(Rule::dict_value, dict_value);
    let key = expect(rule_key, pairs, context);
    let value = expect(rule_value, pairs, context);
    DictPair { key, value }
}

fn unchecked_dict(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> ast::UncheckedDict {
    let rule = rule_pair(Rule::dict_element, dict_element);
    let mut result = HashMap::new();
    for pair in expect_all(rule, pairs, context) {
        result.insert(pair.key, pair.value);
    }
    result
}

fn rule_unchecked_dict<'a>() -> RuleApp<'a, ast::UncheckedDict> {
    rule_pair(Rule::unchecked_dict, unchecked_dict)
}

// Readers

fn version(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Version {
    require_rule(Rule::version_keyword, pairs, context);
    let major_s = expect(rule_n_raw(), pairs, context);
    let minor_s = expect(rule_n_raw(), pairs, context);
    let detailed_s = expect(rule_n_raw(), pairs, context);

    let major = major_s.parse::<u32>().unwrap();
    let minor = minor_s.parse::<u32>().unwrap();
    let detailed = detailed_s.parse::<u32>().unwrap();

    ast::Version {
        major,
        minor,
        detailed,
    }
}

fn interpret_ir_dict(
    s: String,
    data: ast::UncheckedDict,
) -> ast::LocalTypeInfo {
    match s.as_str() {
        "native_value" => {
            let storage_type =
                match unchecked_value(&ast::Value::ID("type".to_string()), &data) {
                    ast::Value::Type(t) => t,
                    v => panic!("Unsupported storage type {:?}", v),
                };
            ast::LocalTypeInfo::NativeValue { storage_type }
        }
        "slot" => {
            let storage_type =
                match unchecked_value(&ast::Value::ID("type".to_string()), &data) {
                    ast::Value::Type(t) => t,
                    v => panic!("Unsupported storage type {:?}", v),
                };
            let queue_stage =
                match unchecked_value(&ast::Value::ID("stage".to_string()), &data) {
                    ast::Value::Stage(t) => t,
                    v => panic!("Unsupported queue stage {:?}", v),
                };
            let queue_place =
                match unchecked_value(&ast::Value::ID("place".to_string()), &data) {
                    ast::Value::Place(t) => t,
                    v => panic!("Unsupported queue place {:?}", v),
                };
            ast::LocalTypeInfo::Slot {
                storage_type,
                queue_stage,
                queue_place,
            }
        }
        "fence" => {
            let queue_place =
                match unchecked_value(&ast::Value::ID("place".to_string()), &data) {
                    ast::Value::Place(t) => t,
                    v => panic!("Unsupported queue place {:?}", v),
                };
            ast::LocalTypeInfo::Fence { queue_place }
        }
        "buffer" => {
            let storage_place =
                match unchecked_value(&ast::Value::ID("place".to_string()), &data) {
                    ast::Value::Place(t) => t,
                    v => panic!("Unsupported storage place {:?}", v),
                };
            let static_layout_opt = data
                .get(&ast::Value::ID("static_layout_opt".to_string()))
                .and_then(|v| match v {
                    ast::DictValue::Dict(d) => {
                        let alignment_bits = match unchecked_value(
                            &ast::Value::ID("alignment_bits".to_string()),
                            &d,
                        ) {
                            ast::Value::Num(n) => n,
                            v => panic!("Unsupported alignment bits {:?}", v),
                        };
                        let byte_size = match unchecked_value(
                            &ast::Value::ID("byte_size".to_string()),
                            &d,
                        ) {
                            ast::Value::Num(n) => n,
                            v => panic!("Unsupported byte size {:?}", v),
                        };
                        Some(ir::StaticBufferLayout {
                            alignment_bits,
                            byte_size,
                        })
                    }
                    _ => panic!("Unsupported static layout opt {:?}", v),
                });
            ast::LocalTypeInfo::Buffer {
                storage_place,
                static_layout_opt,
            }
        }
        "space_buffer" => ast::LocalTypeInfo::BufferSpace,
        "event" => {
            let place = match unchecked_value(&ast::Value::ID("place".to_string()), &data)
            {
                ast::Value::Place(t) => t,
                v => panic!("Unsupported place {:?}", v),
            };
            ast::LocalTypeInfo::Event { place }
        }
        _ => panic!("Unexpected slot check {:?}", s),
    }
}

fn ir_type_decl(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::TypeDecl {
    let event_rule = rule_str_unwrap(Rule::ir_type_decl_key_sep, 1, Box::new(string));
    let type_kind = expect(event_rule, pairs, context);
    let name_rule = rule_str_unwrap(Rule::type_name, 1, Box::new(string));
    let name = expect(name_rule, pairs, context);
    let unchecked_dict = expect(rule_unchecked_dict(), pairs, context);
    let data = interpret_ir_dict(type_kind, unchecked_dict);
    let result = ast::LocalType { name, data };
    ast::TypeDecl::Local(result)
}

fn ffi_type_decl(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::TypeDecl {
    let ffi_typ = expect(rule_ffi_type(), pairs, context);
    ast::TypeDecl::FFI(ffi_typ)
}

fn type_def(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::TypeDecl {
    let mut rules = Vec::new();
    rules.push(rule_pair(Rule::ffi_type_decl, ffi_type_decl));
    rules.push(rule_pair(Rule::ir_type_decl, ir_type_decl));
    expect_vec(rules, pairs, context)
}

fn types(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Types {
    let rule = rule_pair(Rule::type_def, type_def);
    expect_all(rule, pairs, context)
}

fn external_cpu_args(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<ast::FFIType> {
    expect_all(rule_ffi_type(), pairs, context)
}

fn external_cpu_return_args(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<ast::FFIType> {
    let mut rules = Vec::new();
    rules.push(rule_pair(Rule::external_cpu_args, external_cpu_args));
    rules.push(rule_pair_boxed(
        Rule::ffi_type,
        compose_pair(ffi_type, |t| vec![t]),
    ));
    expect_vec(rules, pairs, context)
}

fn external_cpu_funclet(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> ast::ExternalCpuFunction {
    require_rule(Rule::external_cpu_sep, pairs, context);

    let name = reject_hole(expect(rule_fn_name(), pairs, context));

    let rule_extern_args = rule_pair(Rule::external_cpu_args, external_cpu_args);
    let input_types = expect(rule_extern_args, pairs, context);

    let rule_extern_return_args = rule_pair(
        Rule::external_cpu_return_args,
        external_cpu_return_args,
    );
    let output_types = expect(rule_extern_return_args, pairs, context);
    ast::ExternalCpuFunction {
        name,
        input_types,
        output_types,
    }
}

fn external_gpu_arg(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> (ast::FFIType, String) {
    let name = reject_hole(expect(rule_var_name(), pairs, context));
    let typ = expect(rule_ffi_type(), pairs, context);
    (typ, name)
}

fn external_gpu_args(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<(ast::FFIType, String)> {
    let rule = rule_pair(Rule::external_gpu_arg, external_gpu_arg);
    expect_all(rule, pairs, context)
}

fn external_gpu_body(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<ast::UncheckedDict> {
    let rule = rule_pair_unwrap(
        Rule::external_gpu_resource,
        1,
        Box::new(unchecked_dict),
    );
    expect_all(rule, pairs, context)
}

fn interpret_external_gpu_dict(
    data: ast::UncheckedDict,
) -> ast::ExternalGpuFunctionResourceBinding {
    let group = match unchecked_value(&ast::Value::ID("group".to_string()), &data) {
        ast::Value::Num(n) => n,
        v => panic!("Unsupported group {:?}", v),
    };
    let binding = match unchecked_value(&ast::Value::ID("binding".to_string()), &data) {
        ast::Value::Num(n) => n,
        v => panic!("Unsupported binding {:?}", v),
    };
    let input = data
        .get(&ast::Value::ID("input".to_string()))
        .and_then(|v| match v {
            ast::DictValue::Raw(ast::Value::VarName(t)) => Some(t.clone()),
            v => panic!("Unsupported input {:?}", v),
        });
    let output = data
        .get(&ast::Value::ID("output".to_string()))
        .and_then(|v| match v {
            ast::DictValue::Raw(ast::Value::VarName(t)) => Some(t.clone()),
            v => panic!("Unsupported output {:?}", v),
        });
    ast::ExternalGpuFunctionResourceBinding {
        group,
        binding,
        input,
        output,
    }
}

fn external_gpu_funclet(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> ast::ExternalGpuFunction {
    require_rule(Rule::external_gpu_sep, pairs, context);

    let name = reject_hole(expect(rule_fn_name(), pairs, context));

    let rule_extern_args = rule_pair(Rule::external_gpu_args, external_gpu_args);
    let input_args = expect(rule_extern_args, pairs, context);

    let rule_extern_args = rule_pair(Rule::external_gpu_args, external_gpu_args);
    let output_types = expect(rule_extern_args, pairs, context);

    let shader_module = expect(rule_string_clean(), pairs, context);

    let rule_binding = rule_pair(Rule::external_gpu_body, external_gpu_body);
    let unchecked_bindings = expect(rule_binding, pairs, context);

    let resource_bindings = unchecked_bindings
        .into_iter()
        .map(|d| interpret_external_gpu_dict(d))
        .collect();

    ast::ExternalGpuFunction {
        name,
        input_args,
        output_types,
        shader_module,
        entry_point: "main".to_string(), // todo: uhhhh, allow syntax perhaps
        resource_bindings,
    }
}

fn external_funclet(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> ast::FuncletDef {
    let mut rules = Vec::new();
    let comp = compose_pair(
        external_cpu_funclet,
        ast::FuncletDef::ExternalCPU,
    );
    rules.push(rule_pair_boxed(Rule::external_cpu, comp));
    let comp = compose_pair(
        external_gpu_funclet,
        ast::FuncletDef::ExternalGPU,
    );
    rules.push(rule_pair_boxed(Rule::external_gpu, comp));
    expect_vec(rules, pairs, context)
}

fn funclet_arg(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> (Option<String>, ast::Type) {
    let pair = pairs.next().unwrap();
    let rule = pair.as_rule();
    match rule {
        Rule::var_name => {
            // You gotta add the phi node when translating IRs when you do this!
            let var = reject_hole(var_name(&mut pair.into_inner(), context));
            let typ = reject_hole(expect(rule_type(), pairs, context));
            (Some(var), typ)
        }
        Rule::typ => (
            None,
            reject_hole(type(&mut pair.into_inner(), context)),
        ),
        _ => panic!(unexpected_rule_raw(vec![Rule::var_name, Rule::typ], rule)),
    }
}

fn funclet_args(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<(Option<String>, ast::Type)> {
    let rule = rule_pair(Rule::funclet_arg, funclet_arg);
    expect_all(rule, pairs, context)
}

fn funclet_return_arg(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> (Option<String>, ast::Type) {
    let pair = pairs.next().unwrap();
    let rule = pair.as_rule();
    match rule {
        Rule::var_name => {
            // You gotta add the phi node when translating IRs when you do this!
            let var = reject_hole(var_name(&mut pair.into_inner(), context));
            let typ = reject_hole(expect(rule_type(), pairs, context));
            (Some(var), typ)
        }
        Rule::typ => (
            None,
            reject_hole(type(&mut pair.into_inner(), context)),
        ),
        _ => panic!(unexpected_rule_raw(vec![Rule::var_name, Rule::typ], rule)),
    }
}

fn funclet_return_args(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<(Option<String>, ast::Type)> {
    let rule = rule_pair(Rule::funclet_arg, funclet_return_arg);
    expect_all(rule, pairs, context)
}

fn funclet_return(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<(Option<String>, ast::Type)> {
    let mut rules = Vec::new();
    rules.push(rule_pair(Rule::funclet_args, funclet_return_args));
    rules.push(rule_pair_boxed(
        Rule::typ,
        compose_pair_reject(type, |t| vec![(None, t)]),
    ));
    expect_vec(rules, pairs, context)
}

fn funclet_header(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> (ast::FuncletHeader, Vec<String>) {
    let name = reject_hole(expect(rule_fn_name(), pairs, context));

    let rule_args = rule_pair(Rule::funclet_args, funclet_args);
    let named_args = option_to_vec(optional(rule_args, pairs, context));
    let args = named_args.clone().into_iter().map(|x| x.1).collect();
    let names = named_args
        .into_iter()
        .map(|x| x.0.unwrap_or("_".to_string()))
        .collect();

    let rule_return = rule_pair(Rule::funclet_return, funclet_return);
    let ret = expect(rule_return, pairs, context);
    (ast::FuncletHeader { ret, name, args }, names)
}

fn rule_funclet_header<'a>() -> RuleApp<'a, (ast::FuncletHeader, Vec<String>)> {
    rule_pair(Rule::funclet_header, funclet_header)
}

fn var_assign(pairs: &mut Pairs<Rule>, context: &mut Context) -> String {
    let var = reject_hole(expect(rule_var_name(), pairs, context));
    var
}

fn node_list(pairs: &mut Pairs<Rule>, context: &mut Context) -> Vec<Hole<String>> {
    expect_all(rule_var_name(), pairs, context)
}

fn node_box_raw(pairs: &mut Pairs<Rule>, context: &mut Context) -> Vec<Hole<String>> {
    match pairs.peek() {
        None => {
            vec![]
        }
        Some(_) => {
            let rule = rule_pair(Rule::node_list, node_list);
            expect(rule, pairs, context)
        }
    }
}

fn node_box(pairs: &mut Pairs<Rule>, context: &mut Context) -> Hole<Vec<Hole<String>>> {
    expect_hole(
        rule_pair(Rule::node_box_raw, node_box_raw),
        pairs,
        context,
    )
}

fn rule_node_box<'a>() -> RuleApp<'a, Hole<Vec<Hole<String>>>> {
    rule_pair(Rule::node_box, node_box)
}

fn return_args(pairs: &mut Pairs<Rule>, context: &mut Context) -> Vec<Hole<String>> {
    expect_all(rule_var_name(), pairs, context)
}

fn return_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::TailEdge {
    require_rule(Rule::return_sep, pairs, context);
    let rule = rule_pair(Rule::return_args, return_args);
    let return_values = expect_node_hole(rule, pairs, context);
    ast::TailEdge::Return { return_values }
}

fn rule_return_command<'a>() -> RuleApp<'a, ast::TailEdge> {
    rule_pair(Rule::return_command, return_command)
}

fn yield_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::TailEdge {
    require_rule(Rule::yield_sep, pairs, context);
    let point_id_hole = expect_hole(rule_n(), pairs, context);
    let pipeline_yield_point_id = point_id_hole.map(ir::PipelineYieldPointId);
    let yielded_nodes = expect(rule_node_box(), pairs, context);
    let next_funclet = expect(rule_fn_name(), pairs, context);
    let continuation_join = expect(rule_var_name(), pairs, context);
    let arguments = expect(rule_node_box(), pairs, context);
    ast::TailEdge::Yield {
        pipeline_yield_point_id,
        yielded_nodes,
        next_funclet,
        continuation_join,
        arguments,
    }
}

fn rule_yield_command<'a>() -> RuleApp<'a, ast::TailEdge> {
    rule_pair(Rule::yield_command, yield_command)
}

fn jump_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::TailEdge {
    require_rule(Rule::jump_sep, pairs, context);
    let join = expect(rule_var_name(), pairs, context);
    let arguments = expect(rule_node_box(), pairs, context);
    ast::TailEdge::Jump { join, arguments }
}

fn rule_jump_command<'a>() -> RuleApp<'a, ast::TailEdge> {
    rule_pair(Rule::jump_command, jump_command)
}

fn schedule_call_command(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> ast::TailEdge {
    require_rule(Rule::schedule_call_sep, pairs, context);
    let value_operation = expect(rule_funclet_loc(), pairs, context);
    let callee_funclet_id = expect(rule_fn_name(), pairs, context);
    let callee_arguments = expect(rule_node_box(), pairs, context);
    let continuation_join = expect(rule_var_name(), pairs, context);
    ast::TailEdge::ScheduleCall {
        value_operation,
        callee_funclet_id,
        callee_arguments,
        continuation_join,
    }
}

fn rule_schedule_call_command<'a>() -> RuleApp<'a, ast::TailEdge> {
    rule_pair(Rule::schedule_call_command, schedule_call_command)
}

fn tail_fn_nodes(pairs: &mut Pairs<Rule>, context: &mut Context) -> Vec<Hole<String>> {
    expect_all(rule_fn_name(), pairs, context)
}

fn tail_fn_box_raw(pairs: &mut Pairs<Rule>, context: &mut Context) -> Vec<Hole<String>> {
    match pairs.peek() {
        None => {
            vec![]
        }
        Some(_) => {
            let rule = rule_pair(Rule::tail_fn_nodes, tail_fn_nodes);
            expect(rule, pairs, context)
        }
    }
}

fn tail_fn_box(pairs: &mut Pairs<Rule>, context: &mut Context) -> Hole<Vec<Hole<String>>> {
    expect_hole(
        rule_pair(Rule::tail_fn_box_raw, tail_fn_box_raw),
        pairs,
        context,
    )
}

fn rule_tail_fn_box<'a>() -> RuleApp<'a, Hole<Vec<Hole<String>>>> {
    rule_pair(Rule::tail_fn_box, tail_fn_box)
}

fn schedule_select_command(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> ast::TailEdge {
    require_rule(Rule::schedule_select_sep, pairs, context);
    let value_operation = expect(rule_funclet_loc(), pairs, context);
    let condition = expect(rule_var_name(), pairs, context);
    let callee_funclet_ids = expect(rule_tail_fn_box(), pairs, context);
    let callee_arguments = expect(rule_node_box(), pairs, context);
    let continuation_join = expect(rule_var_name(), pairs, context);
    ast::TailEdge::ScheduleSelect {
        value_operation,
        condition,
        callee_funclet_ids,
        callee_arguments,
        continuation_join,
    }
}

fn rule_schedule_select_command<'a>() -> RuleApp<'a, ast::TailEdge> {
    rule_pair(Rule::schedule_select_command, schedule_select_command)
}

fn tail_none(_: String, _: &mut Context) -> Option<Hole<String>> {
    None
}

fn tail_option_node(pairs: &mut Pairs<Rule>, context: &mut Context) -> Option<Hole<String>> {
    let mut rules = Vec::new();
    let apply_some = compose_pair(var_name, Some);
    rules.push(rule_pair_boxed(Rule::var_name, apply_some));
    rules.push(rule_str(Rule::none, tail_none));
    expect_vec(rules, pairs, context)
}

fn tail_option_nodes(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<Option<Hole<String>>> {
    let rule = rule_pair(Rule::tail_option_node, tail_option_node);
    expect_all(rule, pairs, context)
}

fn tail_option_box_raw(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<Option<Hole<String>>> {
    match pairs.peek() {
        None => {
            vec![]
        }
        Some(_) => {
            let rule = rule_pair(Rule::tail_option_nodes, tail_option_nodes);
            expect(rule, pairs, context)
        }
    }
}

fn tail_option_box(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Hole<Vec<Option<Hole<String>>>> {
    expect_hole(
        rule_pair(Rule::tail_option_box_raw, tail_option_box_raw),
        pairs,
        context,
    )
}

fn rule_tail_option_box<'a>() -> RuleApp<'a, Hole<Vec<Option<Hole<String>>>>> {
    rule_pair(Rule::tail_option_box, tail_option_box)
}

fn dynamic_alloc_command(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> ast::TailEdge {
    require_rule(Rule::dynamic_alloc_sep, pairs, context);
    let buffer = expect(rule_var_name(), pairs, context);
    let arguments = expect(rule_node_box(), pairs, context);
    let dynamic_allocation_size_slots = expect(rule_tail_option_box(), pairs, context);
    let success_funclet_id = expect(rule_fn_name(), pairs, context);
    let failure_funclet_id = expect(rule_fn_name(), pairs, context);
    let continuation_join = expect(rule_var_name(), pairs, context);
    ast::TailEdge::DynamicAllocFromBuffer {
        buffer,
        arguments,
        dynamic_allocation_size_slots,
        success_funclet_id,
        failure_funclet_id,
        continuation_join,
    }
}

fn rule_dynamic_alloc_command<'a>() -> RuleApp<'a, ast::TailEdge> {
    rule_pair(Rule::dynamic_alloc_command, dynamic_alloc_command)
}

fn tail_edge(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::TailEdge {
    let mut rules = Vec::new();
    rules.push(rule_return_command());
    rules.push(rule_yield_command());
    rules.push(rule_jump_command());
    rules.push(rule_schedule_call_command());
    rules.push(rule_schedule_select_command());
    rules.push(rule_dynamic_alloc_command());
    expect_vec(rules, pairs, context)
}

fn phi_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
    let index = expect_hole(rule_n(), pairs, context);
    ast::Node::Phi { index }
}

fn rule_phi_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::phi_command, phi_command)
}

fn constant_raw(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
    let value = Some(expect(rule_n_raw(), pairs, context));
    let type_id = Some(ast::Type::FFI(expect(
        rule_ffi_type(),
        pairs,
        context,
    )));

    ast::Node::Constant { value, type_id }
}

fn constant_hole(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
    ast::Node::Constant {
        value: None,
        type_id: None,
    }
}

fn constant(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
    let mut rules = Vec::new();
    rules.push(rule_pair(Rule::constant_raw, constant_raw));
    rules.push(rule_pair(Rule::hole, constant_hole));
    expect_vec(rules, pairs, context)
}

fn constant_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
    let rule = rule_pair(Rule::constant, constant);
    expect(rule, pairs, context)
}

fn rule_constant_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::constant_command, constant_command)
}

fn extract_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
    require_rule(Rule::extract_sep, pairs, context);
    let node_id = expect(rule_var_name(), pairs, context);
    let index = Some(expect(rule_n(), pairs, context));
    ast::Node::ExtractResult { node_id, index }
}

fn rule_extract_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::extract_command, extract_command)
}

fn call_args(pairs: &mut Pairs<Rule>, context: &mut Context) -> Vec<Hole<String>> {
    expect_all(rule_var_name(), pairs, context)
}

fn call_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
    require_rule(Rule::call_sep, pairs, context);
    let external_function_id = expect(rule_fn_name(), pairs, context);
    let rule = rule_pair(Rule::call_args, call_args);
    let args = expect_hole(rule, pairs, context).map(|x| x.into_boxed_slice());
    match pairs.peek() {
        None => {
            // NOTE: semi-arbitrary choice for unification
            ast::Node::CallExternalCpu {
                external_function_id,
                arguments: args,
            }
        }
        Some(_) => {
            let rule = rule_pair(Rule::call_args, call_args);
            let arguments = expect_hole(rule, pairs, context).map(|x| x.into_boxed_slice());
            ast::Node::CallExternalGpuCompute {
                external_function_id,
                arguments,
                dimensions: args,
            }
        }
    }
}

fn rule_call_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::call_command, call_command)
}

fn select_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
    require_rule(Rule::select_sep, pairs, context);
    let condition = expect(rule_var_name(), pairs, context);
    let true_case = expect(rule_var_name(), pairs, context);
    let false_case = expect(rule_var_name(), pairs, context);
    ast::Node::Select {
        condition,
        true_case,
        false_case,
    }
}

fn rule_select_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::select_command, select_command)
}

fn value_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
    let mut rules = Vec::new();
    rules.push(rule_phi_command());
    rules.push(rule_constant_command());
    rules.push(rule_extract_command());
    rules.push(rule_call_command());
    rules.push(rule_select_command());
    expect_vec(rules, pairs, context)
}

fn value_assign(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::NamedNode {
    let name = reject_hole(expect(rule_var_name(), pairs, context));
    let rule = rule_pair(Rule::value_command, value_command);
    let node = expect(rule, pairs, context);
    ast::NamedNode { name, node }
}

fn alloc_temporary_command(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> ast::Node {
    let place = expect(rule_place(), pairs, context);
    let storage_type = expect(rule_type(), pairs, context);
    let operation = expect(rule_funclet_loc(), pairs, context);
    ast::Node::AllocTemporary {
        place,
        storage_type,
        operation,
    }
}

fn rule_alloc_temporary_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::alloc_temporary_command, alloc_temporary_command)
}

fn encode_do_args(pairs: &mut Pairs<Rule>, context: &mut Context) -> Vec<Hole<String>> {
    expect_all(rule_var_name(), pairs, context)
}

fn encode_do_params(pairs: &mut Pairs<Rule>, context: &mut Context) -> Box<[Hole<String>]> {
    option_to_vec(optional(
        rule_pair(Rule::encode_do_args, encode_do_args),
        pairs,
        context,
    ))
        .into_boxed_slice()
}

fn encode_do_call(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> (Hole<ast::RemoteNodeId>, Hole<Box<[Hole<String>]>>) {
    let operation = expect(rule_funclet_loc(), pairs, context);
    let inputs = expect_hole(
        rule_pair(Rule::encode_do_params, encode_do_params),
        pairs,
        context,
    );
    (operation, inputs)
}

fn encode_do_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
    let rule_place = rule_str_unwrap(Rule::encode_do_sep, 1, Box::new(place));
    let place = expect(rule_place, pairs, context);
    let rule_call = rule_pair(Rule::encode_do_call, encode_do_call);
    let call = expect_hole(rule_call, pairs, context);
    let output = expect(rule_var_name(), pairs, context);
    let operation = call.clone().map(|x| x.0.unwrap()); // this unwrap is safe by parser definition
    let inputs = call.map(|x| x.1.unwrap()); // also safe since no empty input vec for now
    ast::Node::EncodeDo {
        place,
        operation,
        inputs,
        outputs: Some(Box::new([output])),
    }
}

fn rule_encode_do_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::encode_do_command, encode_do_command)
}

fn create_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
    let place = expect(rule_place(), pairs, context);
    let storage_type = expect(rule_type(), pairs, context);
    let operation = expect(rule_funclet_loc(), pairs, context);
    ast::Node::UnboundSlot {
        place,
        storage_type,
        operation,
    }
}

fn rule_create_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::create_command, create_command)
}

fn drop_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
    let node = expect(rule_var_name(), pairs, context);
    ast::Node::Drop { node }
}

fn rule_drop_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::drop_command, drop_command)
}

fn alloc_sep(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> (Hole<ir::Place>, Hole<ast::Type>) {
    let place = expect(rule_place(), pairs, context);
    let storage_type = expect(rule_type(), pairs, context);
    (place, storage_type)
}

fn alloc_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
    let rule = rule_pair(Rule::alloc_sep, alloc_sep);
    let (place, storage_type) = expect(rule, pairs, context);
    let buffer = expect(rule_var_name(), pairs, context);
    let operation = expect(rule_funclet_loc(), pairs, context);
    ast::Node::StaticAllocFromStaticBuffer {
        buffer,
        place,
        storage_type,
        operation,
    }
}

fn rule_alloc_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::alloc_command, alloc_command)
}

fn encode_copy_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
    let rule = rule_str_unwrap(Rule::encode_copy_sep, 1, Box::new(place));
    let place = expect(rule, pairs, context);
    let input = expect(rule_var_name(), pairs, context);
    let output = expect(rule_var_name(), pairs, context);
    ast::Node::EncodeCopy {
        place,
        input,
        output,
    }
}

fn rule_encode_copy_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::encode_copy_command, encode_copy_command)
}

fn encode_fence_command(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> ast::Node {
    let place = expect(rule_place(), pairs, context);
    let event = expect(rule_funclet_loc(), pairs, context);
    ast::Node::EncodeFence { place, event }
}

fn rule_encode_fence_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::encode_fence_command, encode_fence_command)
}

fn submit_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
    let place = expect(rule_place(), pairs, context);
    let event = expect(rule_funclet_loc(), pairs, context);
    ast::Node::Submit { place, event }
}

fn rule_submit_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::submit_command, submit_command)
}

fn sync_fence_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
    let rule = rule_str_unwrap(Rule::sync_fence_sep, 1, Box::new(place));
    let place = expect(rule, pairs, context);
    let fence = expect(rule_var_name(), pairs, context);
    let event = expect(rule_funclet_loc(), pairs, context);
    ast::Node::SyncFence {
        place,
        fence,
        event,
    }
}

fn rule_sync_fence_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::sync_fence_command, sync_fence_command)
}

fn inline_join_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
    require_rule(Rule::inline_join_sep, pairs, context);
    let funclet = expect(rule_fn_name(), pairs, context);
    let captures = expect(rule_node_box(), pairs, context).map(|v| v.into_boxed_slice());
    let continuation = expect(rule_var_name(), pairs, context);
    // empty captures re conversation
    ast::Node::InlineJoin {
        funclet,
        captures,
        continuation,
    }
}

fn rule_inline_join_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::inline_join_command, inline_join_command)
}

fn serialized_join_command(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> ast::Node {
    require_rule(Rule::serialized_join_sep, pairs, context);
    let funclet = expect(rule_fn_name(), pairs, context);
    let captures = expect(rule_node_box(), pairs, context).map(|v| v.into_boxed_slice());
    let continuation = expect(rule_var_name(), pairs, context);
    ast::Node::InlineJoin {
        funclet,
        captures,
        continuation,
    }
}

fn rule_serialized_join_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::serialized_join_command, serialized_join_command)
}

fn default_join_command(s: String, context: &mut Context) -> ast::Node {
    ast::Node::DefaultJoin
}

fn rule_default_join_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_str(Rule::default_join_command, default_join_command)
}

fn schedule_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
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

fn schedule_assign(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> ast::NamedNode {
    let name = reject_hole(expect(rule_var_name(), pairs, context));
    let rule = rule_pair(Rule::schedule_command, schedule_command);
    let node = expect(rule, pairs, context);
    ast::NamedNode { name, node }
}

fn sync_sep(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> (Hole<ir::Place>, Hole<ir::Place>) {
    let place1 = expect(rule_place(), pairs, context);
    let place2 = expect(rule_place(), pairs, context);
    (place1, place2)
}

fn sync_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
    let rule = rule_pair(Rule::sync_sep, sync_sep);
    let (here_place, there_place) = expect(rule, pairs, context);
    let local_past = expect(rule_var_name(), pairs, context);
    let remote_local_past = expect(rule_var_name(), pairs, context);
    ast::Node::SynchronizationEvent {
        here_place,
        there_place,
        local_past,
        remote_local_past,
    }
}

fn rule_sync_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::sync_command, sync_command)
}

fn submission_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
    let here_place = expect(rule_place(), pairs, context);
    let there_place = expect(rule_place(), pairs, context);
    let local_past = expect(rule_var_name(), pairs, context);
    ast::Node::SubmissionEvent {
        here_place,
        there_place,
        local_past,
    }
}

fn rule_submission_command<'a>() -> RuleApp<'a, ast::Node> {
    rule_pair(Rule::submission_command, submission_command)
}

fn timeline_command(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Node {
    let mut rules = Vec::new();
    rules.push(rule_phi_command());
    rules.push(rule_sync_command());
    rules.push(rule_submission_command());
    expect_vec(rules, pairs, context)
}

fn spatial_command(_: &mut Pairs<Rule>, _: &mut Context) -> ast::Node {
    unimplemented!() // currently invalid
}

fn timeline_assign(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> ast::NamedNode {
    let name = reject_hole(expect(rule_var_name(), pairs, context));
    let rule = rule_pair(Rule::timeline_command, timeline_command);
    let node = expect(rule, pairs, context);
    ast::NamedNode { name, node }
}

fn spatial_assign(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::NamedNode {
    let name = reject_hole(expect(rule_var_name(), pairs, context));
    let rule = rule_pair(Rule::spatial_command, spatial_command);
    let node = expect(rule, pairs, context);
    ast::NamedNode { name, node }
}

fn funclet_blob(
    kind: ir::FuncletKind,
    rule_command: RuleApp<ast::NamedNode>,
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> ast::Funclet {
    let mut commands: Vec<Hole<ast::NamedNode>> = Vec::new();
    let (header, in_names) = expect(rule_funclet_header(), pairs, context);
    for (index, name) in itertools::enumerate(in_names) {
        let node = ast::Node::Phi { index: Some(index) };
        commands.push(Some(ast::NamedNode { name, node }));
    }
    // this gets very silly for checking reasons
    // we both want to check for if we have a tail edge,
    //   _and_ if the last node hole could be a tail edge
    let mut tail: Option<Hole<ast::TailEdge>> = None;
    for pair in pairs {
        let rule = pair.as_rule();
        if rule == Rule::tail_edge {
            tail = match tail {
                None => Some(Some(tail_edge(&mut pair.into_inner(), context))),
                Some(None) => {
                    commands.push(None); // push the "tail edge hole" into the commands list
                    Some(Some(tail_edge(&mut pair.into_inner(), context)))
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
            ast::Funclet {
                kind,
                header,
                commands,
                tail_edge,
            }
        }
        None => panic!(format!("No tail edge found for funclet",)),
    }
}

fn value_funclet(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Funclet {
    let rule_command = rule_pair(Rule::value_assign, value_assign);
    funclet_blob(ir::FuncletKind::Value, rule_command, pairs, context)
}

fn schedule_funclet(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Funclet {
    let rule_command = rule_pair(Rule::schedule_assign, schedule_assign);
    funclet_blob(
        ir::FuncletKind::ScheduleExplicit,
        rule_command,
        pairs,
        context,
    )
}

fn timeline_funclet(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Funclet {
    let rule_command = rule_pair(Rule::timeline_assign, timeline_assign);
    funclet_blob(ir::FuncletKind::Timeline, rule_command, pairs, context)
}

fn spatial_funclet(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Funclet {
    let rule_command = rule_pair(Rule::spatial_assign, spatial_assign);
    funclet_blob(ir::FuncletKind::Spatial, rule_command, pairs, context)
}

fn funclet(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Funclet {
    let rule = pairs.next().unwrap().as_rule();
    let vrule = rule_pair(Rule::value_funclet, value_funclet);
    let srule = rule_pair(Rule::schedule_funclet, schedule_funclet);
    let trule = rule_pair(Rule::timeline_funclet, timeline_funclet);
    let sprule = rule_pair(Rule::spatial_funclet, spatial_funclet);
    match rule {
        Rule::value_sep => expect(vrule, pairs, context),
        Rule::schedule_sep => expect(srule, pairs, context),
        Rule::timeline_sep => expect(trule, pairs, context),
        Rule::spatial_sep => expect(sprule, pairs, context),
        _ => panic!(unexpected_rule(&vec![vrule, srule, trule], rule)),
    }
}

fn funclet_def(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::FuncletDef {
    let mut rules = Vec::new();
    rules.push(rule_pair(Rule::external_funclet, external_funclet));
    let rule = compose_pair(funclet, ast::FuncletDef::Local);
    rules.push(rule_pair_boxed(Rule::funclet, rule));
    let rule = compose_pair(
        value_function,
        ast::FuncletDef::ValueFunction,
    );
    rules.push(rule_pair_boxed(Rule::value_function, rule));
    expect_vec(rules, pairs, context)
}

fn value_function_args(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> Vec<ast::Type> {
    let rule = compose_pair(type, reject_hole);
    expect_all(rule_pair_boxed(Rule::typ, rule), pairs, context)
}

fn value_function_funclets(pairs: &mut Pairs<Rule>, context: &mut Context) -> Vec<String> {
    let rule = compose_pair(fn_name, reject_hole);
    expect_all(rule_pair_boxed(Rule::fn_name, rule), pairs, context)
}

fn value_function(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> ast::ValueFunction {
    require_rule(Rule::value_function_sep, pairs, context);
    let name = reject_hole(expect(rule_fn_name(), pairs, context));

    let rule = rule_pair(Rule::value_function_args, value_function_args);
    let input_types = expect(rule, pairs, context);

    let output_types = vec![reject_hole(expect(rule_type(), pairs, context))];

    let rule = rule_pair(Rule::value_function_funclets, value_function_funclets);
    let allowed_funclets = expect(rule, pairs, context);

    ast::ValueFunction {
        name,
        input_types,
        output_types,
        allowed_funclets,
    } // todo add syntax
}

fn funclets(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::FuncletDefs {
    let rule = rule_pair(Rule::funclet_def, funclet_def);
    expect_all(rule, pairs, context)
}

fn extra(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> (String, ast::UncheckedDict) {
    let name = reject_hole(expect(rule_fn_name_sep(), pairs, context));
    let data = expect(rule_unchecked_dict(), pairs, context);
    (name, data)
}

fn extras(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Extras {
    let mut result = HashMap::new();
    let extras = expect_all(rule_pair(Rule::extra, extra), pairs, context);
    for extra in extras.into_iter() {
        result.insert(extra.0, extra.1);
    }
    result
}

fn pipeline(
    pairs: &mut Pairs<Rule>,
    context: &mut Context,
) -> (String, ast::FuncletId) {
    require_rule(Rule::pipeline_sep, pairs, context);
    let name = expect(rule_string_clean(), pairs, context);
    let funclet = reject_hole(expect(rule_fn_name(), pairs, context));
    (name, funclet)
}

fn pipelines(pairs: &mut Pairs<Rule>, context: &mut Context) -> ast::Pipelines {
    let mut result = HashMap::new();
    let extras = expect_all(rule_pair(Rule::pipeline, pipeline), pairs, context);
    for extra in extras.into_iter() {
        result.insert(extra.0, extra.1);
    }
    result
}

fn program(parsed: &mut Pairs<Rule>) -> ast::Program {
    let head = parsed.next().unwrap();
    let mut pairs = match head.as_rule() {
        Rule::program => head.into_inner(),
        _ => panic!("CAIR must start with a program"),
    };

    let mut context = Context::new();

    let version = expect(
        rule_pair(Rule::version, version),
        &mut pairs,
        &mut context,
    );
    let types = expect(rule_pair(Rule::types, types), &mut pairs, &mut context);
    let funclets = expect(
        rule_pair(Rule::funclets, funclets),
        &mut pairs,
        &mut context,
    );
    let extras = expect(
        rule_pair(Rule::extras, extras),
        &mut pairs,
        &mut context,
    );
    let pipelines = expect(
        rule_pair(Rule::pipelines, pipelines),
        &mut pairs,
        &mut context,
    );

    ast::Program {
        version,
        types,
        funclets,
        extras,
        pipelines,
    }
}

pub fn parse(code: &str) -> ast::Program {
    let parsed = IRParser::parse(Rule::program, code);
    match parsed {
        Err(why) => panic!("{:?}", why),
        Ok(mut parse_result) => program(&mut parse_result),
    }
}

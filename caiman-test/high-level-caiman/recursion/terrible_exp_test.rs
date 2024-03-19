struct Callbacks;

impl main::CpuFunctions for Callbacks {
    fn _add_i64_i64(&self, _: &mut dyn caiman_rt::State, a: i64, b: i64) -> (i64,) {
        (a + b,)
    }
    fn _eq_i64_i64(&self, _: &mut dyn caiman_rt::State, a: i64, b: i64) -> (i32,) {
        (if a == b { 1 } else { 0 },)
    }
    fn _leq_i64_i64(&self, _: &mut dyn caiman_rt::State, a: i64, b: i64) -> (i32,) {
        (if a <= b { 1 } else { 0 },)
    }
    fn _sub_i64_i64(&self, _: &mut dyn caiman_rt::State, a: i64, b: i64) -> (i64,) {
        (a - b,)
    }
}

#[test]
fn main() -> Result<(), String> {
    let callbacks = Callbacks;
    let mut wgpu_instance = crate::util::INSTANCE.lock().unwrap();
    let mut root_state = wgpu_instance.create_root_state();
    let mut join_stack_bytes = [0u8; 4096usize];
    let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
    let instance = main::Instance::new(&mut root_state, &callbacks);
    let mut result = instance.start(&mut join_stack, 3, 3);
    while result.returned().is_none() {
        let instance = result.prepare_next();
        result = instance.resume_at__loop_impl(&mut join_stack);
    }
    crate::expect_returned!(27, result.returned().map(|x| x.0))
}

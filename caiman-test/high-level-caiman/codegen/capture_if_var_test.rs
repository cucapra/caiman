struct Callbacks;

impl main::CpuFunctions for Callbacks {
    fn _mul_i64_i64(&self, _: &mut dyn caiman_rt::State, a: i64, b: i64) -> (i64,) {
        (a * b,)
    }

    fn _lt_i64_i64(&self, _: &mut dyn caiman_rt::State, a: i64, b: i64) -> (i32,) {
        if a < b {
            (1,)
        } else {
            (0,)
        }
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
    let result = instance.start(&mut join_stack, 1, -1);
    crate::expect_returned!(-4, result.returned().map(|x| x.0))
}

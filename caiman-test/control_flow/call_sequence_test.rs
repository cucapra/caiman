struct Callbacks;

impl main::CpuFunctions for Callbacks {
    fn mult(&self, _: &mut dyn caiman_rt::State, x: i64, y: i64) -> main::outputs::op {
        (x * y,)
    }
    fn add(&self, _: &mut dyn caiman_rt::State, x: i64, y: i64) -> main::outputs::op {
        (x + y,)
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
    let result = instance.start(&mut join_stack, 3);
    crate::expect_returned!(8, result.returned().map(|x| x.0))
}

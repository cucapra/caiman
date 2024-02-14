struct Callbacks;

impl main::CpuFunctions for Callbacks {
    fn sub(&self, _: &mut dyn caiman_rt::State, value1: i64, value2: i64) -> main::outputs::sub {
        return (value1 - value2,);
    }
    fn mult(&self, _: &mut dyn caiman_rt::State, value1: i64, value2: i64) -> main::outputs::mult {
        return (value1 * value2,);
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
    let result = instance.start(&mut join_stack);
    crate::expect_returned!(25, result.returned().map(|x| x.0))
}
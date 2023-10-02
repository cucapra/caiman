struct Callbacks;

impl main::CpuFunctions for Callbacks {
    fn compute(&self, _: &mut dyn caiman_rt::State, x: i64, z: i64) -> main::outputs::compute {
        return (x * z,);
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
    let end = result.returned().map(|x| x.0);
    crate::expect_returned!(40, end);
}

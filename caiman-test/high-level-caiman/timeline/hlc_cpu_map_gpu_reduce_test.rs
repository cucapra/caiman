struct Callbacks;

impl main::CpuFunctions for Callbacks {
    fn bar(&self, _: &mut dyn caiman_rt::State) -> (i32,) {
        (100,)
    }

    fn baz(&self, _: &mut dyn caiman_rt::State) -> (i32,) {
        (-1,)
    }
}

#[test]
fn main() -> Result<(), String> {
    let mut wgpu_instance = crate::util::INSTANCE.lock().unwrap();
    let callbacks = Callbacks;
    let mut root_state = wgpu_instance.create_root_state();
    let mut join_stack_bytes = [0u8; 4096usize];
    let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
    let instance = main::Instance::new(&mut root_state, &callbacks);
    let result = instance.start(&mut join_stack, 1);
    crate::expect_returned!(-2, result.returned().map(|x| x.0))
}

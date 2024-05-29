struct Callbacks;

impl main::CpuFunctions for Callbacks {
    fn _add_i32_i32(&self, _: &mut dyn caiman_rt::State, a: i32, b: i32) -> (i32,) {
        (a + b,)
    }

    fn simple_cpu(
        &self,
        _: &mut dyn caiman_rt::State,
        dx: i32,
        dy: i32,
        dz: i32,
        a: i32,
    ) -> (i32,) {
        (a + dx + dy + dz + 1,)
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
    let result = instance.start(&mut join_stack, &mut 1);
    crate::expect_returned!(5, result.returned().map(|x| x.0))
}

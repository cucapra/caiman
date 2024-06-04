struct Callbacks;

impl main::CpuFunctions for Callbacks {
    fn neg(&self, _: &mut dyn caiman_rt::State, value: i64) -> main::outputs::neg {
        return (-value,);
    }
    fn add(&self, _: &mut dyn caiman_rt::State, x: i64, y: i64) -> main::outputs::add {
        return (x + y,);
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
    let result = instance.start(&mut join_stack, 3, 4);
    crate::expect_returned!((-3, 5), result.returned().cloned())
}
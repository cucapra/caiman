struct Callbacks;

<<<<<<< HEAD
impl pipeline::main::CpuFunctions for Callbacks {}
=======
impl main::CpuFunctions for Callbacks {
    fn do_thing_on_cpu(
        &self,
        _: &mut dyn caiman_rt::State,
        value: i64,
    ) -> main::outputs::do_thing_on_cpu {
        return (value + 1,);
    }
}
>>>>>>> 0dad0777b4e90316e6bfa55aad90b5c9ee5e0cdd

#[test]
fn main() -> Result<(), String> {
    let callbacks = Callbacks;
    let mut wgpu_instance = crate::util::INSTANCE.lock().unwrap();
    let mut root_state = wgpu_instance.create_root_state();
    let mut join_stack_bytes = [0u8; 4096usize];
    let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
    let instance = main::Instance::new(&mut root_state, &callbacks);
    let result = instance.start(&mut join_stack, 1);
    crate::expect_returned!(4, result.returned().map(|x| x.0))
}

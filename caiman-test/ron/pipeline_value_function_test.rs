struct Callbacks;

impl pipeline_with_value_function::CpuFunctions for Callbacks {
    fn do_thing_on_cpu(
        &self,
        _: &mut dyn caiman_rt::State,
        value: i32,
    ) -> pipeline_with_value_function::outputs::do_thing_on_cpu {
        return (value + 1,);
    }
}

#[test]
fn pipeline_with_value_function() -> Result<(), String> {
    let callbacks = Callbacks;
    let mut wgpu_instance = crate::util::INSTANCE.lock().unwrap();
    let mut root_state = wgpu_instance.create_root_state();
    let mut join_stack_bytes = [0u8; 4096usize];
    let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
    let instance = pipeline_with_value_function::Instance::new(&mut root_state, &callbacks);
    let result = instance.start(&mut join_stack, &mut 1);
    crate::expect_returned!(2, result.returned().map(|x| x.0));
}

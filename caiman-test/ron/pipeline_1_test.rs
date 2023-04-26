struct Callbacks;

impl pipeline_1::CpuFunctions for Callbacks {
    fn do_thing_on_cpu(
        &self,
        _: &mut dyn caiman_rt::State,
        value: i32,
    ) -> pipeline_1::outputs::do_thing_on_cpu {
        return (value + 1,);
    }
}

#[test]
fn pipeline_1() -> Result<(), String> {
    let callbacks = Callbacks;
    let mut wgpu_instance = crate::util::INSTANCE.lock().unwrap();
    let mut root_state = wgpu_instance.create_root_state();
    let mut join_stack_bytes = [0u8; 4096usize];
    let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
    //let result = pipeline_1::run(&mut root_state, & callbacks, 1);
    let instance = pipeline_1::Instance::new(&mut root_state, &callbacks);
    let result = instance.start(&mut join_stack, 1);
    //let result = pipeline_1::funclet11_func(instance, &mut join_stack, 1);
    crate::expect_returned!(3, result.returned().map(|x| x.0))
}

struct Callbacks;

impl main::CpuFunctions for Callbacks {}

#[test]
fn main() -> Result<(), String> {
    let mut wgpu_instance = crate::util::INSTANCE.lock().unwrap();
    let callbacks = Callbacks;
    let mut root_state = wgpu_instance.create_root_state();
    let mut join_stack_bytes = [0u8; 4096usize];
    let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
    let instance = main::Instance::new(&mut root_state, &callbacks);
    let mut result = instance.start(&mut join_stack);
    // infinite recursion since that's the only recursion supported with timeline
    for i in 0..20 {
        let instance = result.prepare_next();
        result = instance.resume_at_loop(&mut join_stack);
    }
    Ok(())
}

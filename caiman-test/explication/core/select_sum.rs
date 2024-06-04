struct Callbacks;

impl main::CpuFunctions for Callbacks {
    fn is_negative(&self, _: &mut dyn caiman_rt::State, x : i64) -> main::outputs::is_negative {
        return ((x < 0) as i64,)
    }
    fn sum(&self, _: &mut dyn caiman_rt::State, arr: [i64; 4]) -> main::outputs::sum {
        return (arr.iter().sum(),);
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
    let result = instance.start(&mut join_stack, [1, 0, 0, 0], [2, 2, 2, 2], [1, 1, 1, 1]);
    crate::expect_returned!(4, result.returned().map(|x| x.0))
}
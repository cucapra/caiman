struct Callbacks;

impl main::CpuFunctions for Callbacks {
    fn _add_i64_i64(&self, _: &mut dyn caiman_rt::State, a: i64, b: i64) -> (i64,) {
        (a + b,)
    }
    fn sort5(
        &self,
        _: &mut dyn caiman_rt::State,
        a: i64,
        b: i64,
        c: i64,
        d: i64,
        e: i64,
    ) -> (i64, i64, i64, i64, i64) {
        let mut arr = [a, b, c, d, e];
        arr.sort();
        (arr[0], arr[1], arr[2], arr[3], arr[4])
    }
    fn _mul_i64_i64(&self, _: &mut dyn caiman_rt::State, a: i64, b: i64) -> (i64,) {
        (a * b,)
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
    let result = instance.start(&mut join_stack, 0, -10, 2);
    if !matches!(result.returned(), Some((-20, 0, 2, 4, 20))) {
        return Err(format!("{:?}", result.returned()));
    }
    Ok(())
}

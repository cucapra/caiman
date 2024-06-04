struct Callbacks;

use main::outputs;
impl main::CpuFunctions for Callbacks {
    fn _add_i32_i32(&self, _: &mut dyn caiman_rt::State, a: i32, b: i32) -> outputs::_add_i32_i32 {
        (a + b,)
    }

    fn _lt_i32_i32(&self, _: &mut dyn caiman_rt::State, a: i32, b: i32) -> outputs::_lt_i32_i32 {
        if a < b {
            (1,)
        } else {
            (0,)
        }
    }

    fn _gt_i32_i32(&self, _: &mut dyn caiman_rt::State, a: i32, b: i32) -> outputs::_lt_i32_i32 {
        if a > b {
            (1,)
        } else {
            (0,)
        }
    }

    fn _eq_i32_i32(&self, _: &mut dyn caiman_rt::State, a: i32, b: i32) -> outputs::_lt_i32_i32 {
        if a == b {
            (1,)
        } else {
            (0,)
        }
    }

    fn rec_sum_cpu(
        &self,
        _: &mut dyn caiman_rt::State,
        _: i32,
        _: i32,
        _: i32,
        start: i32,
        end: i32,
    ) -> outputs::rec_sum_cpu {
        let mut n = 0;
        for i in start..end {
            n += i;
        }
        (n,)
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
    let result = instance.start(&mut join_stack, 0, 10, 11);
    let mut c = 0;
    for i in 0..10 {
        c += i;
    }
    crate::expect_returned!(c, result.returned().map(|x| x.0))
}

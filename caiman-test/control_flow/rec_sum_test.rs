struct Callbacks;

impl main::CpuFunctions for Callbacks {
    fn add(&self, _: &mut dyn caiman_rt::State, a: i64, b: i64) -> main::outputs::add {
        (a + b,)
    }

    fn lte(&self, _: &mut dyn caiman_rt::State, a: i64, b: i64) -> main::outputs::lte {
        if a <= b {
            (1,)
        } else {
            (0,)
        }
    }
    fn gt(&self, _: &mut dyn caiman_rt::State, a: i64, b: i64) -> main::outputs::gt {
        if a > b {
            (1,)
        } else {
            (0,)
        }
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
    let result = instance.start(&mut join_stack);

    fn sum(a: i64) -> i64 {
        if a <= 0 {
            0
        } else {
            let mut s = 0;
            for i in 0..=a {
                s += i;
            }
            s
        }
    }
    crate::expect_returned!(sum(20), result.returned().map(|x| x.0))
}

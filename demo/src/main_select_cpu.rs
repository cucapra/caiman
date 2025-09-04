#![allow(warnings)]

mod caiman_out;
mod util;

struct Callbacks;

impl caiman_out::main::CpuFunctions for Callbacks {
    fn sum(&self, _: &mut dyn caiman_rt::State, x: [i32; 4]) -> caiman_out::main::outputs::sum {
        (x.iter().sum(),)
    }

    fn negate(&self, _: &mut dyn caiman_rt::State, x: [i32; 4]) -> caiman_out::main::outputs::negate {
        ([-x[0], -x[1], -x[2], -x[3]],)
    }

    fn _lt_i32_i32(&self, _: &mut dyn caiman_rt::State, a: i32, b: i32) -> (i32,) {
        if a < b {
            (1,)
        } else {
            (0,)
        }
    }
}

fn main() {
    let v1: [i32; 4] = [1, 0, 0, 0];
    let v2: [i32; 4] = [1, 2, 3, 4];
    let v3: [i32; 4] = [1, 1, 1, 1];

    let callbacks = Callbacks;
    let mut wgpu_instance = util::INSTANCE.lock().unwrap();
    let mut root_state = wgpu_instance.create_root_state();
    let mut join_stack_bytes = [0u8; 4096usize];
    let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
    let instance = caiman_out::main::Instance::new(&mut root_state, &callbacks);
    let result = instance.start(&mut join_stack, v1, v2, v3);
    println!("Output: {:?}", result.returned().map(|x| x.0).unwrap());
}

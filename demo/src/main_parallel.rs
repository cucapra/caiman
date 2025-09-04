#![allow(warnings)]

mod caiman_out;
mod util;

struct Callbacks;

impl caiman_out::main::CpuFunctions for Callbacks {
    fn add(&self, state: &mut caiman_rt::State, x: i32, y: i32) -> caiman_out::main::outputs::add {
        (x + y,)
    }
}

fn main() {
    let mut x : i32 = 1;
    let mut z : i32 = 2;

    let callbacks = Callbacks;
    let mut wgpu_instance = util::INSTANCE.lock().unwrap();
    let mut root_state = wgpu_instance.create_root_state();
    let mut join_stack_bytes = [0u8; 4096usize];
    let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
    let instance = caiman_out::main::Instance::new(&mut root_state, &callbacks);
    let result = instance.start(&mut join_stack, &mut x, &mut z);
    println!("Output: {:?}", result.returned().map(|x| x.0).unwrap());
}

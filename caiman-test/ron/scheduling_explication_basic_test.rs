#![allow(warnings)]
mod pipeline {
    #![allow(warnings)]
    include!("_generated.rs");
}

struct Callbacks;

impl pipeline::pipeline_with_value_function::CpuFunctions for Callbacks {
    fn do_thing_on_cpu(
        &self,
        state: &mut caiman_rt::State,
        value: i32,
    ) -> pipeline::looping_pipeline::outputs::do_thing_on_cpu {
        return (value + 1,);
    }
}

fn main() {
    use caiman_rt::wgpu;
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let adapter =
        futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
            .unwrap();
    let (mut device, mut queue) = futures::executor::block_on(
        adapter.request_device(&std::default::Default::default(), None),
    )
        .unwrap();
    let callbacks = crate::Callbacks;
    let mut root_state = caiman_rt::RootState::new(&mut device, &mut queue);
    let mut join_stack_bytes = [0u8; 4096usize];
    let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
    let instance = crate::pipeline::pipeline_with_value_function::Instance::new(
        &mut root_state,
        &callbacks,
    );
    let result = instance.start(&mut join_stack, 1);
    println!("{}", result.returned().unwrap().0);
}
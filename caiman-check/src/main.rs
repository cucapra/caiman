mod pipelines {
    include!("output.out");
}

struct Callbacks;

impl pipelines::main::CpuFunctions for Callbacks {
    fn add(
        &self,
        state: &mut dyn caiman_rt::State,
        value: (i64, i64),
    ) -> pipelines::main::outputs::add {
        return (value.0 + value.1,);
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
    //let result = crate::pipelines::main::run(&mut root_state, & callbacks, 1);
    let instance = crate::pipelines::main::Instance::new(&mut root_state, &callbacks);
    let result = instance.start(&mut join_stack, 1);
    //let result = crate::pipelines::main::funclet11_func(instance, &mut join_stack, 1);
    println!("{}", result.returned().unwrap().0);
}
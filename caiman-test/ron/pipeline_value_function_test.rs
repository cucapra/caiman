struct Callbacks;

impl pipeline::looping_pipeline::CpuFunctions for Callbacks {
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

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: std::mem::size_of::<i32>() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: true,
    });
    buffer
        .slice(0..)
        .get_mapped_range_mut()
        .copy_from_slice(&0i32.to_ne_bytes());
    buffer.unmap();

    let callbacks = crate::Callbacks;
    let mut root_state = caiman_rt::RootState::new(&mut device, &mut queue);
    let mut join_stack_bytes = [0u8; 4096usize];
    let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
    let instance =
        crate::pipeline::looping_pipeline::Instance::new(&mut root_state, &callbacks);

    let input_ref = caiman_rt::GpuBufferRef::<i32>::new(&buffer, 0);
    let mut result = instance.start(&mut join_stack, input_ref);
    for i in 0..5 {
        let instance = result.prepare_next();
        result = instance.resume_at_loop(&mut join_stack);
    }

    device.poll(wgpu::Maintain::Wait);
    //futures::executor::block_on(queue.on_submitted_work_done());
    //device.poll(wgpu::Maintain::Poll);
    use std::convert::TryInto;
    let buffer_slice = buffer.slice(0..);
    let future = buffer_slice.map_async(wgpu::MapMode::Read);
    //futures::executor::block_on(future);
    device.poll(wgpu::Maintain::Wait);
    let slice =
        unsafe { std::slice::from_raw_parts(buffer_slice.get_mapped_range().as_ptr(), 4) };
    let final_value: i32 = i32::from_ne_bytes([slice[0], slice[1], slice[2], slice[3]]);
    println!("{}", final_value);
}
struct Callbacks;

impl looping_pipeline::CpuFunctions for Callbacks {
    fn do_thing_on_cpu(
        &self,
        _: &mut dyn caiman_rt::State,
        value: i32,
    ) -> looping_pipeline::outputs::do_thing_on_cpu {
        (value + 1,)
    }
}

#[test]
fn looping_pipeline() -> Result<(), String> {
    use caiman_rt::wgpu;
    let mut wgpu_instance = crate::util::INSTANCE.lock().unwrap();
    let buffer = wgpu_instance
        .device()
        .create_buffer(&wgpu::BufferDescriptor {
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

    let callbacks = Callbacks;
    let mut root_state = wgpu_instance.create_root_state();
    let mut join_stack_bytes = [0u8; 4096usize];
    let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
    let instance = looping_pipeline::Instance::new(&mut root_state, &callbacks);

    let input_ref = caiman_rt::GpuBufferRef::<i32>::new(&buffer, 0);
    let mut result = instance.start(&mut join_stack, input_ref, None);
    for _ in 0..5 {
        let instance = result.prepare_next();
        result = instance.resume_at_loop(&mut join_stack);
    }

    wgpu_instance.device().poll(wgpu::Maintain::Wait);
    //futures::executor::block_on(queue.on_submitted_work_done());
    //device.poll(wgpu::Maintain::Poll);
    let buffer_slice = buffer.slice(0..);
    let _future = buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
    //futures::executor::block_on(future);
    wgpu_instance.device().poll(wgpu::Maintain::Wait);
    let slice = unsafe { std::slice::from_raw_parts(buffer_slice.get_mapped_range().as_ptr(), 4) };
    let final_value: i32 = i32::from_ne_bytes([slice[0], slice[1], slice[2], slice[3]]);
    crate::expect_returned!(6, Some(final_value));
}

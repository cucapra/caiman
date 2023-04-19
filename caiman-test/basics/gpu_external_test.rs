struct Callbacks;

impl main::CpuFunctions for Callbacks {}

#[test]
fn main() -> Result<(), String> {
    use caiman_rt::wgpu;
    let mut wgpu_instance = crate::util::INSTANCE.lock().unwrap();
    let input_buffer = wgpu_instance
        .device()
        .create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 1024u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::MAP_READ
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
    let output_buffer = wgpu_instance
        .device()
        .create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 1024u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::MAP_READ
                | wgpu::BufferUsages::MAP_WRITE
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

    let callbacks = Callbacks;
    let mut root_state = wgpu_instance.create_root_state();
    let mut join_stack_bytes = [0u8; 4096usize];
    let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);

    let input_alloc = caiman_rt::GpuBufferAllocator::new(&input_buffer, 1024);
    let output_alloc = caiman_rt::GpuBufferAllocator::new(&output_buffer, 1024);
    let instance = main::Instance::new(&mut root_state, &callbacks);

    let _result = instance.start(&mut join_stack, 0, input_alloc, output_alloc);
    wgpu_instance.device().poll(wgpu::Maintain::Wait);
    let buffer_slice = output_buffer.slice(0..);
    // TODO: ...how does this not block indefinitely? The docs say that the call will only complete
    // once the callback returns, and the callback will only be executed once poll() or submit()
    // is called elsewhere in the program.
    let _future = buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
    wgpu_instance.device().poll(wgpu::Maintain::Wait);
    let slice = unsafe { std::slice::from_raw_parts(buffer_slice.get_mapped_range().as_ptr(), 4) };
    let final_value = i32::from_ne_bytes([slice[0], slice[1], slice[2], slice[3]]);
    crate::expect_returned!(1, Some(final_value));
}

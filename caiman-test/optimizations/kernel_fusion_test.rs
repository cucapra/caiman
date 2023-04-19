struct Callbacks {}

impl fusion::CpuFunctions for Callbacks {}

impl fusion_elide::CpuFunctions for Callbacks {}

impl fusion_partial::CpuFunctions for Callbacks {}

#[test]
pub fn fusion() -> Result<(), String> {
    use caiman_rt::wgpu;
    let mut wgpu_instance = crate::util::INSTANCE.lock().unwrap();
    let buf1 = wgpu_instance
        .device()
        .create_buffer(&wgpu::BufferDescriptor {
            label: Some("buf1 (a, b, c, d, ab+cd)"),
            size: 2048,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
    let buf2 = wgpu_instance
        .device()
        .create_buffer(&wgpu::BufferDescriptor {
            label: Some("buf2 (ab, cd)"),
            size: 2048,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

    let callbacks = Callbacks {};
    let mut root_state = wgpu_instance.create_root_state();
    let mut join_stack_bytes = [0u8; 4096usize];
    let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
    let instance = fusion::Instance::new(&mut root_state, &callbacks);
    let buf1_alloc = caiman_rt::GpuBufferAllocator::new(&buf1, 2048);
    let buf2_alloc = caiman_rt::GpuBufferAllocator::new(&buf2, 2048);
    let result = instance.start(
        &mut join_stack,
        9.0,
        -5.0,
        16.4,
        0.75,
        buf1_alloc,
        buf2_alloc,
    );
    crate::expect_returned!(-32.7, result.returned().map(|x| x.0));
}

#[test]
pub fn fusion_elide() -> Result<(), String> {
    let mut wgpu_instance = crate::util::INSTANCE.lock().unwrap();
    let callbacks = Callbacks {};
    let mut root_state = wgpu_instance.create_root_state();
    let mut join_stack_bytes = [0u8; 4096usize];
    let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
    let instance = fusion_elide::Instance::new(&mut root_state, &callbacks);
    let result = instance.start(&mut join_stack, 9.0, -5.0, 16.4, 0.75);
    crate::expect_returned!(-32.7, result.returned().map(|x| x.0));
}

#[test]
pub fn fusion_partial() -> Result<(), String> {
    let mut wgpu_instance = crate::util::INSTANCE.lock().unwrap();
    let callbacks = Callbacks {};
    let mut root_state = wgpu_instance.create_root_state();
    let mut join_stack_bytes = [0u8; 4096usize];
    let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
    let instance = fusion_partial::Instance::new(&mut root_state, &callbacks);
    let result = instance.start(&mut join_stack, 9.0, -5.0, 16.4, 0.75);
    crate::expect_returned!(-32.7, result.returned().map(|x| x.0));
}

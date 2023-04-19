struct Callbacks {}

impl saxpy::CpuFunctions for Callbacks {
    fn fmul_cpu(&self, _: &mut dyn caiman_rt::State, a: f32, b: f32) -> saxpy::outputs::fmul_cpu {
        return (a * b,);
    }
    fn fadd_cpu(&self, _: &mut dyn caiman_rt::State, a: f32, b: f32) -> saxpy::outputs::fmul_cpu {
        return (a + b,);
    }
}

#[test]
pub fn saxpy() -> Result<(), String> {
    use caiman_rt::wgpu;
    let mut wgpu_instance = crate::util::INSTANCE.lock().unwrap();
    let buf1 = wgpu_instance
        .device()
        .create_buffer(&wgpu::BufferDescriptor {
            label: Some("buf1 (a, x, ax+y)"),
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
            label: Some("buf2 (ax, y)"),
            size: 2048,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

    let callbacks = Callbacks {};
    let mut root_state = wgpu_instance.create_root_state();
    let mut join_stack_bytes = [0u8; 4096usize];
    let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
    let instance = saxpy::Instance::new(&mut root_state, &callbacks);
    let buf1_alloc = caiman_rt::GpuBufferAllocator::new(&buf1, 2048);
    let buf2_alloc = caiman_rt::GpuBufferAllocator::new(&buf2, 2048);
    let result = instance.start(&mut join_stack, 128.0, 56.0, -12.4, buf1_alloc, buf2_alloc);
    crate::expect_returned!(7155.6, result.returned().map(|x| x.0));
}

#[test]
pub fn saxpy_call() -> Result<(), String> {
    todo!()
}

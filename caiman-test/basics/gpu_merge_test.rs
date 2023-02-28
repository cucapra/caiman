struct Callbacks;

impl pipeline::main::CpuFunctions for Callbacks {
    fn cpu_add(
        &self,
        state: &mut caiman_rt::State,
        x: i64,
        y: i64
    ) -> pipeline::main::outputs::cpu_add {
        return (x + y, );
    }
    fn flatten (
        &self,
        state: &mut caiman_rt::State,
        x: i64
    ) -> pipeline::main::outputs::flatten {
        use std::convert::TryFrom;
        return (i32::try_from(x).ok().unwrap(), );
    }
}

fn main() {
    use caiman_rt::wgpu;
    use caiman_rt::State;

    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let adapter =
        futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
            .unwrap();
    let (mut device, mut queue) = futures::executor::block_on(
        adapter.request_device(&Default::default(), None),
    )
        .unwrap();

    let input_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 1024u64,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::MAP_READ
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 1024u64,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::MAP_READ
            | wgpu::BufferUsages::MAP_WRITE
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let callbacks = crate::Callbacks;

    let mut root_state = caiman_rt::RootState::new(&mut device, &mut queue);
    let mut join_stack_bytes = [0u8; 4096usize];
    let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);

    let mut input_alloc = caiman_rt::GpuBufferAllocator::new(&input_buffer, 1024);
    let mut output_alloc = caiman_rt::GpuBufferAllocator::new(&output_buffer, 1024);
    let instance = crate::pipeline::main::Instance::new(&mut root_state, &callbacks);

    let mut result = instance.start(&mut join_stack, 0, input_alloc, output_alloc);
    device.poll(wgpu::Maintain::Wait);
    use std::convert::TryInto;
    let buffer_slice = output_buffer.slice(0..);
    let future = buffer_slice.map_async(wgpu::MapMode::Read);
    device.poll(wgpu::Maintain::Wait);
    let slice =
        unsafe { std::slice::from_raw_parts(buffer_slice.get_mapped_range().as_ptr(), 4) };
    let final_value: i32 = i32::from_ne_bytes([slice[0], slice[1], slice[2], slice[3]]);
    println!("{}", final_value);
}
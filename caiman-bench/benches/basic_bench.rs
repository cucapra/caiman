use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn wgpu_basic_bench_1(c : &mut Criterion)
{
	let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
	//let instance = wgpu::Instance::new(wgpu::InstanceDescriptor{backends: wgpu::Backends::VULKAN, dx12_shader_compiler: wgpu::Dx12Compiler::Fxc});
	let adapter_future = instance.request_adapter(&wgpu::RequestAdapterOptions {
		power_preference: wgpu::PowerPreference::default(),
		compatible_surface: None,
		force_fallback_adapter: false,
	});
	let adapter = futures::executor::block_on(adapter_future).unwrap();
	let device_desc = wgpu::DeviceDescriptor {
		label: None,
		features: wgpu::Features::default(),
		limits: wgpu::Limits::default(),
	};
	let device_future = adapter.request_device(&device_desc, None);
	let (device, queue) = futures::executor::block_on(device_future).unwrap();

	let the_answer = 42u32;
	const answer_byte_size : usize = std::mem::size_of::<u32>();
	let src_buffer = device.create_buffer(& wgpu::BufferDescriptor{label : None, size : answer_byte_size as u64, usage : wgpu::BufferUsages::COPY_SRC, mapped_at_creation : true});
	let dst_buffer = device.create_buffer(& wgpu::BufferDescriptor{label : None, size : answer_byte_size as u64, usage : wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation : false});

	src_buffer.slice(0..).get_mapped_range_mut().copy_from_slice(& the_answer.to_le_bytes());
	src_buffer.unmap();

	//let mut rng = rand::rngs::StdRng::seed_from_u64(123456789);
	c.bench_function("wgpu basic bench 1", |b| b.iter(|| {
		for i in 0 .. black_box(1) {
			let command_buffer =
			{
				let mut command_encoder = device.create_command_encoder(& wgpu::CommandEncoderDescriptor{ label : None });
				command_encoder.copy_buffer_to_buffer(& src_buffer, 0, & dst_buffer, 0, answer_byte_size as wgpu::BufferAddress);
				command_encoder.finish()
			};
			let index = queue.submit([command_buffer]);
			device.poll(wgpu::Maintain::WaitForSubmissionIndex(index));
			let (sender, receiver) = futures::channel::oneshot::channel::<()>();
			let callback = |r : Result<(), wgpu::BufferAsyncError>| {
				let _ = r.unwrap();
				sender.send(()).unwrap();
			};
			dst_buffer.slice(0..).map_async(wgpu::MapMode::Read, callback );
			device.poll(wgpu::Maintain::Wait);
			futures::executor::block_on(async { receiver.await });
			let result = unsafe { * std::mem::transmute::<_, * const [u8; answer_byte_size]>(dst_buffer.slice(0..).get_mapped_range().as_ptr()) };
			assert_eq!(the_answer, u32::from_le_bytes(result)); 
			dst_buffer.unmap();
		}
	}));
}

use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Simple matrix-vector multiplication with a permutation matrix
// This mostly exists as a sanity/functional test
pub fn wgpu_matmul_bench_1(c : &mut Criterion)
{
	let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
	let adapter_future = instance.request_adapter(&wgpu::RequestAdapterOptions {
		power_preference: wgpu::PowerPreference::default(),
		compatible_surface: None,
		force_fallback_adapter: false,
	});
	let adapter = futures::executor::block_on(adapter_future).unwrap();
	let device_desc = wgpu::DeviceDescriptor {
		label: None,
		features: wgpu::Features::default() | wgpu::Features::MAPPABLE_PRIMARY_BUFFERS,
		limits: wgpu::Limits::default(),
	};
	let device_future = adapter.request_device(&device_desc, None);
	let (device, queue) = futures::executor::block_on(device_future).unwrap();

	const vector_width : usize = 32;
	const vector_byte_size : usize = std::mem::size_of::<f32>() * vector_width;
	const matrix_byte_size : usize = std::mem::size_of::<f32>() * vector_width * vector_width;
	let src_buffer = device.create_buffer(& wgpu::BufferDescriptor{label : None, size : vector_byte_size as u64, usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_WRITE, mapped_at_creation : false});
	let dst_buffer = device.create_buffer(& wgpu::BufferDescriptor{label : None, size : vector_byte_size as u64, usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ, mapped_at_creation : false});
	let mat_buffer = device.create_buffer(& wgpu::BufferDescriptor{label : None, size : matrix_byte_size as u64, usage : wgpu::BufferUsages::STORAGE, mapped_at_creation : true});
	let dimensions_buffer = device.create_buffer(& wgpu::BufferDescriptor{label : None, size : std::mem::size_of::<[u32; 2]>() as u64, usage : wgpu::BufferUsages::UNIFORM, mapped_at_creation : true});

	let mut matrix = [0f32; vector_width * vector_width];

	for i in 0 .. (vector_width / 2usize)
	{
		if i * 2 < vector_width
		{
			let x = i * 2usize;
			let y = i * 2usize + 1usize;
			matrix[y * vector_width + x] = 1.0f32;
			matrix[x * vector_width + y] = 1.0f32;
		}
	}

	mat_buffer.slice(0..).get_mapped_range_mut().copy_from_slice(bytemuck::cast_slice::<f32, u8>(matrix.as_slice()));
	mat_buffer.unmap();
	dimensions_buffer.slice(0..).get_mapped_range_mut().copy_from_slice(bytemuck::cast_slice::<u32, u8>([vector_width as u32, vector_width as u32].as_slice()));
	dimensions_buffer.unmap();

	let mut shader_module = device.create_shader_module(wgpu::include_wgsl!("kernels/matvecmul.wgsl"));
	let bind_group_layout_entries =
	[
		wgpu::BindGroupLayoutEntry{binding : 0, visibility : wgpu::ShaderStages::COMPUTE, ty : wgpu::BindingType::Buffer{ty : wgpu::BufferBindingType::Uniform, has_dynamic_offset : false, min_binding_size : None}, count : None},
		wgpu::BindGroupLayoutEntry{binding : 1, visibility : wgpu::ShaderStages::COMPUTE, ty : wgpu::BindingType::Buffer{ty : wgpu::BufferBindingType::Storage{read_only : true}, has_dynamic_offset : false, min_binding_size : None}, count : None},
		wgpu::BindGroupLayoutEntry{binding : 2, visibility : wgpu::ShaderStages::COMPUTE, ty : wgpu::BindingType::Buffer{ty : wgpu::BufferBindingType::Storage{read_only : true}, has_dynamic_offset : false, min_binding_size : None}, count : None},
		wgpu::BindGroupLayoutEntry{binding : 3, visibility : wgpu::ShaderStages::COMPUTE, ty : wgpu::BindingType::Buffer{ty : wgpu::BufferBindingType::Storage{read_only : false}, has_dynamic_offset : false, min_binding_size : None}, count : None},
	];
	let bind_group_layout = device.create_bind_group_layout(& wgpu::BindGroupLayoutDescriptor{label : None, entries : & bind_group_layout_entries});
	let mut pipeline_layout = device.create_pipeline_layout(& wgpu::PipelineLayoutDescriptor{ label : None, bind_group_layouts : &[& bind_group_layout], push_constant_ranges : &[]});
	let mut pipeline = device.create_compute_pipeline(& wgpu::ComputePipelineDescriptor{ label : None, layout : Some(& pipeline_layout), module : & shader_module, entry_point : "matvecmul"});
	let bind_group_entries =
	[
		wgpu::BindGroupEntry{binding : 0, resource : wgpu::BindingResource::Buffer(wgpu::BufferBinding{buffer : & dimensions_buffer, offset : 0, size : None})},
		wgpu::BindGroupEntry{binding : 1, resource : wgpu::BindingResource::Buffer(wgpu::BufferBinding{buffer : & mat_buffer, offset : 0, size : None})},
		wgpu::BindGroupEntry{binding : 2, resource : wgpu::BindingResource::Buffer(wgpu::BufferBinding{buffer : & src_buffer, offset : 0, size : None})},
		wgpu::BindGroupEntry{binding : 3, resource : wgpu::BindingResource::Buffer(wgpu::BufferBinding{buffer : & dst_buffer, offset : 0, size : None})},
	];
	let bind_group = device.create_bind_group(& wgpu::BindGroupDescriptor{label : None, layout : & bind_group_layout, entries : & bind_group_entries});	

	let mut src_vector = [0f32; vector_width];

	//let mut rng = rand::rngs::StdRng::seed_from_u64(123456789);
	c.bench_function("wgpu matmul bench 1", |b| b.iter(|| {
		{
			let (sender, receiver) = futures::channel::oneshot::channel::<()>();
			let callback = |r : Result<(), wgpu::BufferAsyncError>| {
				let _ = r.unwrap();
				sender.send(()).unwrap();
			};
			src_buffer.slice(0..).map_async(wgpu::MapMode::Write, callback );
			device.poll(wgpu::Maintain::Wait);
			futures::executor::block_on(async { receiver.await });
			let src = unsafe { std::slice::from_raw_parts_mut::<f32>(std::mem::transmute::<_, * mut f32>(src_buffer.slice(0..).get_mapped_range().as_ptr()), vector_width) };
			for i in 0 .. vector_width
			{
				let v = i as f32;
				src[i] = v;
				src_vector[i] = v;
			}
			src_buffer.unmap();
		}

		let command_buffer =
		{
			let mut command_encoder = device.create_command_encoder(& wgpu::CommandEncoderDescriptor{ label : None });
			{
				let mut compute_pass = command_encoder.begin_compute_pass(& wgpu::ComputePassDescriptor{label: None});
				compute_pass.set_bind_group(0, & bind_group, &[]);
				compute_pass.set_pipeline(& pipeline);
				compute_pass.dispatch_workgroups(caiman_bench::util::divide_rounding_up(vector_width as u32, 32), 1, 1);
			}
			command_encoder.finish()
		};
		let index = queue.submit([command_buffer]);

		let mut dst_vector = [0f32; vector_width];
		for y in 0 .. vector_width
		{
			let mut sum = 0f32;
			for x in 0 .. vector_width
			{
				sum += matrix[y * vector_width + x] * src_vector[x];
			}
			dst_vector[y] = sum;
		}

		device.poll(wgpu::Maintain::WaitForSubmissionIndex(index));

		{
			let (sender, receiver) = futures::channel::oneshot::channel::<()>();
			let callback = |r : Result<(), wgpu::BufferAsyncError>| {
				let _ = r.unwrap();
				sender.send(()).unwrap();
			};
			dst_buffer.slice(0..).map_async(wgpu::MapMode::Read, callback );
			device.poll(wgpu::Maintain::Wait);
			futures::executor::block_on(async { receiver.await });
			let result = unsafe { std::slice::from_raw_parts(std::mem::transmute::<_, * const f32>(dst_buffer.slice(0..).get_mapped_range().as_ptr()), vector_width) };
			for i in 0 .. vector_width
			{
				assert!((result[i] - dst_vector[i].abs() < 0.01));
			}
			dst_buffer.unmap();
		}
	}));
}

criterion_group!(matmul_benches, wgpu_matmul_bench_1);

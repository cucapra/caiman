mod pipelines
{
	include!(concat!(env!("OUT_DIR"), "/generated/pipelines.txt"));
}

struct Callbacks;


impl pipelines::pipeline_1::CpuFunctions for Callbacks
{
	fn do_thing_on_cpu( & self, state : &mut caiman_rt::State, value : i32 ) -> pipelines::pipeline_1::outputs::do_thing_on_cpu
	{
		return ( value + 1, );
	}
}

/*impl pipelines::pipeline_with_gpu_gpu_communication::CpuFunctions for Callbacks
{
	fn do_thing_on_cpu( & self, state : &mut pipelines::State, value : i32 ) -> pipelines::pipeline_with_gpu_gpu_communication::outputs::do_thing_on_cpu
	{
		return pipelines::pipeline_with_gpu_gpu_communication::outputs::do_thing_on_cpu { field_0 : value + 1 };
	}
}

impl pipelines::pipeline_with_single_cpu_call::CpuFunctions for Callbacks
{
	fn do_thing_on_cpu( & self, state : &mut pipelines::State, value : i32 ) -> pipelines::pipeline_with_single_cpu_call::outputs::do_thing_on_cpu
	{
		return pipelines::pipeline_with_single_cpu_call::outputs::do_thing_on_cpu { field_0 : value + 1 };
	}
}

impl pipelines::pipeline_with_gpu_cpu_communication::CpuFunctions for Callbacks
{
	fn do_thing_on_cpu( & self, state : &mut pipelines::State, value : i32 ) -> pipelines::pipeline_with_gpu_cpu_communication::outputs::do_thing_on_cpu
	{
		return pipelines::pipeline_with_gpu_cpu_communication::outputs::do_thing_on_cpu { field_0 : value + 1 };
	}
}

impl pipelines::pipeline_with_cpu_cpu_communication::CpuFunctions for Callbacks
{
	fn do_thing_on_cpu( & self, state : &mut pipelines::State, value : i32 ) -> pipelines::pipeline_with_cpu_cpu_communication::outputs::do_thing_on_cpu
	{
		return pipelines::pipeline_with_cpu_cpu_communication::outputs::do_thing_on_cpu { field_0 : value + 1 };
	}
}

impl pipelines::pipeline_with_single_gpu_call::CpuFunctions for Callbacks
{
	fn do_thing_on_cpu( & self, state : &mut pipelines::State, value : i32 ) -> pipelines::pipeline_with_single_gpu_call::outputs::do_thing_on_cpu
	{
		return pipelines::pipeline_with_single_gpu_call::outputs::do_thing_on_cpu { field_0 : value + 1 };
	}
}

impl pipelines::pipeline_with_yield_enter_loop_exit::CpuFunctions for Callbacks
{
	fn do_thing_on_cpu( & self, state : &mut pipelines::State, value : i32 ) -> pipelines::pipeline_with_yield_enter_loop_exit::outputs::do_thing_on_cpu
	{
		return pipelines::pipeline_with_yield_enter_loop_exit::outputs::do_thing_on_cpu { field_0 : value + 1 };
	}
}*/

impl pipelines::pipeline_with_value_function::CpuFunctions for Callbacks
{
	fn do_thing_on_cpu( & self, state : &mut caiman_rt::State, value : i32 ) -> pipelines::pipeline_with_value_function::outputs::do_thing_on_cpu
	{
		return ( value + 1, );
	}
}

impl pipelines::looping_pipeline::CpuFunctions for Callbacks
{
	fn do_thing_on_cpu( & self, state : &mut caiman_rt::State, value : i32 ) -> pipelines::looping_pipeline::outputs::do_thing_on_cpu
	{
		return ( value + 1, );
	}
}

fn main()
{
}

#[cfg(test)]
mod tests
{
	#[test]
	fn test_1()
	{
		use caiman_rt::wgpu;
		let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
		let adapter = futures::executor::block_on(instance.request_adapter(& wgpu::RequestAdapterOptions { power_preference : wgpu::PowerPreference::default(), compatible_surface : None, force_fallback_adapter : false })).unwrap();
		let (mut device, mut queue) = futures::executor::block_on(adapter.request_device(& std::default::Default::default(), None)).unwrap();
		let callbacks = crate::Callbacks;
		let mut root_state = caiman_rt::RootState::new(&mut device, &mut queue);
		let mut join_stack_bytes = [0u8; 4096usize];
		let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
		//let result = crate::pipelines::pipeline_1::run(&mut root_state, & callbacks, 1);
		let instance = crate::pipelines::pipeline_1::Instance::new(&mut root_state, & callbacks);
		let result = instance.start(&mut join_stack, 1);
		//let result = crate::pipelines::pipeline_1::funclet11_func(instance, &mut join_stack, 1);
		assert_eq!(3, result.returned().unwrap().0);
	}

	/*#[test]
	fn test_2()
	{
		let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
		let adapter = futures::executor::block_on(instance.request_adapter(& wgpu::RequestAdapterOptions { power_preference : wgpu::PowerPreference::default(), compatible_surface : None, force_fallback_adapter : false })).unwrap();
		let (mut device, mut queue) = futures::executor::block_on(adapter.request_device(& std::default::Default::default(), None)).unwrap();
		let callbacks = crate::Callbacks;
		let mut root_state = crate::pipelines::RootState::new(&mut device, &mut queue);
		let result = crate::pipelines::pipeline_with_gpu_gpu_communication::run(&mut root_state, & callbacks, 1);
		assert_eq!(3, result.field_0);
	}

	#[test]
	fn test_3()
	{
		let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
		let adapter = futures::executor::block_on(instance.request_adapter(& wgpu::RequestAdapterOptions { power_preference : wgpu::PowerPreference::default(), compatible_surface : None, force_fallback_adapter : false })).unwrap();
		let (mut device, mut queue) = futures::executor::block_on(adapter.request_device(& std::default::Default::default(), None)).unwrap();
		let callbacks = crate::Callbacks;
		let mut root_state = crate::pipelines::RootState::new(&mut device, &mut queue);
		let result = crate::pipelines::pipeline_with_single_cpu_call::run(&mut root_state, & callbacks, 1);
		assert_eq!(2, result.field_0);
	}

	#[test]
	fn test_4()
	{
		let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
		let adapter = futures::executor::block_on(instance.request_adapter(& wgpu::RequestAdapterOptions { power_preference : wgpu::PowerPreference::default(), compatible_surface : None, force_fallback_adapter : false })).unwrap();
		let (mut device, mut queue) = futures::executor::block_on(adapter.request_device(& std::default::Default::default(), None)).unwrap();
		let callbacks = crate::Callbacks;
		let mut root_state = crate::pipelines::RootState::new(&mut device, &mut queue);
		let result = crate::pipelines::pipeline_with_gpu_cpu_communication::run(&mut root_state, & callbacks, 1);
		assert_eq!(3, result.field_0);
	}

	#[test]
	fn test_5()
	{
		let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
		let adapter = futures::executor::block_on(instance.request_adapter(& wgpu::RequestAdapterOptions { power_preference : wgpu::PowerPreference::default(), compatible_surface : None, force_fallback_adapter : false })).unwrap();
		let (mut device, mut queue) = futures::executor::block_on(adapter.request_device(& std::default::Default::default(), None)).unwrap();
		let callbacks = crate::Callbacks;
		let mut root_state = crate::pipelines::RootState::new(&mut device, &mut queue);
		let result = crate::pipelines::pipeline_with_cpu_cpu_communication::run(&mut root_state, & callbacks, 1);
		assert_eq!(3, result.field_0);
	}

	#[test]
	fn test_6()
	{
		let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
		let adapter = futures::executor::block_on(instance.request_adapter(& wgpu::RequestAdapterOptions { power_preference : wgpu::PowerPreference::default(), compatible_surface : None, force_fallback_adapter : false })).unwrap();
		let (mut device, mut queue) = futures::executor::block_on(adapter.request_device(& std::default::Default::default(), None)).unwrap();
		let callbacks = crate::Callbacks;
		let mut root_state = crate::pipelines::RootState::new(&mut device, &mut queue);
		let result = crate::pipelines::pipeline_with_single_gpu_call::run(&mut root_state, & callbacks, 1);
		assert_eq!(2, result.field_0);
	}

	#[test]
	fn test_pipeline_with_yield_enter_loop_exit()
	{
		let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
		let adapter = futures::executor::block_on(instance.request_adapter(& wgpu::RequestAdapterOptions { power_preference : wgpu::PowerPreference::default(), compatible_surface : None, force_fallback_adapter : false })).unwrap();
		let (mut device, mut queue) = futures::executor::block_on(adapter.request_device(& std::default::Default::default(), None)).unwrap();
		let callbacks = crate::Callbacks;
		let mut root_state = crate::pipelines::RootState::new(&mut device, &mut queue);
		let mut stage = crate::pipelines::pipeline_with_yield_enter_loop_exit::Instance::new(&mut root_state, & callbacks).start(1).step_7(1);
		assert!(stage.can_step_7());
		let result = stage.step_8().complete();
		assert_eq!(3, result.field_0);
	}


	#[test]
	fn test_pipeline_with_yield_enter_loop_exit_iterated()
	{
		let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
		let adapter = futures::executor::block_on(instance.request_adapter(& wgpu::RequestAdapterOptions { power_preference : wgpu::PowerPreference::default(), compatible_surface : None, force_fallback_adapter : false })).unwrap();
		let (mut device, mut queue) = futures::executor::block_on(adapter.request_device(& std::default::Default::default(), None)).unwrap();
		let callbacks = crate::Callbacks;
		let mut root_state = crate::pipelines::RootState::new(&mut device, &mut queue);
		let mut loop_stage = crate::pipelines::pipeline_with_yield_enter_loop_exit::Instance::new(&mut root_state, & callbacks).start(1).step_7(1);
		let iterations = 16;
		for i in 0 .. iterations
		{
			assert!(loop_stage.can_step_7());
			loop_stage = loop_stage.step_7(1)
		}
		let result = loop_stage.step_8().complete();
		assert_eq!(3 + iterations, result.field_0);
	}*/

	#[test]
	fn test_pipeline_with_value_function()
	{
		use caiman_rt::wgpu;
		let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
		let adapter = futures::executor::block_on(instance.request_adapter(& wgpu::RequestAdapterOptions { power_preference : wgpu::PowerPreference::default(), compatible_surface : None, force_fallback_adapter : false })).unwrap();
		let (mut device, mut queue) = futures::executor::block_on(adapter.request_device(& std::default::Default::default(), None)).unwrap();
		let callbacks = crate::Callbacks;
		let mut root_state = caiman_rt::RootState::new(&mut device, &mut queue);
		let mut join_stack_bytes = [0u8; 4096usize];
		let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
		let instance = crate::pipelines::pipeline_with_value_function::Instance::new(&mut root_state, & callbacks);
		//let result = crate::pipelines::pipeline_1::run(&mut root_state, & callbacks, 1);
		//let result = crate::pipelines::pipeline_with_value_function::funclet13_func(instance, &mut join_stack, 1);
		let result = instance.start(&mut join_stack, 1);
		assert_eq!(2, result.returned().unwrap().0);
	}

	#[test]
	fn test_pipeline_with_looping_value_function()
	{
		use caiman_rt::wgpu;
		let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
		let adapter = futures::executor::block_on(instance.request_adapter(& wgpu::RequestAdapterOptions { power_preference : wgpu::PowerPreference::default(), compatible_surface : None, force_fallback_adapter : false })).unwrap();
		let (mut device, mut queue) = futures::executor::block_on(adapter.request_device(& std::default::Default::default(), None)).unwrap();

		let buffer = device.create_buffer(& wgpu::BufferDescriptor{label : None, size : std::mem::size_of::<i32>() as u64, usage : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ, mapped_at_creation : true});
		buffer.slice(0 ..).get_mapped_range_mut().copy_from_slice(& 0i32.to_ne_bytes());
		buffer.unmap();

		let callbacks = crate::Callbacks;
		let mut root_state = caiman_rt::RootState::new(&mut device, &mut queue);
		let mut join_stack_bytes = [0u8; 4096usize];
		let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
		let instance = crate::pipelines::looping_pipeline::Instance::new(&mut root_state, & callbacks);
		
		let input_ref = caiman_rt::GpuBufferRef::<i32>::new(& buffer, 0);
		let mut result = instance.start(&mut join_stack, input_ref);
		for i in 0 .. 5
		{
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
		let slice = unsafe { std::slice::from_raw_parts(buffer_slice.get_mapped_range().as_ptr(), 4) };
		let final_value : i32 = i32::from_ne_bytes([slice[0], slice[1], slice[2], slice[3]]);
		assert_eq!(6, final_value);
	}

	#[test]
	fn test_scheduling_explication_basic()
	{
		use caiman_rt::wgpu;
		let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
		let adapter = futures::executor::block_on(instance.request_adapter(& wgpu::RequestAdapterOptions { power_preference : wgpu::PowerPreference::default(), compatible_surface : None, force_fallback_adapter : false })).unwrap();
		let (mut device, mut queue) = futures::executor::block_on(adapter.request_device(& std::default::Default::default(), None)).unwrap();
		let callbacks = crate::Callbacks;
		let mut root_state = caiman_rt::RootState::new(&mut device, &mut queue);
		let mut join_stack_bytes = [0u8; 4096usize];
		let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
		let instance = crate::pipelines::pipeline_with_value_function::Instance::new(&mut root_state, & callbacks);
		//let result = crate::pipelines::pipeline_1::run(&mut root_state, & callbacks, 1);
		//let result = crate::pipelines::pipeline_with_value_function::funclet13_func(instance, &mut join_stack, 1);
		let result = instance.start(&mut join_stack, 1);
		assert_eq!(2, result.returned().unwrap().0);
	}
}
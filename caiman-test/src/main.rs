mod pipelines
{
	include!(concat!(env!("OUT_DIR"), "/generated/pipelines.txt"));
}

struct Callbacks;

impl pipelines::pipeline_1::CpuFunctions for Callbacks
{
	fn do_thing_on_cpu( & self, state : &mut pipelines::State, value : i32 ) -> pipelines::pipeline_1::outputs::do_thing_on_cpu
	{
		return pipelines::pipeline_1::outputs::do_thing_on_cpu { field_0 : value + 1 };
	}
}

impl pipelines::pipeline_with_gpu_gpu_communication::CpuFunctions for Callbacks
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
		let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
		let adapter = futures::executor::block_on(instance.request_adapter(& wgpu::RequestAdapterOptions { power_preference : wgpu::PowerPreference::default(), compatible_surface : None, force_fallback_adapter : false })).unwrap();
		let (mut device, mut queue) = futures::executor::block_on(adapter.request_device(& std::default::Default::default(), None)).unwrap();
		let callbacks = crate::Callbacks;
		let mut root_state = crate::pipelines::RootState::new(&mut device, &mut queue);
		let result = crate::pipelines::pipeline_1::run(&mut root_state, & callbacks, 1);
		assert_eq!(3, result.field_0);
	}

	#[test]
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
		let result = crate::pipelines::pipeline_with_yield_enter_loop_exit::Funclet6::new(&mut root_state, & callbacks, 1).step_7(1).step_8().complete();
		assert_eq!(3, result.field_0);
	}
}
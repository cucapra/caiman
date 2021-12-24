mod pipelines
{
	include!(concat!(env!("OUT_DIR"), "/generated/pipelines.txt"));
}

struct Callbacks;

impl pipelines::pipeline_1::CpuFunctions for Callbacks
{
	fn do_thing_on_cpu( & self, value : i32 ) -> pipelines::pipeline_1::outputs::do_thing_on_cpu
	{
		return pipelines::pipeline_1::outputs::do_thing_on_cpu { field_0 : value + 1 };
	}
}

impl pipelines::pipeline_with_gpu_gpu_communication::CpuFunctions for Callbacks
{
	fn do_thing_on_cpu( & self, value : i32 ) -> pipelines::pipeline_with_gpu_gpu_communication::outputs::do_thing_on_cpu
	{
		return pipelines::pipeline_with_gpu_gpu_communication::outputs::do_thing_on_cpu { field_0 : value + 1 };
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
		let result = crate::pipelines::pipeline_1::run(&mut device, &mut queue, & callbacks, 1);
		assert_eq!(3, result.field_0);
	}

	#[test]
	fn test_2()
	{
		let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
		let adapter = futures::executor::block_on(instance.request_adapter(& wgpu::RequestAdapterOptions { power_preference : wgpu::PowerPreference::default(), compatible_surface : None, force_fallback_adapter : false })).unwrap();
		let (mut device, mut queue) = futures::executor::block_on(adapter.request_device(& std::default::Default::default(), None)).unwrap();
		let callbacks = crate::Callbacks;
		let result = crate::pipelines::pipeline_with_gpu_gpu_communication::run(&mut device, &mut queue, & callbacks, 1);
		assert_eq!(3, result.field_0);
	}
}
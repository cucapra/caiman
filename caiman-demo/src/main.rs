


mod pipelines
{
	struct Output
	{
		field_0 : i32
	}
	
	fn do_thing_on_cpu( value : i32 ) -> Output
	{
		return Output { field_0 : value };
	}

	include!(concat!(env!("OUT_DIR"), "/generated/pipelines.txt"));
}

fn main()
{
	let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
	let adapter = futures::executor::block_on(instance.request_adapter(& wgpu::RequestAdapterOptions { power_preference : wgpu::PowerPreference::default(), compatible_surface : None, force_fallback_adapter : false })).unwrap();
	let (mut device, mut queue) = futures::executor::block_on(adapter.request_device(& std::default::Default::default(), None)).unwrap();
	pipelines::pipeline_0(&mut device, &mut queue, 1);
}

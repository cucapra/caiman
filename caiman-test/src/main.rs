
mod values
{
	include!(concat!(env!("OUT_DIR"), "/generated/trivial.txt"));
	include!(concat!(env!("OUT_DIR"), "/generated/schedule_trivial.txt"));
}

struct Callbacks;


impl values::pipeline_trivial::CpuFunctions for Callbacks
{
	fn do_thing_on_cpu(& self, state : &mut dyn caiman_rt::State, value : i32 )
					   -> values::pipeline_trivial::outputs::do_thing_on_cpu
	{
		return ( value + 1, );
	}
}

impl values::pipeline_schedule_trivial::CpuFunctions for Callbacks
{
	fn do_thing_on_cpu(& self, state : &mut dyn caiman_rt::State, value : i32 )
					   -> values::pipeline_schedule_trivial::outputs::do_thing_on_cpu
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
		let adapter = futures::executor::block_on(
			instance.request_adapter(& wgpu::RequestAdapterOptions
				{
					power_preference : wgpu::PowerPreference::default(),
					compatible_surface : None,
					force_fallback_adapter : false
				})).unwrap();
		let (mut device, mut queue) = futures::executor::block_on(
			adapter.request_device(& std::default::Default::default(), None)
		).unwrap();
		let callbacks = crate::Callbacks;
		let mut root_state = caiman_rt::RootState::new(&mut device, &mut queue);
		let mut join_stack_bytes = [0u8; 4096usize];
		let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
		//let result = crate::values::pipeline_trivial::run(&mut root_state, & callbacks, 1);
		let instance = crate::values::pipeline_trivial::Instance::new(&mut root_state, & callbacks);
		let result = instance.start(&mut join_stack, 1);
		//let result = crate::values::pipeline_trivial::funclet11_func(instance, &mut join_stack, 1);
		assert_eq!(4, result.returned().unwrap().0);
	}

	#[test]
	fn test_2()
	{
		use caiman_rt::wgpu;
		let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
		let adapter = futures::executor::block_on(
			instance.request_adapter(& wgpu::RequestAdapterOptions
				{
					power_preference : wgpu::PowerPreference::default(),
					compatible_surface : None,
					force_fallback_adapter : false
				})).unwrap();
		let (mut device, mut queue) = futures::executor::block_on(
			adapter.request_device(& std::default::Default::default(), None)
		).unwrap();
		let callbacks = crate::Callbacks;
		let mut root_state = caiman_rt::RootState::new(&mut device, &mut queue);
		let mut join_stack_bytes = [0u8; 4096usize];
		let mut join_stack = caiman_rt::JoinStack::new(&mut join_stack_bytes);
		//let result = crate::values::pipeline_trivial::run(&mut root_state, & callbacks, 1);
		let instance = crate::values::pipeline_schedule_trivial::Instance::new(&mut root_state, & callbacks);
		let result = instance.start(&mut join_stack, 1);
		//let result = crate::values::pipeline_trivial::funclet11_func(instance, &mut join_stack, 1);
		assert_eq!(3, result.returned().unwrap().0);
	}
}
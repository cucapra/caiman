pub extern crate wgpu;

pub trait State
{
	fn get_device_mut(&mut self) -> &mut wgpu::Device;
	fn get_queue_mut(&mut self) -> &mut wgpu::Queue;
}

pub struct RootState<'device, 'queue>
{
	device : & 'device mut wgpu::Device,
	queue : & 'queue mut wgpu::Queue
}

impl<'device, 'queue> RootState<'device, 'queue>
{
	pub fn new(
		device : & 'device mut wgpu::Device,
		queue : & 'queue mut wgpu::Queue) -> Self
	{
		Self{device, queue}
	}
}

impl<'device, 'queue> State for RootState<'device, 'queue>
{
	fn get_device_mut(&mut self) -> &mut wgpu::Device
	{
		self.device
	}

	fn get_queue_mut(&mut self) -> &mut wgpu::Queue
	{
		self.queue
	}
}

#[cfg(test)]
	mod tests {
	#[test]
	fn it_works() {
		let result = 2 + 2;
		assert_eq!(result, 4);
	}
}

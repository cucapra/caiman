use wgpu;

trait Pass
{
	fn encode(&mut self, device : &mut wgpu::Device, queue : &mut wgpu::Queue, encoder : &mut wgpu::CommandEncoder);
}


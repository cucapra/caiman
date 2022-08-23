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

pub struct TypeLayout
{
	pub byte_size : usize,
	pub alignment : usize
}

pub struct GpuBufferAllocator<'buffer>
{
	pub buffer : & 'buffer wgpu::Buffer,
	pub base_address : usize,
	pub size : usize
}

impl<'buffer> GpuBufferAllocator<'buffer>
{
	fn suballocate(&mut self, byte_size : usize, alignment : usize) -> Option<wgpu::BufferAddress>
	{
		if byte_size > self.size
		{
			return None;
		}

		let start_address = {
			let address = self.base_address;
			let aligned_floor_address = (self.base_address / alignment) * alignment;
			let remainder = self.base_address - aligned_floor_address;
			if remainder > 0
			{
				// Overflow-safe version of aligned_floor_address + alignment <= u64::MAX
				if aligned_floor_address <= usize::MAX - alignment
				{
					aligned_floor_address + alignment
				}
				else
				{
					return None;
				}
			}
			else
			{
				aligned_floor_address
			}
		};

		/*let start_address =
			if let Some(start_address) = self.base_address.checked_next_multiple_of(alignment)
			{
				start_address
			}
			else
			{
				return None;
			};*/

		// Overflow-safe version of self.byte_size + start_address > self.base_address + self.size
		if start_address - self.base_address > self.size - byte_size
		{
			return None;
		}

		let start_address_opt : Option<wgpu::BufferAddress> = start_address.try_into().ok();
		if start_address_opt.is_none()
		{
			return None;
		}

		self.size -= start_address - self.base_address + byte_size;
		self.base_address = start_address + byte_size;

		start_address_opt
	}

	pub fn suballocate_ref<T : Sized>(&mut self, type_layout : & TypeLayout) -> Option<GpuBufferRef<'buffer, T>>
	{
		// Need to check that layout (generated at caiman compile time) agrees with the layout at rust compile time
		assert_eq!(type_layout.byte_size, std::mem::size_of::<T>());
		assert_eq!(type_layout.alignment, std::mem::align_of::<T>());

		if let Some(starting_address) = self.suballocate(type_layout.byte_size, type_layout.alignment)
		{
			return Some(GpuBufferRef::new(self.buffer, starting_address));
		}

		return None;
	}

	pub fn suballocate_slice<T : Sized>(&mut self, element_type_layout : & TypeLayout, count : usize) -> Option<GpuBufferSlice<'buffer, T>>
	{
		// Need to check that layout (generated at caiman compile time) agrees with the layout at rust compile time
		assert_eq!(element_type_layout.byte_size, std::mem::size_of::<T>());
		assert_eq!(element_type_layout.alignment, std::mem::align_of::<T>());
		let (byte_size, overflowed) = element_type_layout.byte_size.overflowing_mul(count);

		if overflowed
		{
			return None;
		}

		if let Some(starting_address) = self.suballocate(byte_size, element_type_layout.alignment)
		{
			return Some(GpuBufferSlice::new(self.buffer, starting_address, Some(std::num::NonZeroU64::new(byte_size.try_into().unwrap()).unwrap())));
		}
		
		return None;
	}

	// A very horribly implemented check
	pub fn test_suballocate_many(& self, layouts : &[TypeLayout], element_counts : &[Option<usize>]) -> usize
	{
		let mut self_copy = Self{buffer : self.buffer, base_address : self.base_address, size : self.size};

		let mut success_count = 0usize;

		for (i, layout) in layouts.iter().enumerate()
		{
			let can_allocate =
				if let Some(element_count) = element_counts[i]
				{
					let (byte_size, overflowed) = layout.byte_size.overflowing_mul(element_count);
					!overflowed && self_copy.suballocate(byte_size, layout.alignment).is_some()
				}
				else
				{
					self_copy.suballocate(layout.byte_size, layout.alignment).is_some()
				};

			if ! can_allocate
			{
				return success_count;
			}

			success_count += 1usize;
		}

		success_count
	}
}

// A slot holding a pointer to gpu-resident data of type T
pub struct GpuBufferRef<'buffer, T : Sized>
{
	phantom : std::marker::PhantomData<*const T>,
	pub buffer : & 'buffer wgpu::Buffer,
	pub base_address : wgpu::BufferAddress,
	//offset : wgpu::DynamicOffset,
}

impl<'buffer, T : Sized> GpuBufferRef<'buffer, T>
{
	pub fn new(buffer : & 'buffer wgpu::Buffer, base_address : wgpu::BufferAddress) -> Self
	{
		Self { phantom : std::marker::PhantomData, buffer, base_address }
	}

	pub fn as_binding_resource(&self) -> wgpu::BindingResource<'buffer>
	{
		wgpu::BindingResource::Buffer(wgpu::BufferBinding{buffer : self.buffer, offset : self.base_address, size : std::num::NonZeroU64::new(std::mem::size_of::<T>().try_into().unwrap())})
	}

	pub fn slice(&self) -> wgpu::BufferSlice<'buffer>
	{
		// Technically, this could overflow?
		let mut end_address = self.base_address;
		// Rust needs the type hint...
		let size_opt : Option<wgpu::BufferAddress> = std::mem::size_of::<T>().try_into().ok();
		end_address += size_opt.unwrap();
		self.buffer.slice(self.base_address .. end_address)
	}
}

// A slot holding a pointer to gpu-resident array of elements of type T
pub struct GpuBufferSlice<'buffer, T : Sized>
{
	phantom : std::marker::PhantomData<*const T>,
	pub buffer : & 'buffer wgpu::Buffer,
	pub base_address : wgpu::BufferAddress,
	//offset : wgpu::DynamicOffset,
	pub size_opt : Option<wgpu::BufferSize>
}

impl<'buffer, T : Sized> GpuBufferSlice<'buffer, T>
{
	pub fn new(buffer : & 'buffer wgpu::Buffer, base_address : wgpu::BufferAddress, size_opt : Option<wgpu::BufferSize>) -> Self
	{
		Self { phantom : std::marker::PhantomData, buffer, base_address, size_opt }
	}

	pub fn as_binding_resource(&self) -> wgpu::BindingResource<'buffer>
	{
		wgpu::BindingResource::Buffer(wgpu::BufferBinding{buffer : self.buffer, offset : self.base_address, size : self.size_opt})
	}

	pub fn slice(&self) -> wgpu::BufferSlice<'buffer>
	{
		if let Some(size) = self.size_opt
		{
			// Technically, this could overflow?
			let mut end_address = self.base_address;
			// Rust needs the type hint...
			let size_opt : Option<wgpu::BufferAddress> = size.try_into().ok();
			end_address += size_opt.unwrap();
			self.buffer.slice(self.base_address .. end_address)
		}
		else
		{
			self.buffer.slice(self.base_address..)
		}
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

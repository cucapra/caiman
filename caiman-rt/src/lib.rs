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

#[derive(Clone)]
struct AbstractAllocator
{
	base_address : usize,
	size : usize
}

impl AbstractAllocator
{
	fn suballocate(&mut self, byte_size : usize, alignment : usize) -> Option<usize>
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

		self.size -= start_address - self.base_address + byte_size;
		self.base_address = start_address + byte_size;

		Some(start_address)
	}

	fn suballocate_erased_ref(&mut self, type_layout : & TypeLayout) -> Option<usize>
	{
		self.suballocate(type_layout.byte_size, type_layout.alignment)
	}

	fn suballocate_erased_slice(&mut self, element_type_layout : & TypeLayout, count : usize) -> Option<(usize, usize)>
	{
		let (byte_size, overflowed) = element_type_layout.byte_size.overflowing_mul(count);

		if overflowed
		{
			return None;
		}

		self.suballocate(byte_size, element_type_layout.alignment).map(|address| (address, byte_size))
	}

	fn type_layout_of<T : Sized>() -> TypeLayout
	{
		TypeLayout{byte_size : std::mem::size_of::<T>(), alignment : std::mem::align_of::<T>()}
	}

	fn suballocate_ref<T : Sized>(&mut self) -> Option<usize>
	{
		self.suballocate_erased_ref(& Self::type_layout_of::<T>())
	}

	fn suballocate_slice<T : Sized>(&mut self, count : usize) -> Option<(usize, usize)>
	{
		return self.suballocate_erased_slice(& Self::type_layout_of::<T>(), count);
	}
}

pub struct CpuBufferAllocator<'buffer>
{
	bytes : & 'buffer [u8],
	abstract_allocator : AbstractAllocator
}

impl<'buffer> CpuBufferAllocator<'buffer>
{
	pub fn suballocate_ref<T : Sized>(&mut self) -> Option<& 'buffer mut std::mem::MaybeUninit<T>>
	{
		if let Some(starting_address) = self.abstract_allocator.suballocate_ref::<T>()
		{
			return unsafe
			{
				let bytes_pointer = std::mem::transmute::<& u8, * mut u8>(& self.bytes[starting_address]);
				//let allocation_bytes_pointer = bytes_pointer.offset(starting_address);
				Some(std::mem::transmute::<* mut u8, & 'buffer mut std::mem::MaybeUninit<T>>(bytes_pointer))
			}
		}

		return None;
	}

	pub fn suballocate_slice<T : Sized>(&mut self, count : usize) -> Option<& 'buffer mut [std::mem::MaybeUninit<T>]>
	{
		if let Some((starting_address, byte_size)) = self.abstract_allocator.suballocate_slice::<T>(count)
		{
			return unsafe
			{
				let bytes_pointer = std::mem::transmute::<& u8, * mut u8>(& self.bytes[starting_address]);
				//let allocation_bytes_pointer = bytes_pointer.offset(starting_address);
				let allocation_base_pointer = std::mem::transmute::<* mut u8, * mut std::mem::MaybeUninit<T>>(bytes_pointer);
				Some(std::slice::from_raw_parts_mut::<'buffer, std::mem::MaybeUninit<T>>(allocation_base_pointer, count))
			}
		}

		return None;
	}
}

pub struct GpuBufferAllocator<'buffer>
{
	buffer : & 'buffer wgpu::Buffer,
	abstract_allocator : AbstractAllocator
}

impl<'buffer> GpuBufferAllocator<'buffer>
{
	pub fn suballocate_ref<T : Sized>(&mut self) -> Option<GpuBufferRef<'buffer, T>>
	{
		if let Some(starting_address) = self.abstract_allocator.suballocate_ref::<T>()
		{
			return Some(GpuBufferRef::new(self.buffer, starting_address.try_into().unwrap()));
		}

		return None;
	}

	pub fn suballocate_slice<T : Sized>(&mut self, count : usize) -> Option<GpuBufferSlice<'buffer, T>>
	{
		if let Some((starting_address, byte_size)) = self.abstract_allocator.suballocate_slice::<T>(count)
		{
			return Some(GpuBufferSlice::new(self.buffer, starting_address.try_into().unwrap(), Some(std::num::NonZeroU64::new(byte_size.try_into().unwrap()).unwrap())));
		}
		
		return None;
	}

	// A very horribly implemented check
	pub fn test_suballocate_many(& self, layouts : &[TypeLayout], element_counts : &[Option<usize>]) -> usize
	{
		let mut abstract_allocator = self.abstract_allocator.clone();

		let mut success_count = 0usize;

		for (i, layout) in layouts.iter().enumerate()
		{
			let can_allocate =
				if let Some(element_count) = element_counts[i]
				{
					abstract_allocator.suballocate_erased_slice(layout, element_count).is_some()
				}
				else
				{
					abstract_allocator.suballocate_erased_ref(layout).is_some()
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

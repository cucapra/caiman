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

#[derive(Debug, Clone)]
struct AbstractAllocator
{
	base_address : usize,
	size : usize
}

impl AbstractAllocator
{
	fn new(size : usize) -> Self
	{
		Self { base_address : 0, size }
	}

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
	bytes : & 'buffer mut [u8],
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

#[derive(Debug)]
pub struct JoinStack<'buffer>
{
	bytes : & 'buffer mut [u8],
	abstract_allocator : AbstractAllocator
}

impl<'buffer> JoinStack<'buffer>
{
	pub fn new(bytes : & 'buffer mut [u8]) -> Self
	{
		let abstract_allocator = AbstractAllocator::new(bytes.len());
		Self { bytes, abstract_allocator }
	}

	pub fn used_bytes(& self) -> & [u8]
	{
		& self.bytes[ 0 .. self.abstract_allocator.base_address ]
	}

	pub fn unused_bytes(& self) -> & [u8]
	{
		// To do: Check for overflow?
		& self.bytes[ self.abstract_allocator.base_address .. (self.abstract_allocator.base_address + self.abstract_allocator.size) ]
	}

	pub fn push_raw(& mut self, bytes : & [u8]) -> Option<& 'buffer mut [u8]>
	{
		if let Some(starting_offset) = self.abstract_allocator.suballocate(bytes.len(), 1)
		{
			return unsafe
			{
				let bytes_pointer = std::mem::transmute::<& u8, * mut u8>(& self.bytes[starting_offset]);
				let destination_subslice = std::slice::from_raw_parts_mut::<'buffer, u8>(bytes_pointer, bytes.len());
				destination_subslice.copy_from_slice(bytes);
				Some(destination_subslice)
			}
		}

		return None;
	}

	pub unsafe fn push_unsafe_unaligned<T : Sized>(&mut self, data : T) -> Result<& 'buffer mut [u8], T>
	{
		let bytes = unsafe
		{
			let bytes_pointer = std::mem::transmute::<& T, * const u8>(& data);
			std::slice::from_raw_parts::<'buffer, u8>(bytes_pointer, std::mem::size_of::<T>())
		};

		if let Some(slice) = self.push_raw(bytes)
		{
			std::mem::forget(data);
			Result::Ok(slice)
		}
		else
		{
			Result::Err(data)
		}
	}

	pub fn peek_raw(& self, size : usize) -> Option<& [u8]>
	{
		if self.abstract_allocator.base_address < size
		{
			return None;
		}

		let new_base_address = self.abstract_allocator.base_address - size;

		return Some(& self.bytes[new_base_address .. self.abstract_allocator.base_address]);
	}

	pub fn pop_raw(&mut self, size : usize) -> Option<& [u8]>
	{
		if self.abstract_allocator.base_address < size
		{
			return None;
		}

		let old_base_address = self.abstract_allocator.base_address;
		self.abstract_allocator.base_address -= size;
		// To do: Theoretically we can get overflow for size here if we ever allow initializing at a nonzero base address
		self.abstract_allocator.size += size;

		return Some(& self.bytes[self.abstract_allocator.base_address .. old_base_address]);
	}

	pub unsafe fn pop_unsafe_unaligned<T : Sized>(&mut self) -> Option<T>
	{
		let size = std::mem::size_of::<T>();
		if let Some(source_bytes) = self.pop_raw(size)
		{
			let mut uninit_data : std::mem::MaybeUninit<T> = unsafe { std::mem::uninitialized() };
			let destination_bytes_pointer = std::mem::transmute::<& mut std::mem::MaybeUninit<T>, * mut u8>(&mut uninit_data);
			let destination_subslice = std::slice::from_raw_parts_mut::<'buffer, u8>(destination_bytes_pointer, size);
			destination_subslice.copy_from_slice(source_bytes);
			//let uninit_data_ref = std::mem::transmute::<& u8, & std::mem::MaybeUninit<T>>(& destination_subslice[0]);
			return Some(uninit_data.assume_init());
		}

		return None;
	}
}

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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

/*pub struct SerializedGpuBufferSlot
{
	pub offset : usize,
	pub type_layout : TypeLayout,
	pub size : usize
}*/

/*pub struct SerializedGpuSlot
{
	
}

pub enum SerializedSlot
{
	Cpu,
	Gpu,
}*/

/*pub struct CpuJoinGraph<'buffer>
{
	cpu_buffer_allocator : CpuBufferAllocator<'buffer>
}

impl<'buffer> CpuJoinGraph<'buffer>
{

}*/

/*pub enum CapturedSlot<'buffer>
{
	GpuBufferRef(GpuBufferRef<'buffer>),
	GpuBufferSlice(GpuBufferSlice<'buffer>),
}*/

/*pub struct SerializedGpuBufferSlotIterator<'buffer>
{

}*/

/*impl<'buffer> std::iter::Iterator for SerializedGpuBufferSlotIterator<'buffer>
{
	type Item = SerializedGpuBufferSlot;
	
	fn next(&mut self) -> Option<Self::Item>
	{

	}
}*/

/*pub struct JoinPointSpace<'join_point>
{
	gpu_slot_slice : & 'join_point mut [SerializedGpuBufferSlot]
}

impl<'join_point> JoinPointSpace<'buffer, 'join_point>
{
	fn gpu_slots(& 'join_point mut self) -> & 'join_point mut [SerializedGpuBufferSlot]
	{
		self.gpu_slot_slice
	}
}

pub trait JoinPoint<'buffer>
{
	//fn next(&self) -> Option<& 'buffer dyn JoinPoint<'buffer>>;
	//fn gpu_slots(& 'self self) -> SerializedGpuBufferSlotIterator<'buffer>;
	fn spaces<'self_lifetime>(& 'self_lifetime mut self) -> & 'self_lifetime mut [JoinPointSpace<'self_lifetime>];
}*/

/*impl<'buffer> std::iter::Iterator for SerializedGpuBufferSlotIterator<'buffer>
{
	type Item = SerializedGpuBufferSlot;
	
	fn next(&mut self) -> Option<Self::Item>
	{

	}
}*/

/*pub trait JoinPointGraph<'buffer>
{
	type JoinPointIterator : std::iter::Iterator<Item = dyn JoinPoint<'buffer>>;
	fn join_points(& mut self) -> Self::JoinPointIterator;
}*/

/*pub struct BufferSlotIterator
{

}*/

/*struct JoinGraphEntryHeader
{
	entry_byte_size : u16,
	funclet_id : u16,
}*/



#[cfg(test)]
	mod tests {
	#[test]
	fn it_works() {
		let result = 2 + 2;
		assert_eq!(result, 4);
	}
}

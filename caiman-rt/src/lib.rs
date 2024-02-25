#![allow(warnings)]

use core::slice;
use std::{
    alloc::Layout, any::Any, collections::HashMap, fmt::Alignment, marker::PhantomData,
    mem::MaybeUninit, os::raw::c_void,
};

use wgpu::Buffer;
pub extern crate bytemuck;
pub extern crate wgpu;

// None = waits on whole queue
pub type GpuFence = Option<wgpu::SubmissionIndex>;

/// Manages the allocation of variables in a contiguous buffer,
/// respecting the alignment requirements of each variable.
struct BumpAllocator {
    buffer_align: usize,
    // next available address in the buffer
    next_address: usize,
    // map from variable ids to their starting addresses
    address_map: HashMap<usize, BumpAddr>,
    buffer_len: usize,
}

/// An offset off of the start of the buffer.
#[derive(Debug, Copy, Clone)]
struct BumpAddr(usize);

const DEFAULT_ALIGN: usize = 16;

impl BumpAllocator {
    /// Creates a new allocator with the given buffer size and alignment of
    /// the buffer itself.
    fn new(size: usize, align: usize) -> Self {
        Self {
            next_address: 0,
            address_map: HashMap::new(),
            buffer_len: size,
            buffer_align: align,
        }
    }

    /// Allocates a new variable with the given id, size, and alignment.
    /// The alignment must be less than or equal to `MAX_ALIGN`.
    fn alloc(&mut self, id: usize, size: usize, align: usize) -> BumpAddr {
        let next_aligned_addr = ((self.next_address + self.buffer_align + align - 1)
            & !(align - 1))
            - self.buffer_align;
        assert!(next_aligned_addr + size <= self.buffer_len);
        self.next_address = next_aligned_addr + size;
        let addr = BumpAddr(next_aligned_addr);
        self.address_map.insert(id, addr);
        addr
    }

    /// Gets a pointer to the start of the allocation for the given id.
    /// Requires that the allocation exists and that usage of the pointer is safe.
    fn get_starting_addr(&self, id: usize) -> BumpAddr {
        *self.address_map.get(&id).unwrap()
    }

    /// Resets the allocator to the empty state, essentially erase all allocations.
    fn reset(&mut self) {
        self.next_address = 0;
        self.address_map.clear();
    }
}

/// Allocates data on the CPU.
struct CpuAllocator {
    buffer: Box<[u8]>,
    allocator: BumpAllocator,
}

impl CpuAllocator {
    pub fn new(size: usize) -> Self {
        Self {
            buffer: unsafe {
                Box::from_raw(slice::from_raw_parts_mut(
                    std::alloc::alloc_zeroed(Layout::from_size_align(size, DEFAULT_ALIGN).unwrap()),
                    size,
                ))
            },
            allocator: BumpAllocator::new(size, DEFAULT_ALIGN),
        }
    }

    /// Allocates a local variable with the given id, size, and alignment
    /// and returns a mutable reference to it.
    ///
    /// Panics if the buffer is full
    pub fn alloc(&mut self, id: usize, size: usize, align: usize) -> *mut c_void {
        self.allocator.alloc(id, size, align);
        self.get_ptr_mut(id)
    }

    /// Gets the starting address of the allocation for the given id.
    pub fn get_ptr(&self, id: usize) -> *const c_void {
        unsafe {
            self.buffer
                .as_ptr()
                .add(self.allocator.get_starting_addr(id).0) as *const c_void
        }
    }

    /// Gets the starting address of the allocation for the given id.
    pub fn get_ptr_mut(&mut self, id: usize) -> *mut c_void {
        unsafe {
            self.buffer
                .as_mut_ptr()
                .add(self.allocator.get_starting_addr(id).0) as *mut c_void
        }
    }

    pub fn reset(&mut self) {
        self.allocator.reset();
    }
}

/// Allocates data on the GPU.
struct GpuAllocator {
    buffer: Buffer,
    allocator: BumpAllocator,
}

impl GpuAllocator {
    pub fn new(state: &mut dyn State, usage: wgpu::BufferUsages) -> Self {
        const BUFFER_SIZE: u64 = 4096;
        Self {
            buffer: state
                .get_device_mut()
                .create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size: BUFFER_SIZE,
                    usage,
                    mapped_at_creation: false,
                }),
            allocator: BumpAllocator::new(BUFFER_SIZE as usize, 0),
        }
    }

    pub fn alloc(&mut self, id: usize, size: usize, align: usize) {
        self.allocator.alloc(id, size, align);
    }

    pub fn get_buffer_ref<T: Sized + Any>(&self, id: usize) -> GpuBufferRef<'_, T> {
        let addr = self.allocator.get_starting_addr(id);
        GpuBufferRef::new(&self.buffer, addr.0 as wgpu::BufferAddress)
    }

    pub fn reset(&mut self) {
        self.allocator.reset();
    }
}

pub trait State {
    fn get_device_mut(&mut self) -> &mut wgpu::Device;
    fn get_queue_mut(&mut self) -> &mut wgpu::Queue;
}

pub struct RootState<'device, 'queue> {
    device: &'device mut wgpu::Device,
    queue: &'queue mut wgpu::Queue,
    local_storage: CpuAllocator,
}

impl<'device, 'queue> RootState<'device, 'queue> {
    pub fn new(device: &'device mut wgpu::Device, queue: &'queue mut wgpu::Queue) -> Self {
        Self {
            device,
            queue,
            local_storage: CpuAllocator::new(4096 * 4),
        }
    }
}

impl<'device, 'queue> State for RootState<'device, 'queue> {
    fn get_device_mut(&mut self) -> &mut wgpu::Device {
        self.device
    }

    fn get_queue_mut(&mut self) -> &mut wgpu::Queue {
        self.queue
    }
}

/// Manages the allocation of local variables on the CPU.
/// The variables are allocated in a contiguous buffer, respecting their alignment requirements.
pub struct LocalVars {
    storage: CpuAllocator,
    // maps variable ids to their type ids. Used only for runtime checks
    type_ids: HashMap<usize, std::any::TypeId>,
}

const GPU_BUFFERS: usize = 5;

/// Manages the allocation of local variables on the GPU.
/// Kept separate from `LocalVars` to make it possible to have a GPU and CPU
/// reference live at the same time (ie. mutable borrow from two different
/// objects instead of one).
pub struct GpuLocals {
    // one allocator for each buffer usage, index of buffer usage
    // maps to the index of the allocator
    gpu_allocators: [GpuAllocator; GPU_BUFFERS],
    usages: [wgpu::BufferUsages; GPU_BUFFERS],
    // maps variable ids to the index of the gpu allocator that holds them
    alloc_map: HashMap<usize, usize>,
    // maps variable ids to their type ids. Used only for runtime checks
    type_ids: HashMap<usize, std::any::TypeId>,
}

impl LocalVars {
    pub fn new() -> Self {
        const LOCAL_BUF_SIZE: usize = 4096 * 4;
        Self {
            storage: CpuAllocator::new(LOCAL_BUF_SIZE),
            type_ids: HashMap::new(),
        }
    }

    /// Helper function to allocate a CPU local variable with the given id, size, and alignment.
    /// Returns a mutable reference to the allocated memory.
    fn alloc_uninit<T: Sized + Any>(&mut self, id: usize) -> &mut std::mem::MaybeUninit<T> {
        let align = std::mem::align_of::<T>();
        let size = std::mem::size_of::<T>();
        let type_id = std::any::TypeId::of::<T>();
        self.type_ids.insert(id, type_id);
        let mut r = unsafe {
            self.storage
                .alloc(id, size, align)
                .cast::<std::mem::MaybeUninit<T>>()
                .as_mut()
                .unwrap()
        };
        r
    }

    /// Allocates a CPU local variable with the given id, size, and alignment.
    /// The variable is initialized with the given value.
    pub fn calloc<T: Sized + Any>(&mut self, id: usize, val: T) -> &mut T {
        let mut r = self.alloc_uninit::<T>(id);
        r.write(val);
        unsafe { r.assume_init_mut() }
    }

    /// Allocates a CPU local variable with the given id, size, and alignment.
    /// Initializes the variable with the default value of the type.
    pub fn malloc<T: Sized + Any + Default>(&mut self, id: usize) -> &mut T {
        self.calloc(id, Default::default())
    }

    /// Gets a CPU mutable pointer to the start of the allocation for the given id.
    /// The allocation must have already been created with `alloc_var`
    pub fn get_mut<T: Sized + Any>(&mut self, id: usize) -> &mut T {
        assert_eq!(
            self.type_ids.get(&id).unwrap(),
            &std::any::TypeId::of::<T>()
        );
        unsafe {
            (self.storage.get_ptr_mut(id) as *mut c_void)
                .cast::<T>()
                .as_mut()
                .unwrap()
        }
    }
    /// Gets a CPU const pointer to the start of the allocation for the given id.
    /// The allocation must have already been created with `alloc_var`
    pub fn get<T: Sized + Any>(&self, id: usize) -> &T {
        assert_eq!(
            self.type_ids.get(&id).unwrap(),
            &std::any::TypeId::of::<T>()
        );
        unsafe {
            (self.storage.get_ptr(id) as *const c_void)
                .cast::<T>()
                .as_ref()
                .unwrap()
        }
    }

    /// Clears all allocations and resets the allocator to the empty state.
    pub fn reset(&mut self) {
        self.storage.reset();
        self.type_ids.clear();
    }
}

impl GpuLocals {
    pub fn new(state: &mut dyn State) -> Self {
        let usages = [
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_WRITE,
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
            wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
            wgpu::BufferUsages::MAP_READ
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::STORAGE,
        ];
        Self {
            gpu_allocators: [
                GpuAllocator::new(state, usages[0]),
                GpuAllocator::new(state, usages[1]),
                GpuAllocator::new(state, usages[2]),
                GpuAllocator::new(state, usages[3]),
                GpuAllocator::new(state, usages[4]),
            ],
            usages,
            alloc_map: HashMap::new(),
            type_ids: HashMap::new(),
        }
    }

    /// Clears all allocations and resets the allocator to the empty state.
    pub fn reset(&mut self) {
        self.alloc_map.clear();
        self.type_ids.clear();
        for gpu_alloc in self.gpu_allocators.iter_mut() {
            gpu_alloc.reset();
        }
    }

    /// Allocates a GPU local variable with the given id, size, and alignment.
    pub fn alloc_gpu<T: Sized + Any>(&mut self, id: usize, usage: wgpu::BufferUsages) {
        for (idx, u) in self.usages.iter().enumerate() {
            if u.contains(usage) {
                self.alloc_map.insert(id, idx);
                self.type_ids.insert(id, std::any::TypeId::of::<T>());
                self.gpu_allocators[idx].alloc(
                    id,
                    std::mem::size_of::<T>(),
                    std::mem::align_of::<T>()
                        .max(wgpu::Limits::default().min_storage_buffer_offset_alignment as usize),
                );
                return;
            }
        }
        panic!("No suitable GPU buffer usage found for {:?}", usage);
    }

    /// Gets a GPU mutable pointer to the start of the allocation for the given id.
    pub fn get_gpu_ref<T: Sized + Any>(&self, id: usize) -> GpuBufferRef<'_, T> {
        assert_eq!(
            self.type_ids.get(&id).unwrap(),
            &std::any::TypeId::of::<T>()
        );
        let idx = self.alloc_map.get(&id).unwrap();
        self.gpu_allocators[*idx].get_buffer_ref(id)
    }
}

pub struct TypeLayout {
    pub byte_size: usize,
    pub alignment: usize,
}

#[derive(Debug, Clone)]
struct AbstractAllocator {
    base_address: usize,
    size: usize,
}

impl AbstractAllocator {
    fn new(size: usize) -> Self {
        Self {
            base_address: 0,
            size,
        }
    }

    fn suballocate(&mut self, byte_size: usize, alignment: usize) -> Option<usize> {
        if byte_size > self.size {
            return None;
        }

        let start_address = {
            let address = self.base_address;
            let aligned_floor_address = (self.base_address / alignment) * alignment;
            let remainder = self.base_address - aligned_floor_address;
            if remainder > 0 {
                // Overflow-safe version of aligned_floor_address + alignment <= u64::MAX
                if aligned_floor_address <= usize::MAX - alignment {
                    aligned_floor_address + alignment
                } else {
                    return None;
                }
            } else {
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
        if start_address - self.base_address > self.size - byte_size {
            return None;
        }

        self.size -= start_address - self.base_address + byte_size;
        self.base_address = start_address + byte_size;
        Some(start_address)
    }

    fn suballocate_erased_ref(&mut self, type_layout: &TypeLayout) -> Option<usize> {
        // todo: magic number 64 is overkill and should be fixed with the limit thing
        self.suballocate(type_layout.byte_size, type_layout.alignment * 64)
    }

    fn suballocate_erased_slice(
        &mut self,
        element_type_layout: &TypeLayout,
        count: usize,
    ) -> Option<(usize, usize)> {
        let (byte_size, overflowed) = element_type_layout.byte_size.overflowing_mul(count);

        if overflowed {
            return None;
        }

        self.suballocate(byte_size, element_type_layout.alignment)
            .map(|address| (address, byte_size))
    }

    fn type_layout_of<T: Sized>() -> TypeLayout {
        TypeLayout {
            byte_size: std::mem::size_of::<T>(),
            alignment: std::mem::align_of::<T>(),
        }
    }

    fn suballocate_ref<T: Sized>(&mut self) -> Option<usize> {
        self.suballocate_erased_ref(&Self::type_layout_of::<T>())
    }

    fn suballocate_slice<T: Sized>(&mut self, count: usize) -> Option<(usize, usize)> {
        return self.suballocate_erased_slice(&Self::type_layout_of::<T>(), count);
    }
}

pub struct CpuBufferAllocator<'buffer> {
    bytes: &'buffer mut [u8],
    abstract_allocator: AbstractAllocator,
}

impl<'buffer> CpuBufferAllocator<'buffer> {
    pub fn suballocate_ref<T: Sized>(&mut self) -> Option<&'buffer mut std::mem::MaybeUninit<T>> {
        if let Some(starting_address) = self.abstract_allocator.suballocate_ref::<T>() {
            return unsafe {
                let bytes_pointer =
                    std::mem::transmute::<&u8, *mut u8>(&self.bytes[starting_address]);
                //let allocation_bytes_pointer = bytes_pointer.offset(starting_address);
                Some(std::mem::transmute::<
                    *mut u8,
                    &'buffer mut std::mem::MaybeUninit<T>,
                >(bytes_pointer))
            };
        }

        return None;
    }

    pub fn suballocate_slice<T: Sized>(
        &mut self,
        count: usize,
    ) -> Option<&'buffer mut [std::mem::MaybeUninit<T>]> {
        if let Some((starting_address, byte_size)) =
            self.abstract_allocator.suballocate_slice::<T>(count)
        {
            return unsafe {
                let bytes_pointer =
                    std::mem::transmute::<&u8, *mut u8>(&self.bytes[starting_address]);
                //let allocation_bytes_pointer = bytes_pointer.offset(starting_address);
                let allocation_base_pointer =
                    std::mem::transmute::<*mut u8, *mut std::mem::MaybeUninit<T>>(bytes_pointer);
                Some(std::slice::from_raw_parts_mut::<
                    'buffer,
                    std::mem::MaybeUninit<T>,
                >(allocation_base_pointer, count))
            };
        }

        return None;
    }
}

#[derive(Debug)]
// A slot holding a pointer to gpu-resident data of type T
pub struct CpuBufferRef<'buffer, T: Sized> {
    pub buffer: &'buffer std::mem::MaybeUninit<T>,
}

impl<'buffer, T: Sized> CpuBufferRef<'buffer, T> {
    pub fn new(buffer: &'buffer std::mem::MaybeUninit<T>) -> Self {
        Self { buffer }
    }
}

#[derive(Debug)]
pub struct JoinStack<'buffer> {
    bytes: &'buffer mut [u8],
    abstract_allocator: AbstractAllocator,
}

impl<'buffer> JoinStack<'buffer> {
    pub fn new(bytes: &'buffer mut [u8]) -> Self {
        let abstract_allocator = AbstractAllocator::new(bytes.len());
        Self {
            bytes,
            abstract_allocator,
        }
    }

    pub fn used_bytes(&self) -> &[u8] {
        &self.bytes[0..self.abstract_allocator.base_address]
    }

    pub fn unused_bytes(&self) -> &[u8] {
        // To do: Check for overflow?
        &self.bytes[self.abstract_allocator.base_address
            ..(self.abstract_allocator.base_address + self.abstract_allocator.size)]
    }

    pub fn push_raw(&mut self, bytes: &[u8]) -> Option<&'buffer mut [u8]> {
        if let Some(starting_offset) = self.abstract_allocator.suballocate(bytes.len(), 1) {
            return unsafe {
                let bytes_pointer =
                    std::mem::transmute::<&u8, *mut u8>(&self.bytes[starting_offset]);
                let destination_subslice =
                    std::slice::from_raw_parts_mut::<'buffer, u8>(bytes_pointer, bytes.len());
                destination_subslice.copy_from_slice(bytes);
                Some(destination_subslice)
            };
        }

        return None;
    }

    pub unsafe fn push_unsafe_unaligned<T: Sized>(
        &mut self,
        data: T,
    ) -> Result<&'buffer mut [u8], T> {
        let bytes = unsafe {
            let bytes_pointer = std::mem::transmute::<&T, *const u8>(&data);
            std::slice::from_raw_parts::<'buffer, u8>(bytes_pointer, std::mem::size_of::<T>())
        };

        if let Some(slice) = self.push_raw(bytes) {
            std::mem::forget(data);
            Result::Ok(slice)
        } else {
            Result::Err(data)
        }
    }

    pub fn peek_raw(&self, size: usize) -> Option<&[u8]> {
        if self.abstract_allocator.base_address < size {
            return None;
        }

        let new_base_address = self.abstract_allocator.base_address - size;

        return Some(&self.bytes[new_base_address..self.abstract_allocator.base_address]);
    }

    pub fn pop_raw(&mut self, size: usize) -> Option<&[u8]> {
        if self.abstract_allocator.base_address < size {
            return None;
        }

        let old_base_address = self.abstract_allocator.base_address;
        self.abstract_allocator.base_address -= size;
        // To do: Theoretically we can get overflow for size here if we ever allow initializing at a nonzero base address
        self.abstract_allocator.size += size;

        return Some(&self.bytes[self.abstract_allocator.base_address..old_base_address]);
    }

    pub unsafe fn pop_unsafe_unaligned<T: Sized>(&mut self) -> Option<T> {
        let size = std::mem::size_of::<T>();
        if let Some(source_bytes) = self.pop_raw(size) {
            let mut uninit_data: std::mem::MaybeUninit<T> = unsafe { std::mem::uninitialized() };
            let destination_bytes_pointer =
                std::mem::transmute::<&mut std::mem::MaybeUninit<T>, *mut u8>(&mut uninit_data);
            let destination_subslice =
                std::slice::from_raw_parts_mut::<'buffer, u8>(destination_bytes_pointer, size);
            destination_subslice.copy_from_slice(source_bytes);
            //let uninit_data_ref = std::mem::transmute::<& u8, & std::mem::MaybeUninit<T>>(& destination_subslice[0]);
            return Some(uninit_data.assume_init());
        }

        return None;
    }
}

#[derive(Debug)]
pub struct GpuBufferAllocator<'buffer> {
    buffer: &'buffer wgpu::Buffer,
    abstract_allocator: AbstractAllocator,
}

impl<'buffer> GpuBufferAllocator<'buffer> {
    pub fn new(buffer: &'buffer wgpu::Buffer, size: usize) -> Self {
        Self {
            buffer,
            abstract_allocator: AbstractAllocator::new(size),
        }
    }

    pub fn suballocate_ref<T: Sized>(&mut self) -> Option<GpuBufferRef<'buffer, T>> {
        if let Some(starting_address) = self.abstract_allocator.suballocate_ref::<T>() {
            return Some(GpuBufferRef::new(
                self.buffer,
                starting_address.try_into().unwrap(),
            ));
        }

        return None;
    }

    pub fn suballocate_slice<T: Sized>(
        &mut self,
        count: usize,
    ) -> Option<GpuBufferSlice<'buffer, T>> {
        if let Some((starting_address, byte_size)) =
            self.abstract_allocator.suballocate_slice::<T>(count)
        {
            return Some(GpuBufferSlice::new(
                self.buffer,
                starting_address.try_into().unwrap(),
                Some(std::num::NonZeroU64::new(byte_size.try_into().unwrap()).unwrap()),
            ));
        }

        return None;
    }

    // A very horribly implemented check
    pub fn test_suballocate_many(
        &self,
        layouts: &[TypeLayout],
        element_counts: &[Option<usize>],
    ) -> usize {
        let mut abstract_allocator = self.abstract_allocator.clone();

        let mut success_count = 0usize;

        for (i, layout) in layouts.iter().enumerate() {
            let can_allocate = if let Some(element_count) = element_counts[i] {
                abstract_allocator
                    .suballocate_erased_slice(layout, element_count)
                    .is_some()
            } else {
                abstract_allocator.suballocate_erased_ref(layout).is_some()
            };

            if !can_allocate {
                return success_count;
            }

            success_count += 1usize;
        }

        success_count
    }
}

#[derive(Debug)]
// A slot holding a pointer to gpu-resident data of type T
pub struct GpuBufferRef<'buffer, T: Sized> {
    phantom: std::marker::PhantomData<*const T>,
    pub buffer: &'buffer wgpu::Buffer,
    pub base_address: wgpu::BufferAddress,
    //offset : wgpu::DynamicOffset,
}

impl<'buffer, T: Sized> GpuBufferRef<'buffer, T> {
    pub fn new(buffer: &'buffer wgpu::Buffer, base_address: wgpu::BufferAddress) -> Self {
        Self {
            phantom: std::marker::PhantomData,
            buffer,
            base_address,
        }
    }

    pub fn as_binding_resource(&self) -> wgpu::BindingResource<'buffer> {
        let size_n: u64 = std::mem::size_of::<T>().try_into().unwrap();
        wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: self.buffer,
            offset: self.base_address,
            size: std::num::NonZeroU64::new(size_n),
        })
    }

    pub fn slice(&self) -> wgpu::BufferSlice<'buffer> {
        // Technically, this could overflow?
        let mut end_address = self.base_address;
        // Rust needs the type hint...
        let size_opt: Option<wgpu::BufferAddress> = std::mem::size_of::<T>().try_into().ok();
        end_address += size_opt.unwrap();
        self.buffer.slice(self.base_address..end_address)
    }
}

#[derive(Debug)]
// A slot holding a pointer to gpu-resident array of elements of type T
pub struct GpuBufferSlice<'buffer, T: Sized> {
    phantom: std::marker::PhantomData<*const T>,
    pub buffer: &'buffer wgpu::Buffer,
    pub base_address: wgpu::BufferAddress,
    //offset : wgpu::DynamicOffset,
    pub size_opt: Option<wgpu::BufferSize>,
}

impl<'buffer, T: Sized> GpuBufferSlice<'buffer, T> {
    pub fn new(
        buffer: &'buffer wgpu::Buffer,
        base_address: wgpu::BufferAddress,
        size_opt: Option<wgpu::BufferSize>,
    ) -> Self {
        Self {
            phantom: std::marker::PhantomData,
            buffer,
            base_address,
            size_opt,
        }
    }

    pub fn as_binding_resource(&self) -> wgpu::BindingResource<'buffer> {
        wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: self.buffer,
            offset: self.base_address,
            size: self.size_opt,
        })
    }

    pub fn slice(&self) -> wgpu::BufferSlice<'buffer> {
        if let Some(size) = self.size_opt {
            // Technically, this could overflow?
            let mut end_address = self.base_address;
            // Rust needs the type hint...
            let size_opt: Option<wgpu::BufferAddress> = size.try_into().ok();
            end_address += size_opt.unwrap();
            self.buffer.slice(self.base_address..end_address)
        } else {
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

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::pin::Pin;
use std::any::Any;
use crate::functional;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct BufferId(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CommandListId(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SubmissionId(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FuncletInstanceId(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SlotId(FuncletInstanceId, functional::NodeId);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LogicalTimestamp(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FenceId(usize);

impl LogicalTimestamp
{
	fn new() -> Self
	{
		Self(0)
	}

	fn step(&mut self)
	{
		self.0 += 1;
	}
}

// It is ok if this is horrible (performance-wise) because the goal is to formalize the semantics of the caiman scheduler in terms of the wgpu api and not to be useful for anything else

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum QueueState
{
	None,
	Encoded, // implies MappedLocalWrite if mapped
	Submitted,
	Ready, // implies MappedLocalRead if mapped
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Place
{
	Local,
	Cpu,
	Gpu,
}

enum MapMode
{
	ReadOnly,
	WriteOnly
}

enum Binding
{
	LocalVariable{ value_box : Box<dyn Any> },
	Buffer{ buffer_id : BufferId, start : usize, size : usize },
	ReadOnlyMappedBuffer{ buffer_id : BufferId, start : usize, size : usize, nasty_raw_pointer : *const u8 },
	WriteableMappedBuffer{ buffer_id : BufferId, start : usize, size : usize, nasty_raw_pointer : *mut u8 },
}

// For now, it's assumed that all buffers live on the gpu (this isn't at all true) and can be mapped local (this isn't true either)
struct Buffer
{
	queue_state : QueueState,
	map_count : usize,
	other_use_count : usize,
	allocated_ranges : BTreeMap<usize, usize>,
	wgpu_buffer : wgpu::Buffer
}

struct CommandEncoder
{
	buffer_ids : BTreeSet<BufferId>,
	wgpu_command_encoder_opt : Option<wgpu::CommandEncoder>,
	wgpu_command_buffers : Vec<wgpu::CommandBuffer>,
}

struct CommandList
{
	buffer_ids : BTreeSet<BufferId>,
	time_submitted : LogicalTimestamp
}

struct Fence
{
	time_inserted_opt : Option<LogicalTimestamp>,
	// This is disgusting in so many ways
	completion_future : Option<Pin<Box<dyn futures::Future<Output = ()> + Send>>>
}

struct FuncletInstance
{
	funclet_id : functional::FuncletId,
}

struct SchedulerState<'device, 'queue>
{
	device : & 'device mut wgpu::Device,
	queue : & 'queue mut wgpu::Queue,
	buffers : BTreeMap<BufferId, Buffer>,
	funclet_instances : Vec<FuncletInstance>,
	pending_command_lists : VecDeque<CommandList>,
	active_command_encoder_opt : Option<CommandEncoder>,
	fences : BTreeMap<FenceId, Fence>,
	slot_per_place_bindings : BTreeMap<SlotId, BTreeMap<Place, Binding>>,
	local_logical_timestamp : LogicalTimestamp,
	latest_gpu_synchronized_logical_timestamp : LogicalTimestamp,
}

// Essentially just a thin layer over wgpu
impl<'device, 'queue> SchedulerState<'device, 'queue>
{
	fn new(device : & 'device mut wgpu::Device, queue : & 'queue mut wgpu::Queue) -> Self
	{
		let mut buffers = BTreeMap::<BufferId, Buffer>::new();
		let mut fences = BTreeMap::<FenceId, Fence>::new();
		let funclet_instances = Vec::<FuncletInstance>::new();
		let slot_per_place_bindings = BTreeMap::<SlotId, BTreeMap<Place, Binding>>::new();
		let pending_command_lists = VecDeque::<CommandList>::new();
		Self{device, queue, buffers, funclet_instances, active_command_encoder_opt : None, pending_command_lists, /*submissions,*/ fences, slot_per_place_bindings, local_logical_timestamp : LogicalTimestamp::new(), latest_gpu_synchronized_logical_timestamp : LogicalTimestamp::new()}
	}

	fn queue_state_of_binding(&self, binding : & Binding) -> QueueState
	{
		match binding
		{
			Binding::LocalVariable {value_box} => panic!("Not implemented"),
			Binding::Buffer{buffer_id, start, size} => self.buffers[buffer_id].queue_state,
			Binding::ReadOnlyMappedBuffer{ buffer_id, start, size, nasty_raw_pointer } => self.buffers[buffer_id].queue_state,
			Binding::WriteableMappedBuffer{ buffer_id, start, size, nasty_raw_pointer } => self.buffers[buffer_id].queue_state,
		}
	}

	/*fn instance_funclet(&mut self, funclet_id : functional::FuncletId) -> FuncletInstanceId
	{
		self.funclet_instances.push(FuncletInstance{funclet_id});
		FuncletInstanceId(self.funclet_instances.len() - 1)
	}*/

	pub fn assert(&mut self, slot_ids : &[SlotId], place_and_queue_state_pairs : &[(Place, QueueState)])
	{
		for slot_id in slot_ids.iter()
		{
			let per_place_bindings = & self.slot_per_place_bindings[slot_id];
			for & (place, queue_state) in place_and_queue_state_pairs.iter()
			{
				assert_eq!(queue_state, self.queue_state_of_binding(& per_place_bindings[& place]));
			}
		}
	}

	pub fn bind_buffer(&mut self, slot_id : SlotId, place : Place, buffer_id : BufferId, offset : usize, size : usize)
	{
		// For now, it only makes sense to treat buffers as if they exist on the gpu
		assert_eq!(place, Place::Gpu);
		assert_eq!(self.buffers[& buffer_id].queue_state, QueueState::None);
		self.slot_per_place_bindings.get_mut(& slot_id).unwrap().insert(place, Binding::Buffer{buffer_id, start : offset, size});
	}

	pub fn unbind(&mut self, slot_ids : &[SlotId], place : Place)
	{
		for slot_id in slot_ids.iter()
		{
			let (buffer_id, is_a_mapping) = match self.slot_per_place_bindings[& slot_id][& place]
			{
				Binding::LocalVariable {ref value_box} => panic!("Not implemented"),
				Binding::ReadOnlyMappedBuffer{buffer_id, start, size, nasty_raw_pointer} => (buffer_id, true),
				Binding::WriteableMappedBuffer{buffer_id, start, size, nasty_raw_pointer} => (buffer_id, true),
				Binding::Buffer {buffer_id, ..} => (buffer_id, false),
			};
			if is_a_mapping
			{
				let buffer : &mut Buffer = self.buffers.get_mut(& buffer_id).unwrap();
				assert!(buffer.map_count > 0);
				buffer.map_count -= 1;
				if buffer.map_count == 0
				{
					buffer.wgpu_buffer.unmap();
				}
			}
			self.slot_per_place_bindings.get_mut(& slot_id).unwrap().remove(& place);
		}
	}

	// Self note: fences + queues implement an asynchronous reliable message passing system

	// Inserts a fence into the queue of fenced_place
	pub fn insert_fence(&mut self, fenced_place : Place, fence_id : FenceId)
	{
		// Only gpu -> local sync is implemented (because only local -> gpu submission is implemented)
		assert_eq!(fenced_place, Place::Gpu);
		let fence : &mut Fence = self.fences.get_mut(& fence_id).unwrap();
		assert!(fence.time_inserted_opt.is_none());
		fence.time_inserted_opt = Some(self.local_logical_timestamp);
		fence.completion_future = Some(Box::pin(self.queue.on_submitted_work_done()));
	}

	// Stalls the queue of synced_place until signaled through the given fence
	pub async fn sync_fence(&mut self, synced_place : Place, fence_id : FenceId)
	{
		self.local_logical_timestamp.step();

		// Only gpu -> local sync is implemented (because only local -> gpu submission is implemented)
		assert_eq!(synced_place, Place::Local);

		let fence : &mut Fence = self.fences.get_mut(& fence_id).unwrap();

		assert!(fence.time_inserted_opt.is_some());
		let time_inserted = fence.time_inserted_opt.unwrap();

		let mut completion_future = None;
		std::mem::swap(&mut completion_future, &mut fence.completion_future);

		if time_inserted >= self.latest_gpu_synchronized_logical_timestamp
		{
			completion_future.unwrap().await;
			//futures::executor::block_on(completion_future.unwrap());
			self.latest_gpu_synchronized_logical_timestamp = time_inserted;
		}

		while let Some(mut command_list) = self.pending_command_lists.pop_front()
		{
			if command_list.time_submitted > time_inserted
			{
				self.pending_command_lists.push_front(command_list);
				break;
			}

			for buffer_id in command_list.buffer_ids.iter()
			{
				let buffer : &mut Buffer = self.buffers.get_mut(& buffer_id).unwrap();

				assert!(buffer.other_use_count > 0);
				buffer.other_use_count -= 1;

				buffer.queue_state = match buffer.queue_state
				{
					QueueState::None => panic!("Buffer was not properly encoded or submitted"),
					QueueState::Encoded => panic!("Buffer was not properly submitted"),
					QueueState::Submitted => QueueState::Ready,
					QueueState::Ready => QueueState::Ready,
				};
			}
		}
	}

	fn do_local_constant_unsigned_integer(&mut self, slot_id : SlotId, value : u64)
	{
		let value_box : Box<dyn Any> = Box::new(value);
		self.slot_per_place_bindings.get_mut(& slot_id).unwrap().insert(Place::Local, Binding::LocalVariable{value_box});
	}

	fn do_local_constant_integer(&mut self, slot_id : SlotId, value : i64)
	{
		let value_box : Box<dyn Any> = Box::new(value);
		self.slot_per_place_bindings.get_mut(& slot_id).unwrap().insert(Place::Local, Binding::LocalVariable{value_box});
	}

	fn encode_gpu_call_gpu_external(&mut self, slot_id : SlotId, dimensions : &[SlotId], arguments : &[SlotId], outputs : &[SlotId])
	{
		
	}

	/*pub fn do_local(&mut self, slot_ids : &[SlotId])
	{
	}*/

	/*pub fn encode_gpu(&mut self, command_list_id : CommandListId, slot_ids : &[SlotId]);*/

	fn begin_command_encoding(&mut self)
	{
		if self.active_command_encoder_opt.is_none()
		{
			let active_command_encoder = CommandEncoder{buffer_ids : BTreeSet::new(), wgpu_command_encoder_opt : None, wgpu_command_buffers : vec![]};
			self.active_command_encoder_opt = Some(active_command_encoder);
		}

		let active_command_encoder = self.active_command_encoder_opt.as_mut().unwrap();
		if active_command_encoder.wgpu_command_encoder_opt.is_none()
		{
			active_command_encoder.wgpu_command_encoder_opt = Some(self.device.create_command_encoder(& wgpu::CommandEncoderDescriptor{label : None}));
		}
	}

	fn end_command_encoding(&mut self)
	{
		if let Some(active_command_encoder) = self.active_command_encoder_opt.as_mut()
		{
			let mut wgpu_command_encoder_opt = None;
			std::mem::swap(&mut wgpu_command_encoder_opt, &mut active_command_encoder.wgpu_command_encoder_opt);
			if let Some(wgpu_command_encoder) = wgpu_command_encoder_opt
			{
				active_command_encoder.wgpu_command_buffers.push(wgpu_command_encoder.finish());
			}
		}
	}

	pub fn submit_gpu(&mut self)
	{
		self.local_logical_timestamp.step();

		self.end_command_encoding();

		let mut active_command_encoder_opt = None;
		std::mem::swap(&mut active_command_encoder_opt, &mut self.active_command_encoder_opt);
		if let Some(mut active_command_encoder) = active_command_encoder_opt
		{
			let command_list = CommandList{buffer_ids : active_command_encoder.buffer_ids, time_submitted : self.local_logical_timestamp};
			self.queue.submit(active_command_encoder.wgpu_command_buffers);

			for buffer_id in command_list.buffer_ids.iter()
			{
				let buffer : &mut Buffer = self.buffers.get_mut(& buffer_id).unwrap();
				buffer.queue_state = match buffer.queue_state
				{
					QueueState::None => panic!("Buffer was not properly encoded"),
					QueueState::Encoded => QueueState::Submitted,
					QueueState::Submitted => QueueState::Submitted,
					QueueState::Ready => QueueState::Ready,
				};
			}

			self.pending_command_lists.push_back(command_list);
		}
	}

	pub async fn map_local(&mut self, slot_id : SlotId, map_mode : MapMode)
	{
		assert!(! self.slot_per_place_bindings[& slot_id].contains_key(& Place::Local));
		let (buffer_id, start, size) = match self.slot_per_place_bindings[& slot_id][& Place::Gpu]
		{
			Binding::Buffer{buffer_id, start, size} => (buffer_id, start, size),
			_ => panic!("Incorrect binding for slot")
		};
		
		let buffer : &mut Buffer = self.buffers.get_mut(& buffer_id).unwrap();

		let wgpu_map_mode = match map_mode
		{
			MapMode::ReadOnly =>
			{
				assert_eq!(buffer.queue_state, QueueState::Ready);
				wgpu::MapMode::Read
			}
			MapMode::WriteOnly =>
			{
				assert_eq!(buffer.other_use_count, 0);
				assert_eq!(buffer.queue_state, QueueState::None);
				assert_eq!(buffer.map_count, 0);
				wgpu::MapMode::Write
			}
		};

		// (Mostly) Dynamic part
		let slice = buffer.wgpu_buffer.slice((start as u64) .. (start + size) as u64);

		// The correctness of this depends on all gpu users of the buffer having completed beforehand
		// This should be captured by the above checks
		// This goes wrong by deadlocking
		// This is all very silly because wgpu is checking to maintain properties that we should have already guaranteed 
		//self.device.poll(wgpu::Maintain::Poll);

		// The host rust code is expected to invoke polling interleaved with this
		slice.map_async(wgpu_map_mode).await;
		//futures::executor::block_on(slice.map_async(wgpu_map_mode));
		let binding = match map_mode
		{
			MapMode::ReadOnly =>
			{
				buffer.map_count += 1;
				let nasty_raw_pointer = slice.get_mapped_range().as_ptr();
				Binding::ReadOnlyMappedBuffer{buffer_id, start, size, nasty_raw_pointer}
			}
			MapMode::WriteOnly =>
			{
				buffer.map_count += 1;
				let nasty_raw_pointer = slice.get_mapped_range_mut().as_mut_ptr();
				Binding::WriteableMappedBuffer{buffer_id, start, size, nasty_raw_pointer}
			}
		};

		self.slot_per_place_bindings.get_mut(& slot_id).unwrap().insert(Place::Local, binding);
	}
}

fn main()
{
	let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
	let adapter = futures::executor::block_on(instance.request_adapter(& wgpu::RequestAdapterOptions { power_preference : wgpu::PowerPreference::default(), compatible_surface : None, force_fallback_adapter : false })).unwrap();
	let (mut device, mut queue) = futures::executor::block_on(adapter.request_device(& std::default::Default::default(), None)).unwrap();
	let scheduler_state = SchedulerState::new(&mut device, &mut queue);
	println!("Hello, world!");
}

use std::collections::{BTreeMap, BTreeSet};
use std::pin::Pin;
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
	Buffer{ buffer_id : BufferId, start : usize, size : usize },
	ReadOnlyMappedBuffer{ buffer_id : BufferId, start : usize, size : usize, nasty_raw_pointer : *const u8 },
	WriteableMappedBuffer{ buffer_id : BufferId, start : usize, size : usize, nasty_raw_pointer : *mut u8 }
}

// For now, it's assumed that all buffers live on the gpu (this isn't at all true) and can be mapped local (this isn't true either)
struct Buffer
{
	queue_state : QueueState,
	map_count : usize,
	allocated_ranges : BTreeMap<usize, usize>,
	wgpu_buffer : wgpu::Buffer
}

struct CommandList
{
	queue_state : QueueState,
	buffer_ids : BTreeSet<BufferId>,
	wgpu_command_encoder_opt : Option<wgpu::CommandEncoder>,
	// This is disgusting in so many ways
	completion_future : Option<Pin<Box<dyn futures::Future<Output = ()> + Send>>>
}

/*struct Submission
{
	queue_state : QueueState,
}

struct Placement
{
	
}

struct SlotPlacement
{
	
}*/

struct SchedulerState<'device, 'queue>
{
	device : & 'device mut wgpu::Device,
	queue : & 'queue mut wgpu::Queue,
	buffers : BTreeMap<BufferId, Buffer>,
	command_lists : BTreeMap<CommandListId, CommandList>,
	//submissions : BTreeMap<SubmissionId, Submission>,
	slot_per_place_bindings : BTreeMap<SlotId, BTreeMap<Place, Binding>>,
}

// Essentially just a thin layer over wgpu
impl<'device, 'queue> SchedulerState<'device, 'queue>
{
	fn new(device : & 'device mut wgpu::Device, queue : & 'queue mut wgpu::Queue) -> Self
	{
		let mut buffers = BTreeMap::<BufferId, Buffer>::new();
		let mut command_lists = BTreeMap::<CommandListId, CommandList>::new();
		//let mut submissions = BTreeMap::<SubmissionId, Submission>::new();
		let slot_per_place_bindings = BTreeMap::<SlotId, BTreeMap<Place, Binding>>::new();
		Self{device, queue, buffers, command_lists, /*submissions,*/ slot_per_place_bindings}
	}

	fn queue_state_of_binding(&self, binding : & Binding) -> QueueState
	{
		match binding
		{
			Binding::Buffer{buffer_id, start, size} => self.buffers[buffer_id].queue_state,
			Binding::ReadOnlyMappedBuffer{ buffer_id, start, size, nasty_raw_pointer } => self.buffers[buffer_id].queue_state,
			Binding::WriteableMappedBuffer{ buffer_id, start, size, nasty_raw_pointer } => self.buffers[buffer_id].queue_state,
		}
	}

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
			self.slot_per_place_bindings.get_mut(& slot_id).unwrap().remove(& place);
		}
	}

	/*pub fn do_local(&mut self, slot_ids : &[SlotId])
	{

	}*/

	/*pub fn encode_gpu(&mut self, command_list_id : CommandListId, slot_ids : &[SlotId]);*/

	pub fn submit_gpu(&mut self, command_list_id : CommandListId)
	{
		let command_list : &mut CommandList = self.command_lists.get_mut(& command_list_id).unwrap();
		assert_eq!(command_list.queue_state, QueueState::Encoded);
		let mut command_encoder = None;
		std::mem::swap(&mut command_encoder, &mut command_list.wgpu_command_encoder_opt);

		// Dynamic part
		self.queue.submit([command_encoder.unwrap().finish()]);
		command_list.completion_future = Some(Box::pin(self.queue.on_submitted_work_done()));

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
	}

	pub fn sync_gpu(&mut self, command_list_ids : &[CommandListId])
	{
		for command_list_id in command_list_ids.iter()
		{
			let command_list : &mut CommandList = self.command_lists.get_mut(& command_list_id).unwrap();
			assert_eq!(command_list.queue_state, QueueState::Submitted);
			let mut completion_future = None;
			std::mem::swap(&mut completion_future, &mut command_list.completion_future);
			
			// Dynamic part
			futures::executor::block_on(completion_future.unwrap());

			command_list.queue_state = QueueState::Ready;

			for buffer_id in command_list.buffer_ids.iter()
			{
				let buffer : &mut Buffer = self.buffers.get_mut(& buffer_id).unwrap();
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

	pub fn map_local(&mut self, slot_id : SlotId, map_mode : MapMode)
	{
		assert!(! self.slot_per_place_bindings[& slot_id].contains_key(& Place::Local));
		let (buffer_id, start, size) = match self.slot_per_place_bindings[& slot_id][& Place::Gpu]
		{
			Binding::Buffer{buffer_id, start, size} => (buffer_id, start, size),
			_ => panic!("Incorrect binding for slot")
		};
		
		let buffer : &mut Buffer = self.buffers.get_mut(& buffer_id).unwrap();
		assert_eq!(buffer.queue_state, QueueState::None);

		let wgpu_map_mode = match map_mode
		{
			MapMode::ReadOnly => wgpu::MapMode::Read,
			MapMode::WriteOnly => wgpu::MapMode::Write,
		};

		//Dynamic part
		let slice = buffer.wgpu_buffer.slice((start as u64) .. (start + size) as u64);
		futures::executor::block_on(slice.map_async(wgpu_map_mode));
		let binding = match map_mode
		{
			MapMode::ReadOnly =>
			{
				let nasty_raw_pointer = slice.get_mapped_range().as_ptr();
				Binding::ReadOnlyMappedBuffer{buffer_id, start, size, nasty_raw_pointer}
			}
			MapMode::WriteOnly =>
			{
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

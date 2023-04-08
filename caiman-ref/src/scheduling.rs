use crate::functional;
use std::any::Any;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::pin::Pin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct BufferId(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FuncletInstanceId(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SlotId(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ValueId(FuncletInstanceId, functional::NodeId);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LogicalTimestamp(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FenceId(usize);

impl LogicalTimestamp {
    fn new() -> Self {
        Self(0)
    }

    fn step(&mut self) {
        self.0 += 1;
    }
}

// It is ok if this is horrible (performance-wise) because the goal is to formalize the semantics of the caiman scheduler in terms of the wgpu api and not to be useful for anything else

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum QueueState {
    None,
    Encoded, // implies MappedLocalWrite if mapped
    Submitted,
    Ready, // implies MappedLocalRead if mapped
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Place {
    Local,
    Cpu,
    Gpu,
}

enum MapMode {
    ReadOnly,
    WriteOnly,
}

enum Binding {
    Variable {
        value_box: Box<dyn Any>,
    },
    Buffer {
        buffer_id: BufferId,
        start: usize,
        size: usize,
    },
    ReadOnlyMappedBuffer {
        buffer_id: BufferId,
        start: usize,
        size: usize,
        nasty_raw_pointer: *const u8,
    },
    WriteableMappedBuffer {
        buffer_id: BufferId,
        start: usize,
        size: usize,
        nasty_raw_pointer: *mut u8,
    },
}

struct Slot {
    // Part of the type describing the value
    value_id_opt: Option<ValueId>,
    // Part of the type describing when the value will be available
    queue_place: Place,
    queue_state: QueueState,
    time_observed: LogicalTimestamp,
    // Part of the type describing the resource that will hold the value
    binding: Binding,
}

// For now, it's assumed that all buffers live on the gpu (this isn't at all true) and can be mapped local (this isn't true either)
struct Buffer {
    map_count: usize,
    other_use_count: usize,
    wgpu_buffer: wgpu::Buffer,
}

struct CommandEncoder {
    wgpu_command_encoder_opt: Option<wgpu::CommandEncoder>,
    wgpu_command_buffers: Vec<wgpu::CommandBuffer>,
}

struct Fence {
    fenced_place: Place,
    time_inserted_opt: Option<LogicalTimestamp>,
    // This is disgusting in so many ways
    completion_future: Option<Pin<Box<dyn futures::Future<Output = ()> + Send>>>,
}

struct FuncletInstance {
    funclet_id: functional::FuncletId,
}

/*trait GpuComputeFunction
{

}*/

/*struct PlaceState
{
    logical_timestamp : LogicalTimestamp
}*/

struct SchedulerState<'device, 'queue> {
    device: &'device mut wgpu::Device,
    queue: &'queue mut wgpu::Queue,
    buffers: BTreeMap<BufferId, Buffer>,
    funclet_instances: Vec<FuncletInstance>,
    active_command_encoder_opt: Option<CommandEncoder>,
    fences: BTreeMap<FenceId, Fence>,
    slots: Vec<Option<Slot>>,
    //place_states : BTreeMap<Place, PlaceState>,
    local_logical_timestamp: LogicalTimestamp,
    latest_gpu_synchronized_logical_timestamp: LogicalTimestamp,
}

// Essentially just a thin layer over wgpu
impl<'device, 'queue> SchedulerState<'device, 'queue> {
    fn new(device: &'device mut wgpu::Device, queue: &'queue mut wgpu::Queue) -> Self {
        let mut buffers = BTreeMap::<BufferId, Buffer>::new();
        let mut fences = BTreeMap::<FenceId, Fence>::new();
        let funclet_instances = Vec::<FuncletInstance>::new();
        let slots = Vec::<Option<Slot>>::new();
        Self {
            device,
            queue,
            buffers,
            funclet_instances,
            active_command_encoder_opt: None,
            /*submissions,*/ fences,
            slots,
            local_logical_timestamp: LogicalTimestamp::new(),
            latest_gpu_synchronized_logical_timestamp: LogicalTimestamp::new(),
        }
    }

    fn create_slot(&mut self, slot: Slot) -> SlotId {
        let slot_id = SlotId(self.slots.len());
        self.slots.push(Some(slot));
        slot_id
    }

    fn destroy_slot(&mut self, slot_id: SlotId) {
        self.slots[slot_id.0] = None;
    }

    fn has_slot(&self, slot_id: SlotId) -> bool {
        self.slots[slot_id.0].is_some()
    }

    fn get_slot(&self, slot_id: SlotId) -> &Slot {
        self.slots[slot_id.0].as_ref().unwrap()
    }

    fn get_slot_mut(&mut self, slot_id: SlotId) -> &mut Slot {
        self.slots[slot_id.0].as_mut().unwrap()
    }

    /*fn instance_funclet(&mut self, funclet_id : functional::FuncletId) -> FuncletInstanceId
    {
        self.funclet_instances.push(FuncletInstance{funclet_id});
        FuncletInstanceId(self.funclet_instances.len() - 1)
    }*/

    /*pub fn assert(&mut self, slot_ids : &[SlotId], place_and_queue_state_pairs : &[(Place, QueueState)])
    {
        for & slot_id in slot_ids.iter()
        {
            for & (place, queue_state) in place_and_queue_state_pairs.iter()
            {
                assert_eq!(place, self.get_slot(slot_id).place);
                assert_eq!(queue_state, self.get_slot(slot_id).queue_state);
            }
        }
    }*/

    pub fn bind_buffer(&mut self, buffer_id: BufferId, offset: usize, size: usize) -> SlotId {
        let time_observed = self.local_logical_timestamp;
        let binding = Binding::Buffer {
            buffer_id,
            start: offset,
            size,
        };
        let slot = Slot {
            value_id_opt: None,
            queue_place: Place::Gpu,
            queue_state: QueueState::None,
            time_observed,
            binding,
        };
        self.create_slot(slot)
    }

    pub fn discard(&mut self, slot_ids: &[SlotId]) {
        for &slot_id in slot_ids.iter() {
            let mapped_buffer_id_opt = match self.get_slot(slot_id).binding {
                Binding::Variable { ref value_box } => None,
                Binding::ReadOnlyMappedBuffer {
                    buffer_id,
                    start,
                    size,
                    nasty_raw_pointer,
                } => Some(buffer_id),
                Binding::WriteableMappedBuffer {
                    buffer_id,
                    start,
                    size,
                    nasty_raw_pointer,
                } => Some(buffer_id),
                Binding::Buffer { buffer_id, .. } => None,
            };

            if let Some(buffer_id) = mapped_buffer_id_opt {
                let buffer: &mut Buffer = self.buffers.get_mut(&buffer_id).unwrap();
                assert!(buffer.map_count > 0);
                buffer.map_count -= 1;
                if buffer.map_count == 0 {
                    buffer.wgpu_buffer.unmap();
                }
            }

            self.destroy_slot(slot_id);
        }
    }

    // Self note: fences + queues implement an asynchronous reliable message passing system

    // Inserts a fence into the queue of fenced_place
    pub fn insert_fence(&mut self, fenced_place: Place, fence_id: FenceId) {
        // Only gpu -> local sync is implemented (because only local -> gpu submission is implemented)
        assert_eq!(fenced_place, Place::Gpu);
        let fence: &mut Fence = self.fences.get_mut(&fence_id).unwrap();
        assert!(fence.time_inserted_opt.is_none());
        fence.fenced_place = fenced_place;
        fence.time_inserted_opt = Some(self.local_logical_timestamp);
        fence.completion_future = Some(Box::pin(self.queue.on_submitted_work_done()));
    }

    fn transition_resource_states(
        &mut self,
        place: Place,
        from_state: QueueState,
        to_state: QueueState,
    ) {
        let time_observed = match place {
            Place::Local => self.local_logical_timestamp,
            Place::Gpu => self.latest_gpu_synchronized_logical_timestamp,
            Place::Cpu => panic!("Not yet implemented"),
        };

        for (slot_id, slot_opt) in self.slots.iter_mut().enumerate() {
            if let Some(slot) = slot_opt {
                if slot.queue_place == place && time_observed >= slot.time_observed {
                    if slot.queue_state == from_state {
                        slot.time_observed = time_observed;
                        slot.queue_state = to_state;
                    }
                }
            }
        }
    }

    // Stalls the queue of synced_place until signaled through the given fence
    pub async fn sync_fence(&mut self, synced_place: Place, fence_id: FenceId) {
        self.local_logical_timestamp.step();

        // Only gpu -> local sync is implemented (because only local -> gpu submission is implemented)
        assert_eq!(synced_place, Place::Local);

        let fence: &mut Fence = self.fences.get_mut(&fence_id).unwrap();

        assert!(fence.time_inserted_opt.is_some());
        let time_inserted = fence.time_inserted_opt.unwrap();

        let mut completion_future = None;
        std::mem::swap(&mut completion_future, &mut fence.completion_future);

        if time_inserted >= self.latest_gpu_synchronized_logical_timestamp {
            completion_future.unwrap().await;
            //futures::executor::block_on(completion_future.unwrap());
            self.latest_gpu_synchronized_logical_timestamp = time_inserted;
        }

        self.transition_resource_states(Place::Gpu, QueueState::Submitted, QueueState::Ready);

        /*for (slot_id, slot_opt) in self.slots.iter_mut().enumerate()
        {
            if let Some(slot) = slot_opt
            {
                if slot.queue_place == Place::Gpu && time_inserted >= slot.time_observed
                {
                    if slot.queue_state == QueueState::Submitted
                    {
                        slot.time_observed = self.latest_gpu_synchronized_logical_timestamp;
                        slot.queue_state = QueueState::Ready;
                    }
                }
            }
        }*/
    }

    /*fn do_local_constant<T : 'static + Copy>(&mut self, slot_id : SlotId, value_ref : &T)
    {
        match self.slot_per_place_bindings[& slot_id].get(& Place::Local).as_ref()
        {
            None =>
            {
                assert!(! self.slot_per_place_bindings[& slot_id].contains_key(& Place::Gpu));
                assert!(! self.has_slot(slot_id));
                let value_box : Box<dyn Any> = Box::new(* value_ref);
                let time_observed = self.logical_timestamp;
                self.initialize_slot(slot_id, SlotId{value_id : , queue_place : Place::Local, queue_state : QueueState::Ready, time_observed, binding : Binding::Variable{value_box}});
                self.slot_per_place_bindings.get_mut(& slot_id).unwrap().insert(Place::Local, Binding::Variable{value_box});
            }
            Some(& Binding::WriteableMappedBuffer{buffer_id, start : _, size, nasty_raw_pointer}) =>
            {
                assert_eq!(* size, std::mem::size_of::<T>());
                let source_bytes : &[u8] = unsafe { std::slice::from_raw_parts( std::mem::transmute::<*const T, *const u8>(value_ref), std::mem::size_of::<T>()) };
                let destination_bytes : &mut [u8] = unsafe { std::slice::from_raw_parts_mut::<u8>( *nasty_raw_pointer, std::mem::size_of::<T>()) };
                destination_bytes.clone_from_slice(source_bytes);
            }
            _ => panic!("Incorrect binding for slot")
        };
    }

    fn encode_gpu_from_local<T : 'static + Copy>(&mut self, slot_id : SlotId)
    {
        let (buffer_id, start, size) = match self.slot_per_place_bindings[& slot_id][& Place::Gpu]
        {
            Binding::Buffer{buffer_id, start, size} => (buffer_id, start, size),
            _ => panic!("Incorrect binding for slot")
        };

        let value_ref : &T = match self.slot_per_place_bindings[& slot_id][& Place::Local]
        {
            Binding::Variable{ref value_box} => value_box.downcast_ref::<T>().unwrap(),
            _ => panic!("Incorrect binding for slot")
        };

        assert_eq!(size, std::mem::size_of::<T>());
        let bytes : &[u8] = unsafe { std::slice::from_raw_parts( std::mem::transmute::<*const T, *const u8>(value_ref), std::mem::size_of::<T>()) };
        self.queue.write_buffer(& self.buffers[& buffer_id].wgpu_buffer, start.try_into().unwrap(), bytes);
    }*/

    /*fn encode_gpu_call_gpu_compute_function<F : GpuComputeFunction>(&mut self, slot_id : SlotId, function : &F, dimensions : &[SlotId], arguments : &[SlotId], outputs : &[SlotId])
    {

    }*/

    /*pub fn do_local(&mut self, slot_ids : &[SlotId])
    {
    }*/

    /*pub fn encode_gpu(&mut self, command_list_id : CommandListId, slot_ids : &[SlotId]);*/

    fn begin_command_encoding(&mut self) {
        if self.active_command_encoder_opt.is_none() {
            let active_command_encoder = CommandEncoder {
                wgpu_command_encoder_opt: None,
                wgpu_command_buffers: vec![],
            };
            self.active_command_encoder_opt = Some(active_command_encoder);
        }

        let active_command_encoder = self.active_command_encoder_opt.as_mut().unwrap();
        if active_command_encoder.wgpu_command_encoder_opt.is_none() {
            active_command_encoder.wgpu_command_encoder_opt = Some(
                self.device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None }),
            );
        }
    }

    fn end_command_encoding(&mut self) {
        if let Some(active_command_encoder) = self.active_command_encoder_opt.as_mut() {
            let mut wgpu_command_encoder_opt = None;
            std::mem::swap(
                &mut wgpu_command_encoder_opt,
                &mut active_command_encoder.wgpu_command_encoder_opt,
            );
            if let Some(wgpu_command_encoder) = wgpu_command_encoder_opt {
                active_command_encoder
                    .wgpu_command_buffers
                    .push(wgpu_command_encoder.finish());
            }
        }
    }

    pub fn submit(&mut self, target_place: Place) {
        // Only supported for GPU currently
        assert_eq!(target_place, Place::Gpu);

        self.local_logical_timestamp.step();

        self.end_command_encoding();

        let mut active_command_encoder_opt = None;
        std::mem::swap(
            &mut active_command_encoder_opt,
            &mut self.active_command_encoder_opt,
        );
        if let Some(mut active_command_encoder) = active_command_encoder_opt {
            self.queue
                .submit(active_command_encoder.wgpu_command_buffers);

            /*for (slot_id, slot_opt) in self.slots.iter_mut().enumerate()
            {
                if let Some(slot) = slot_opt
                {
                    if slot.queue_place == target_place && self.local_logical_timestamp >= slot.time_observed
                    {
                        if slot.queue_state == QueueState::Encoded
                        {
                            slot.time_observed = self.local_logical_timestamp;
                            slot.queue_state = QueueState::Submitted;
                        }
                    }
                }
            }*/
        }

        self.transition_resource_states(target_place, QueueState::Encoded, QueueState::Submitted);
    }

    /*pub async fn map_local(&mut self, to_slot_id : SlotId, from_slot_id : SlotId, map_mode : MapMode)
    {
        assert!(! self.has_slot(to_slot_id));
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

    pub fn convert_locally_mapped_to_variable<T : 'static + Copy>(&mut self, to_slot_id : SlotId, from_slot_id : SlotId)
    {
        let (size, nasty_raw_pointer) = match self.slot_per_place_bindings[& slot_id][& Place::Local]
        {
            Binding::ReadOnlyMappedBuffer{buffer_id : _, start : _, size, nasty_raw_pointer} => (size, nasty_raw_pointer),
            //Binding::WriteableMappedBuffer{buffer_id : _, start : _, size, nasty_raw_pointer} => (size, nasty_raw_pointer as (*const u8)),
            _ => panic!("Incorrect binding for slot")
        };

        assert_eq!(size, std::mem::size_of::<T>());
        let value : T = unsafe { * std::mem::transmute::<*const u8, *const T>(nasty_raw_pointer) };

        let value_box : Box<dyn Any> = Box::new(value);
        self.discard(&[slot_id], Place::Local);
        self.initialize_slot(slot_id, Place::Local, Slot{queue_state : QueueState::Ready, Binding::Variable{value_box}})
        self.slot_per_place_bindings.get_mut(& slot_id).unwrap().insert(Place::Local, Binding::Variable{value_box});
    }*/
}

fn main() {
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let adapter =
        futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .unwrap();
    let (mut device, mut queue) = futures::executor::block_on(
        adapter.request_device(&std::default::Default::default(), None),
    )
    .unwrap();
    let scheduler_state = SchedulerState::new(&mut device, &mut queue);
    println!("Hello, world!");
}

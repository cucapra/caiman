use crate::ir;
use crate::arena::Arena;
use std::default::Default;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use std::collections::BinaryHeap;
use std::cmp::Reverse;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SlotId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ValueId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubmissionId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LogicalTimestamp(usize);

impl LogicalTimestamp
{
	pub fn new() -> Self
	{
		Self(0)
	}

	pub fn step(&mut self)
	{
		self.0 += 1;
	}
}

impl Default for LogicalTimestamp
{
	fn default() -> Self
	{
		Self::new()
	}
}

#[derive(Debug)]
enum StateBinding
{
	TemporaryHack,
}

#[derive(Debug)]
struct Value
{
	type_id_opt : Option<ir::TypeId>,
	dependencies : BTreeSet<ValueId>,
}

#[derive(Debug)]
struct Slot
{
	type_id : ir::TypeId,
	value_opt : Option<ValueId>,
	timestamp : LogicalTimestamp,
	queue_place : ir::Place,
	queue_stage : ir::ResourceQueueStage,
	state_binding : StateBinding,
}

#[derive(Debug)]
struct Submission
{
	queue_place : ir::Place,
	timestamp : LogicalTimestamp,
}

// Records the most recent state of a place as known to the local coordinator
#[derive(Debug, Default)]
struct PlaceState
{
	timestamp : LogicalTimestamp, // most recent timestamp of the coordinator that the coordinator knows the place knows
	pending_submissions : BTreeMap<LogicalTimestamp, SubmissionId>,
}

#[derive(Debug, Default)]
pub struct SchedulingState
{
	place_states : HashMap<ir::Place, PlaceState>, // as known to the coordinator
	slots : Arena<Slot>,
	values : Arena<Value>,
	submissions : Arena<Submission>
}

#[derive(Debug)]
pub enum SchedulingEvent
{
	SyncSubmission{ submission_id : SubmissionId }
}

impl SchedulingState
{
	pub fn new() -> Self
	{
		let mut place_states = HashMap::new();
		place_states.insert(ir::Place::Local, PlaceState{ .. Default::default() });
		place_states.insert(ir::Place::Gpu, PlaceState{ .. Default::default() });
		Self{ place_states, .. Default::default() }
	}

	pub fn insert_hacked_slot(&mut self, type_id : ir::TypeId, queue_place : ir::Place) -> SlotId
	{
		let timestamp = self.get_local_time();
		let slot = Slot{type_id, value_opt : None, timestamp, queue_place, queue_stage : ir::ResourceQueueStage::None, state_binding : StateBinding::TemporaryHack};
		SlotId(self.slots.create(slot))
	}

	pub fn bind_slot_value(&mut self, slot_id : SlotId, value_id : ValueId)
	{
		let slot = &mut self.slots[& slot_id.0];
		let value = & self.values[& value_id.0];
		assert!(value.type_id_opt.is_some());
		assert_eq!(slot.type_id, value.type_id_opt.unwrap());
		assert!(slot.value_opt.is_none());
		slot.value_opt = Some(value_id);
	}

	pub fn insert_value(&mut self, type_id_opt : Option<ir::TypeId>, mandatory_dependencies : &[ValueId]) -> ValueId
	{
		use std::iter::FromIterator;
		let dependencies = BTreeSet::<ValueId>::from_iter(mandatory_dependencies.iter().map(|x| * x));
		ValueId(self.values.create(Value{type_id_opt, dependencies}))
	}

	pub fn insert_submission<Listener>(&mut self, queue_place : ir::Place, listener : &mut Listener) -> SubmissionId
		where Listener : FnMut(&Self, &SchedulingEvent) -> ()
	{
		let timestamp = self.get_local_time();

		for (slot_index, slot) in self.slots.iter_mut()
		{
			if slot.queue_place != queue_place
			{
				continue;
			}
			
			match slot.queue_stage
			{
				ir::ResourceQueueStage::Encoded =>
				{
					slot.queue_stage = ir::ResourceQueueStage::Submitted;
					slot.timestamp = timestamp;
				}
				_ => ()
			}
		}

		SubmissionId(self.submissions.create(Submission{queue_place, timestamp}))
	}

	/*fn get_slot_queue_stage(&self, slot_id : SlotId)
	{

	}*/

	fn get_local_time(&self) -> LogicalTimestamp
	{
		self.place_states[& ir::Place::Local].timestamp
	}

	pub fn advance_local_time(&mut self) -> LogicalTimestamp
	{
		self.place_states.get_mut(& ir::Place::Local).unwrap().timestamp.step();
		let local_timestamp = self.place_states.get(& ir::Place::Local).unwrap().timestamp;
		local_timestamp
	}

	pub fn advance_known_place_time<Listener>(&mut self, place : ir::Place, known_timestamp : LogicalTimestamp, listener : &mut Listener) -> Option<LogicalTimestamp>
		where Listener : FnMut(& Self, &SchedulingEvent) -> ()
	{
		assert!(place != ir::Place::Local);
		let local_timestamp = self.place_states[& ir::Place::Local].timestamp;
		// The local coordinator is always the latest time because all events are caused by the coordinator
		assert!(known_timestamp <= local_timestamp);

		let place_state : &mut PlaceState = self.place_states.get_mut(& place).unwrap();

		// Return if we already know of this or a later time

		if place_state.timestamp >= known_timestamp
		{
			return Some(place_state.timestamp);
		}
		
		place_state.timestamp = known_timestamp;

		// Update submissions for this place
		let mut last_submission_id_opt : Option<SubmissionId> = None;
		let mut expired_timestamps = Vec::<LogicalTimestamp>::new();
		for (& timestamp, & submission_id) in place_state.pending_submissions.iter()
		{
			if timestamp <= known_timestamp
			{
				expired_timestamps.push(timestamp);
				//self.sync_submission(submission_id);
				// Relies on iteration order of a BTreeMap
				last_submission_id_opt = Some(submission_id);
			}
			else
			{
				// Also relies on iteration order
				break
			}
		}

		for & timestamp in expired_timestamps.iter()
		{
			place_state.pending_submissions.remove(& timestamp);
		}

		if let Some(submission_id) = last_submission_id_opt
		{
			// Need to do something here
			//self.code_generator.sync_submission(submission_id);
			listener(& self, & SchedulingEvent::SyncSubmission{ submission_id });
		}

		// Transition resource stages

		for (slot_index, slot) in self.slots.iter_mut()
		{
			if slot.queue_place != place || slot.timestamp > known_timestamp
			{
				continue;
			}
			
			match slot.queue_stage
			{
				ir::ResourceQueueStage::Submitted =>
				{
					slot.queue_stage = ir::ResourceQueueStage::Ready;
					slot.timestamp = local_timestamp;
				}
				_ => ()
			}
		}

		None
	}
}
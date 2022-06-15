use crate::ir;
use crate::shadergen;
use crate::arena::Arena;
use std::default::Default;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
use crate::rust_wgpu_backend::code_generator::CodeGenerator;
use crate::rust_wgpu_backend::code_generator::SubmissionId;
use std::fmt::Write;
use crate::node_usage_analysis::*;
use std::collections::BinaryHeap;
use std::cmp::Reverse;

// This is a temporary hack
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum Value
{
	Retired,
	LocalVariable(usize),
	GpuBuffer(usize),
}

enum NodeResult
{
	Error,
	Retired,
	None,
	SingleOutput(Value),
	MultipleOutput(Box<[Value]>),
}

#[derive(Debug, Clone, Copy)]
enum GpuResidencyState
{
	Useable(usize),
	Encoded(usize),
	Submitted(usize)
}

// A hack
fn gpu_residency_state_with_var_replaced(state : &GpuResidencyState, variable_id : usize) -> GpuResidencyState
{
	use GpuResidencyState::*;
	match state
	{
		Useable(_) => Useable(variable_id),
		Encoded(_) => Encoded(variable_id),
		Submitted(_) => Submitted(variable_id),
	}
}

#[derive(Debug, Clone, Copy)]
enum LocalResidencyState
{
	Useable(usize),
}

fn local_residency_state_with_var_replaced(state : &LocalResidencyState, variable_id : usize) -> LocalResidencyState
{
	use LocalResidencyState::*;
	match state
	{
		Useable(_) => Useable(variable_id),
	}
}

#[derive(Debug, Default)]
struct PlacementState
{
	node_gpu_residency_states : HashMap<ir::NodeId, GpuResidencyState>,
	node_local_residency_states : HashMap<ir::NodeId, LocalResidencyState>,
	// A hack to match with the old code generator
	node_call_states : HashMap<ir::NodeId, Box<[usize]>>,
	submit_node_submission_ids : HashMap<ir::NodeId, Option<SubmissionId>>,
	pending_submission_node_ids : BinaryHeap<Reverse<ir::NodeId>>,
	node_submission_node_ids : HashMap<ir::NodeId, ir::NodeId>
}

impl PlacementState
{
	fn new() -> Self
	{
		Default::default()
	}

	fn get_local_state_var_ids(&self, node_ids : &[ir::NodeId]) -> Option<Box<[usize]>>
	{
		let mut var_ids = Vec::<usize>::new();
		for node_id in node_ids.iter()
		{
			let var_id = match self.node_local_residency_states.get(node_id)
			{
				Some(LocalResidencyState::Useable(variable_id)) => *variable_id,
				None => return None
			};
			var_ids.push(var_id);
		}
		Some(var_ids.into_boxed_slice())
	}

	fn get_gpu_state_var_ids(&self, node_ids : &[ir::NodeId]) -> Option<Box<[usize]>>
	{
		let mut var_ids = Vec::<usize>::new();
		for node_id in node_ids.iter()
		{
			let var_id = match self.node_gpu_residency_states.get(node_id)
			{
				Some(GpuResidencyState::Useable(variable_id)) => *variable_id,
				Some(GpuResidencyState::Encoded(variable_id)) => *variable_id,
				Some(GpuResidencyState::Submitted(variable_id)) => *variable_id,
				None => return None
			};
			var_ids.push(var_id);
		}
		Some(var_ids.into_boxed_slice())
	}

	/*fn encode_gpu(&mut self, node_ids : &[ir::NodeId], funclet_builder : &mut ir_builders::FuncletBuilder)
	{
		self.transition_gpu(node_ids, funclet_builder, GpuResidencyState::Encoded);
	}

	fn submit_gpu(&mut self, node_ids : &[ir::NodeId], funclet_builder : &mut ir_builders::FuncletBuilder)
	{
		self.transition_gpu(node_ids, funclet_builder, GpuResidencyState::Submitted);
	}

	fn sync_local(&mut self, node_ids : &[ir::NodeId], funclet_builder : &mut ir_builders::FuncletBuilder)
	{

	}*/
}

/*#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct PipelineStageKey
{
	funclet_id : ir::FuncletId,
	funclet_stage_id : Option<usize>,
}

// When pipeline stages surface to the outside world (the calling function) entry points don't strictly correspond to funclets (nodes), but instead correspond to the prior stage and the next funclet (paths)
// This can introduce potentially infinite cycles, so it's important that we not try to do placement inference across funclets lest we wake the sleeping halting problem
// For now we dodge this by forcing local on entry and exit, but this will have to change and it's important to be careful when it does
// Ignore the above
struct PipelineStageData
{
	placement_state_opt : Option<PlacementState>,
	//captured_argument_count : usize,
	//prior_stage_id_opt : Option<usize>
}*/

// PipelineContext tracks traversal through the funclet graph
#[derive(Default)]
struct PipelineContext
{
	//pipeline_stages : HashMap<PipelineStageKey, PipelineStageData>
	funclet_placement_states : HashMap<ir::FuncletId, PlacementState>,
	pending_funclet_ids : Vec<ir::FuncletId>
}

impl PipelineContext
{
	fn new() -> Self
	{
		Default::default()
	}
}

pub struct CodeGen<'program>
{
	program : & 'program ir::Program,
	code_generator : CodeGenerator<'program>,
	print_codegen_debug_info : bool
}

impl<'program> CodeGen<'program>
{
	pub fn new(program : & 'program ir::Program) -> Self
	{
		Self { program : & program, code_generator : CodeGenerator::new(program.types.clone(), program.external_cpu_functions.as_slice(), program.external_gpu_functions.as_slice()), print_codegen_debug_info : false }
	}

	pub fn set_print_codgen_debug_info(&mut self, to : bool)
	{
		self.print_codegen_debug_info = to;
	}

	fn compile_funclet(&mut self, funclet_id : ir::FuncletId, argument_variable_ids : &[usize], pipeline_context : &mut PipelineContext)
	{
		let mut placement_state = PlacementState::new();

		let funclet = & self.program.funclets[& funclet_id];
		assert_eq!(funclet.kind, ir::FuncletKind::MixedExplicit);

		if self.print_codegen_debug_info
		{
			println!("Compiling Funclet #{}...\n{:?}\n", funclet_id, funclet);
		}

		for (current_node_id, node) in funclet.nodes.iter().enumerate()
		{
			self.code_generator.insert_comment(format!(" node #{}: {:?}", current_node_id, node).as_str());

			if self.print_codegen_debug_info
			{
				println!("#{} {:?} : {:?}", current_node_id, node, placement_state);
			}

			match node
			{
				ir::Node::Phi {index} =>
				{
					placement_state.node_local_residency_states.insert(current_node_id, LocalResidencyState::Useable(argument_variable_ids[*index as usize]));
				}
				ir::Node::ExtractResult { node_id, index } =>
				{
					let node_call_state_opt = placement_state.node_call_states.get(node_id);
					assert!(node_call_state_opt.is_some(), "Funclet #{} at node #{} {:?}: Node #{} is not the result of a call {:?}", funclet_id, current_node_id, node, node_id, placement_state);
					let node_call_state = node_call_state_opt.unwrap();

					if let Some(local_residency_state) = placement_state.node_local_residency_states.get(node_id).map(|x| *x)
					{
						placement_state.node_local_residency_states.insert(current_node_id, local_residency_state_with_var_replaced(& local_residency_state, node_call_state[* index as usize]));
					}

					if let Some(gpu_residency_state) = placement_state.node_gpu_residency_states.get(node_id).map(|x| *x)
					{
						placement_state.node_gpu_residency_states.insert(current_node_id, gpu_residency_state_with_var_replaced(& gpu_residency_state, node_call_state[* index as usize]));
					}
				}
				ir::Node::ConstantInteger{value, type_id} =>
				{
					let variable_id = self.code_generator.build_constant_integer(* value, * type_id);
					placement_state.node_local_residency_states.insert(current_node_id, LocalResidencyState::Useable(variable_id));
				}
				ir::Node::ConstantUnsignedInteger{value, type_id} =>
				{
					let variable_id = self.code_generator.build_constant_unsigned_integer(* value, * type_id);
					placement_state.node_local_residency_states.insert(current_node_id, LocalResidencyState::Useable(variable_id));
				}
				ir::Node::CallValueFunction { function_id, arguments } =>
				{
					panic!("Not yet implemented");
					let function = & self.program.value_functions[function_id];
					assert!(function.default_funclet_id.is_some(), "Codegen doesn't know how to handle value functions with no default binding yet");
					let default_funclet_id = function.default_funclet_id.unwrap();
					
					
				}
				ir::Node::CallExternalCpu { external_function_id, arguments } =>
				{
					let argument_var_ids_opt = placement_state.get_local_state_var_ids(arguments);
					assert!(argument_var_ids_opt.is_some(), "#{} {:?}: Not all arguments are local {:?} {:?}", current_node_id, node, arguments, placement_state);
					let argument_var_ids = argument_var_ids_opt.unwrap();
					let raw_outputs = self.code_generator.build_external_cpu_function_call(* external_function_id, & argument_var_ids);
					placement_state.node_local_residency_states.insert(current_node_id, LocalResidencyState::Useable(usize::MAX));
					placement_state.node_call_states.insert(current_node_id, raw_outputs);
				}
				ir::Node::CallExternalGpuCompute {external_function_id, arguments, dimensions} =>
				{
					use std::convert::TryInto;
					//use core::slice::SlicePattern;
					let dimension_var_ids_opt = placement_state.get_local_state_var_ids(dimensions);
					let argument_var_ids_opt = placement_state.get_gpu_state_var_ids(arguments);
					assert!(dimension_var_ids_opt.is_some(), "#{} {:?}: Not all dimensions are local {:?} {:?}", current_node_id, node, dimensions, placement_state);
					assert!(argument_var_ids_opt.is_some(), "#{} {:?}: Not all arguments are gpu {:?} {:?}", current_node_id, node, arguments, placement_state);

					let dimension_var_ids = dimension_var_ids_opt.unwrap();
					let argument_var_ids = argument_var_ids_opt.unwrap();
					let dimensions_slice : &[usize] = & dimension_var_ids;
					let raw_outputs = self.code_generator.build_compute_dispatch(* external_function_id, dimensions_slice.try_into().expect("Expected 3 elements for dimensions"), & argument_var_ids);

					placement_state.node_gpu_residency_states.insert(current_node_id, GpuResidencyState::Encoded(usize::MAX));
					placement_state.node_call_states.insert(current_node_id, raw_outputs);
				}
				ir::Node::EncodeGpu{values} =>
				{
					for node_id in values.iter()
					{
						if let Some(LocalResidencyState::Useable(variable_id)) = placement_state.node_local_residency_states.get(node_id).map(|x| *x)
						{
							let new_variable_id = self.code_generator.make_on_gpu_copy(variable_id).unwrap();
							let old = placement_state.node_gpu_residency_states.insert(* node_id, GpuResidencyState::Encoded(new_variable_id));
							assert!(old.is_none());
						}
						else
						{
							panic!("Encoded node is not locally resident");
						}
					}
				}
				ir::Node::SubmitGpu{values} =>
				{
					for node_id in values.iter()
					{
						if let Some(GpuResidencyState::Encoded(variable_id)) = placement_state.node_gpu_residency_states.get(node_id).map(|x| *x)
						{
							let old = placement_state.node_submission_node_ids.insert(* node_id, current_node_id);
							assert!(old.is_none());
							placement_state.node_gpu_residency_states.insert(* node_id, GpuResidencyState::Submitted(variable_id));
						}
						else
						{
							panic!("Submitted node is not gpu encoded");
						}
					}

					let submission_id = self.code_generator.flush_submission();
					placement_state.submit_node_submission_ids.insert(current_node_id, Some(submission_id));
					placement_state.pending_submission_node_ids.push(Reverse(current_node_id));
				}
				ir::Node::SyncLocal{values} =>
				{
					let mut latest_submission_node_id_opt = None;
					for node_id in values.iter()
					{
						let submission_node_id = *placement_state.node_submission_node_ids.get(node_id).unwrap();
						while placement_state.pending_submission_node_ids.len() > 0
						{
							if let Some(Reverse(pending_submission_node_id)) = placement_state.pending_submission_node_ids.peek().map(|x| *x)
							{
								if pending_submission_node_id < submission_node_id
								{
									placement_state.pending_submission_node_ids.pop();
								}
								else if pending_submission_node_id == submission_node_id
								{
									placement_state.pending_submission_node_ids.pop();
									latest_submission_node_id_opt = Some(pending_submission_node_id);
									break
								}
							}
							else
							{
								break
							}
						}
					}
					
					if let Some(submission_node_id) = latest_submission_node_id_opt
					{
						if let Some(submission_id) = placement_state.submit_node_submission_ids[& submission_node_id]
						{
							self.code_generator.sync_submission(submission_id);
						}
						placement_state.submit_node_submission_ids.insert(submission_node_id, None);
					}

					for node_id in values.iter()
					{
						if let Some(GpuResidencyState::Submitted(variable_id)) = placement_state.node_gpu_residency_states.get(node_id).map(|x| *x)
						{
							// This is a wart with how this code is designed...
							// It should eventually get cleaned up once the scheduling language implementationm is reworked
							assert!(variable_id != usize::MAX, "Cannot synchronize directly on a gpu call because there is no value.  This is a wart resulting from the old scheduling language being value-centric.  This will get fixed.");
							placement_state.node_gpu_residency_states.insert(* node_id, GpuResidencyState::Useable(variable_id));
							let new_variable_id = self.code_generator.make_local_copy(variable_id).unwrap();
							let old = placement_state.node_local_residency_states.insert(* node_id, LocalResidencyState::Useable(new_variable_id));
							assert!(old.is_none());
						}
						else
						{
							panic!("Locally synced node is not gpu submitted");
						}
					}
				}
				_ => panic!("Unknown node")
			};
		}

		match & funclet.tail_edge
		{
			ir::TailEdge::Return { return_values } =>
			{
				let return_var_ids = placement_state.get_local_state_var_ids(return_values).unwrap();

				self.code_generator.build_return(& return_var_ids);
			}
			ir::TailEdge::Yield { funclet_ids, captured_arguments, return_values } =>
			{
				let captured_argument_var_ids = placement_state.get_local_state_var_ids(captured_arguments).unwrap();
				let return_var_ids = placement_state.get_local_state_var_ids(return_values).unwrap();

				let mut next_funclet_input_types = Vec::<Box<[ir::TypeId]>>::new();
				for & next_funclet_id in funclet_ids.iter()
				{
					pipeline_context.pending_funclet_ids.push(next_funclet_id);
					/*if ! pipeline_context.funclet_placement_states.contains_key(& funclet_id)
					{
					}*/
					let input_types = self.program.funclets[& next_funclet_id].input_types.to_vec();
					//let input_types = Vec::<ir::TypeId>::new();
					next_funclet_input_types.push(input_types.into_boxed_slice());
				}
				// Proper codegen is a lot more complicated than this
				// self.code_generator.build_yield(& captured_argument_var_ids, & return_var_ids);
				// This is disgusting
				self.code_generator.build_yield(funclet_ids, next_funclet_input_types.into_boxed_slice(), & captured_argument_var_ids, & return_var_ids);
			}
		}

		let old = pipeline_context.funclet_placement_states.insert(funclet_id, placement_state);
		assert!(old.is_none());
	}

	/*fn generate_pipeline_stage(&mut self, pipeline_context : &mut PipelineContext, parent_stage_id_opt : Option<usize>) -> usize
	{

	}*/

	fn generate_cpu_function(&mut self, entry_funclet_id : ir::FuncletId, pipeline_name : &str)
	{
		let entry_funclet = & self.program.funclets[& entry_funclet_id];
		assert_eq!(entry_funclet.kind, ir::FuncletKind::MixedExplicit);

		let mut pipeline_context = PipelineContext::new();
		pipeline_context.pending_funclet_ids.push(entry_funclet_id);

		self.code_generator.begin_pipeline(pipeline_name);

		while let Some(funclet_id) = pipeline_context.pending_funclet_ids.pop()
		{
			if ! pipeline_context.funclet_placement_states.contains_key(& funclet_id)
			{
				let funclet = & self.program.funclets[& funclet_id];
				assert_eq!(funclet.kind, ir::FuncletKind::MixedExplicit);

				let argument_variable_ids = self.code_generator.begin_funclet(funclet_id, &funclet.input_types, &funclet.output_types);
				self.compile_funclet(funclet_id, & argument_variable_ids, &mut pipeline_context);
				self.code_generator.end_funclet();
			}
		}

		/*match & entry_funclet.tail_edge
		{
			ir::TailEdge::Return {return_values : _} =>
			{
				let argument_variable_ids = self.code_generator.begin_oneshot_entry_funclet(&entry_funclet.input_types, &entry_funclet.output_types);
				self.compile_funclet(entry_funclet_id, & argument_variable_ids, &mut pipeline_context);
				self.code_generator.end_funclet();
			}

			ir::TailEdge::Yield {funclet_ids : _, captured_arguments : _, return_values : _} => 
			{
				()
			}
			//self.code_generator.begin_corecursive_base_funclet(pipeline_name, &entry_funclet.input_types, &entry_funclet.output_types),
		};*/

		self.code_generator.emit_pipeline_entry_point(entry_funclet_id, &entry_funclet.input_types, &entry_funclet.output_types);
		
		match & entry_funclet.tail_edge
		{
			ir::TailEdge::Return {return_values : _} =>
			{
				self.code_generator.emit_oneshot_pipeline_entry_point(entry_funclet_id, &entry_funclet.input_types, &entry_funclet.output_types);
			}

			ir::TailEdge::Yield {funclet_ids : _, captured_arguments : _, return_values : _} => 
			{
				()
			}
		};

		self.code_generator.end_pipeline();
	}

	pub fn generate<'codegen>(& 'codegen mut self) -> String
	{
		for pipeline in self.program.pipelines.iter()
		{
			self.generate_cpu_function(pipeline.entry_funclet, pipeline.name.as_str());
		}

		return self.code_generator.finish();
	}
}

#[cfg(test)]
mod tests
{
	use super::*;
	use crate::ir;
}

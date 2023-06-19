use crate::ir;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::default::Default;
use super::error::{Error, ErrorContext};
//#[macro_use]
//use super::{error_ifn_eq, };

macro_rules! error_ifn_eq {
	($ctx:expr, $left:expr, $right:expr $(,)?) => {
		match (&$left, &$right) {
			(left_val, right_val) => {
				if !(*left_val == *right_val) {
					Result::<(), Error>::Err(assert_failed($ctx, &*left_val, &*right_val))
				}
				else {
					Result::<(), Error>::Ok(())
				}
			}
		}
	}
}

fn assert_failed<T : std::fmt::Debug>(error_context : &ErrorContext, a : T, b : T) -> Error {
	error_context.generic_error(& format!("{:?} != {:?}", a, b))
}

// Language independent plumbing for type checking

/*#[derive(Debug)]
pub struct Scalar
{
	pub tag : ir::Tag,
	pub flow : ir::Flow,
}*/
pub type Scalar = ir::Tag;

#[derive(Debug)]
struct JoinPoint {
    implicit_tag: ir::Tag,
    input_tags: Box<[ir::Tag]>,
	//input_flows: Box<[ir::Flow]>,
}


// This state really isn't meant to be public, so don't rely on it.
#[derive(Debug)]
pub struct FuncletSpecChecker<'program> {
	pub program : &'program ir::Program,
    pub spec_funclet: &'program ir::Funclet,
    pub funclet_spec: &'program ir::FuncletSpec,
    pub scalar_nodes: HashMap<ir::NodeId, Scalar>,
    join_nodes: HashMap<ir::NodeId, JoinPoint>,
    //current_node_id: ir::NodeId,
    pub current_implicit_tag: ir::Tag,
}

//{tags : &[ir::Tag], flows : &[ir::Flow], implicit_tag : ir::Tag, stage : bool},

/*

pub enum Rule {
	Jump{rule_index: usize},
	Enter{rule_index: usize},
	Exit{rule_index: usize},
	Choice{rule_indices: &[usize], unified_spec_node_ids: &[ir::NodeId], unified_spec_node_id : ir::NodeId},
}*/

/*struct BorrowedSnapshot<'scope> {
	tags : &'scope [ir::Tag],
	flows : &'scope [ir::Flow],
	implicit_tag : ir::Tag,
}

#[derive(Clone)]
struct Snapshot {
	tags : Box<[ir::Tag]>,
	flows : Box<[ir::Flow]>,
	implicit_tag : ir::Tag,
	// For now, implicit flow is always Have
}

impl Snapshot {
	fn unify(snapshots : &[&Self] unified_spec_node_ids: &[ir::NodeId], unifying_spec_node_id : ir::NodeId) -> Self {

	}

	fn enter(&mut self, input_spec_node_ids : &[ir::NodeId]) {
		// input_spec_node_ids are the nodes in the caller spec funclet that need to be substituted into the inputs for the callee spec funclet
		/*for (impl_index, tag) in self.tags.iter_mut() {
			*tag = match *tag {
				ir::Tag::Node{node_id} => {
					for (input_index, input_node_id)
				}
				_ => panic!("SHould be a node"),
			};
		}*/
	}

	fn exit(&mut self) {

	}

	fn jump(&mut self) {
		
	}
}*/

/*fn check_rules(rules : &[Rule]) {

}*/

// To do: Better error reporting

impl<'program> FuncletSpecChecker<'program> {
    pub fn new(program : &'program ir::Program, spec_funclet: &'program ir::Funclet, funclet_spec : &'program ir::FuncletSpec) -> Self {
        let mut state = Self {
			program,
            spec_funclet,
            funclet_spec,
            scalar_nodes: HashMap::new(),
            join_nodes: HashMap::new(),
            //current_node_id: 0,
            current_implicit_tag: funclet_spec.implicit_in_tag,
        };
        state.initialize();
        state
    }

	fn initialize(&mut self) {
		self.current_implicit_tag = concretize_input_to_internal_tag(self.current_implicit_tag);
		for input_index in 0 .. self.funclet_spec.input_tags.len()
		{
			let mut tag = self.funclet_spec.input_tags[input_index];
			tag = concretize_input_to_internal_tag(tag);
			self.scalar_nodes.insert(input_index, tag);
		}
		// To do: DefaultJoin should probably immediately follow last input
	}

	pub fn update_scalar_node(&mut self, node_id : ir::NodeId, quot : ir::Quotient, flow : ir::Flow) {
		self.scalar_nodes.insert(node_id, Scalar{quot, flow});
	}

	pub fn update_join_node(&mut self, node_id : ir::NodeId, tags : &[ir::Tag], implicit_tag : ir::Tag) {
		self.join_nodes.insert(node_id, JoinPoint{implicit_tag, input_tags: tags.to_vec().into_boxed_slice()});
	}

	fn contextualize_error(&self, writer : &mut dyn std::fmt::Write) -> Result<(), std::fmt::Error> {
		write!(writer, "Checking spec funclet\nSpec {:?}\nSpec Funclet {:?}\nScalar Nodes {:?}\nJoin Nodes {:?}\nImplicit Tag {:?}\n", self.funclet_spec, self.spec_funclet, self.scalar_nodes, self.join_nodes, self.current_implicit_tag)
	}

	pub fn join(&mut self, error_context : &ErrorContext, node_id : ir::NodeId, capture_node_ids : &[ir::NodeId], funclet_spec : &ir::FuncletSpec, continuation_node_id : ir::NodeId) -> Result<(), Error> {
		let continuation_join = self.join_nodes.remove(& continuation_node_id).unwrap();
		
        check_tag_compatibility_interior(
			error_context,
            self.spec_funclet,
            funclet_spec.implicit_out_tag,
            continuation_join.implicit_tag,
        )?;

        for (capture_index, capture_node_id) in capture_node_ids.iter().enumerate() {
            let scalar = & self.scalar_nodes[capture_node_id];
			
			match scalar.flow {
				ir::Flow::Have => (), // Can borrow
				ir::Flow::Met => (), // Can duplicate borrow
				_ => panic!("Capturing {:?} is unsupported", scalar.flow),
			}
			assert_eq!(funclet_spec.input_tags[capture_index].flow, scalar.flow, "\n{}", error_context);

			check_tag_compatibility_interior(
				error_context,
				self.spec_funclet,
				*scalar,
				funclet_spec.input_tags[capture_index],
			)?;

			let quot = scalar.quot;
			self.update_scalar_node(*capture_node_id, quot, ir::Flow::Met);
		}

		let mut remaining_input_tags = Vec::<ir::Tag>::new();
        for index in capture_node_ids.len() .. funclet_spec.input_tags.len() {
			//assert_eq!(funclet_spec.input_tags[index].flow, ir::Flow::Have, "\n{}", error_context);
			remaining_input_tags.push(funclet_spec.input_tags[index]);
		}

		assert_eq!(funclet_spec.output_tags.len(), continuation_join.input_tags.len());
        for index in 0 .. funclet_spec.output_tags.len() {
			//assert_eq!(funclet_spec.output_tags[index].flow, ir::Flow::Have, "\n{}", error_context);
			//assert_eq!(continuation_join.input_tags[index].flow, ir::Flow::Have, "\n{}", error_context);

            check_tag_compatibility_interior(
				error_context,
                self.spec_funclet,
                funclet_spec.output_tags[index],
                continuation_join.input_tags[index],
            )?;
		}

		self.update_join_node(node_id, remaining_input_tags.as_slice(), funclet_spec.implicit_in_tag);

		return Ok(());
	}

	pub fn initialize_default_join(&mut self, node_id : ir::NodeId) {
		self.update_join_node(node_id, & self.funclet_spec.output_tags, self.funclet_spec.implicit_out_tag);
	}

	pub fn check_jump(&mut self, old_error_context : &ErrorContext, continuation_node_id : ir::NodeId, argument_node_ids : &[ir::NodeId]) -> Result<(), Error> {
		let continuation_join = self.join_nodes.remove(& continuation_node_id).unwrap();

		let immutable_self : &Self = self;
		let error_contextualizer = |writer : &mut std::fmt::Write| { immutable_self.contextualize_error(writer) };
		let error_context = & ErrorContext::new(Some(old_error_context), Some(& error_contextualizer));

        check_tag_compatibility_interior(
			error_context,
            self.spec_funclet,
            self.current_implicit_tag,
            continuation_join.implicit_tag,
        )?;
		
		assert_eq!(argument_node_ids.len(), continuation_join.input_tags.len());
        for index in 0 .. argument_node_ids.len() {
			let Some(scalar) = self.scalar_nodes.get(& argument_node_ids[index]) else {
				panic!("Jump input #{}, impl node #{} has no tag for spec\n{}", index, argument_node_ids[index], error_context)
			};
			//assert_eq!(scalar.flow, ir::Flow::Have, "\n{}", error_context);
			//assert_eq!(continuation_join.input_tags[index].flow, ir::Flow::Have, "\n{}", error_context);

            check_tag_compatibility_interior(
				error_context,
                self.spec_funclet,
                *scalar,
                continuation_join.input_tags[index],
            )?;
		}

		return Ok(());
	}

	pub fn check_return(&mut self, old_error_context : &ErrorContext, return_value_node_ids : &[ir::NodeId]) -> Result<(), Error> {
		let return_error_contextualizer = |writer : &mut std::fmt::Write| { self.contextualize_error(writer) };
		let return_error_context = ErrorContext::new(Some(old_error_context), Some(& return_error_contextualizer));

        check_tag_compatibility_interior(
			&return_error_context,
            self.spec_funclet,
            self.current_implicit_tag,
            self.funclet_spec.implicit_out_tag,
        )?;
		
		assert_eq!(return_value_node_ids.len(), self.funclet_spec.output_tags.len());
        for index in 0 .. return_value_node_ids.len() {
			let scalar = & self.scalar_nodes[& return_value_node_ids[index]];
			assert_eq!(scalar.flow, ir::Flow::Have);
			assert_eq!(self.funclet_spec.output_tags[index].flow, ir::Flow::Have);

            check_tag_compatibility_interior(
				&return_error_context,
                self.spec_funclet,
                *scalar,
                self.funclet_spec.output_tags[index],
            )?;
		}
		
		return Ok(());
	}

	pub fn check_interior_call(&mut self, error_context : &ErrorContext, continuation_node_id : ir::NodeId, argument_node_ids : &[ir::NodeId], callee_funclet_spec : &ir::FuncletSpec) -> Result<(), Error> {
        check_tag_compatibility_interior(
			error_context,
            self.spec_funclet,
            self.current_implicit_tag,
            callee_funclet_spec.implicit_in_tag,
        )?;

		let continuation_join = self.join_nodes.remove(& continuation_node_id).unwrap();
        check_tag_compatibility_interior(
			error_context,
            self.spec_funclet,
            callee_funclet_spec.implicit_out_tag,
            continuation_join.implicit_tag,
        )?;

		assert_eq!(argument_node_ids.len(), callee_funclet_spec.input_tags.len());
        for index in 0 .. argument_node_ids.len() {
			let scalar = & self.scalar_nodes[& argument_node_ids[index]];

            check_tag_compatibility_interior(
				error_context,
                self.spec_funclet,
                *scalar,
                callee_funclet_spec.input_tags[index],
            )?;
		}

		assert_eq!(continuation_join.input_tags.len(), callee_funclet_spec.output_tags.len());
        for index in 0 .. callee_funclet_spec.output_tags.len() {
            check_tag_compatibility_interior(
				error_context,
                self.spec_funclet,
                callee_funclet_spec.output_tags[index],
                continuation_join.input_tags[index],
            )?;
		}

		return Ok(());
	}

	pub fn check_vertical_call(&mut self, error_context : &ErrorContext, continuation_node_id : ir::NodeId, input_impl_node_ids : &[ir::NodeId], callee_funclet_spec : &ir::FuncletSpec, output_spec_node_ids : &[ir::NodeId], call_spec_node_id : ir::NodeId) -> Result<(), Error> {
        check_tag_compatibility_enter(
			error_context,
            output_spec_node_ids,
            self.current_implicit_tag,
            callee_funclet_spec.implicit_in_tag,
        )?;

		let continuation_join = self.join_nodes.remove(& continuation_node_id).unwrap();
        check_tag_compatibility_exit(
			error_context,
            self.spec_funclet,
			call_spec_node_id,
            callee_funclet_spec.implicit_out_tag,
            continuation_join.implicit_tag,
        )?;

		assert_eq!(input_impl_node_ids.len(), callee_funclet_spec.input_tags.len());
        for index in 0 .. input_impl_node_ids.len() {
			let scalar = & self.scalar_nodes[& input_impl_node_ids[index]];

            check_tag_compatibility_enter(
				error_context,
                output_spec_node_ids,
                *scalar,
                callee_funclet_spec.input_tags[index],
            )?;
		}

		assert_eq!(continuation_join.input_tags.len(), callee_funclet_spec.output_tags.len());
        for index in 0 .. callee_funclet_spec.output_tags.len() {
            check_tag_compatibility_exit(
				error_context,
                self.spec_funclet,
				call_spec_node_id,
                callee_funclet_spec.output_tags[index],
                continuation_join.input_tags[index],
            )?;
		}

		return Ok(());
	}

	pub fn check_call(&mut self, error_context : &ErrorContext, operation: ir::Quotient, continuation_impl_node_id : ir::NodeId, input_impl_node_ids : &[ir::NodeId], callee_funclet_spec : &ir::FuncletSpec) -> Result<(), Error> {
		match operation {
			ir::Quotient::Node{node_id} => {
				if let ir::Node::CallFunctionClass{function_id, arguments} = &self.spec_funclet.nodes[node_id] {
					let Some(callee_spec_funclet_id) = callee_funclet_spec.funclet_id_opt else { panic!("Does not have a spec funclet id") };
					let callee_spec_funclet = & self.program.funclets[callee_spec_funclet_id];
					match & callee_spec_funclet.spec_binding {
						ir::FuncletSpecBinding::Value{value_function_id_opt: Some(value_function_id)} if *value_function_id == *function_id => {},
						_ => return Err(error_context.generic_error(& format!("Callee spec funclet #{} does not implement function class #{}", callee_spec_funclet_id, *function_id)))
					}
					self.check_vertical_call(error_context, continuation_impl_node_id, input_impl_node_ids, callee_funclet_spec, arguments, node_id)
				}
				else {
					panic!("Not a call")
				}
			}
			ir::Quotient::None => self.check_interior_call(error_context, continuation_impl_node_id, input_impl_node_ids, callee_funclet_spec),
			_ => panic!("Unsupported: {:?}", operation),
		}
	}

	pub fn check_choice(&mut self, error_context : &ErrorContext, continuation_impl_node_id : ir::NodeId, input_impl_node_ids : &[ir::NodeId], choice_remaps : &[&[(ir::NodeId, ir::NodeId)]], choice_specs : &[&ir::FuncletSpec]) -> Result<(), Error> {
		let continuation_join = self.join_nodes.remove(& continuation_impl_node_id).unwrap();

		assert_eq!(choice_remaps.len(), choice_specs.len());
		for choice_index in 0 .. choice_specs.len() {
			let choice_spec = & choice_specs[choice_index];
			let choice_remap = choice_remaps[choice_index];
			// 
			check_tag_compatibility_interior(
				error_context,
				self.spec_funclet,
				self.current_implicit_tag,
				choice_spec.implicit_in_tag,
			)?;

			assert_eq!(input_impl_node_ids.len(), choice_spec.input_tags.len());
			for index in 0 .. input_impl_node_ids.len() {
				let scalar = & self.scalar_nodes[& input_impl_node_ids[index]];

				check_tag_compatibility_interior(
					error_context,
					self.spec_funclet,
					*scalar,
					choice_spec.input_tags[index],
				)?;
			}

			// Continuation gets the unified result
			check_tag_compatibility_interior_cast(
				error_context,
				self.spec_funclet,
				choice_spec.implicit_out_tag,
				continuation_join.implicit_tag,
				choice_remap
			)?;

			assert_eq!(continuation_join.input_tags.len(), choice_spec.output_tags.len());
			for index in 0 .. choice_spec.output_tags.len() {
				check_tag_compatibility_interior_cast(
					error_context,
					self.spec_funclet,
					choice_spec.output_tags[index],
					continuation_join.input_tags[index],
					choice_remap
				)?;
			}
		}

		return Ok(());
	}

	// To do: Join
	pub fn can_drop_node(&self, impl_node_id : ir::NodeId) -> bool {
		if let Some(scalar) = & self.scalar_nodes.get(& impl_node_id) {
			return scalar.flow.is_droppable();
		}
		return true;
	}

	pub fn is_neutral_node(&self, impl_node_id : ir::NodeId) -> bool {
		if let Some(scalar) = & self.scalar_nodes.get(& impl_node_id) {
			return scalar.flow.is_neutral();
		}
		return true;
	}

	// To do: Join
	pub fn drop_node(&mut self, impl_node_id : ir::NodeId)  {
		self.scalar_nodes.remove(& impl_node_id);
	}

	pub fn check_implicit_tag(&self, old_error_context : &ErrorContext, tag : ir::Tag) -> Result<(), Error>  {
		let error_contextualizer = |writer : &mut std::fmt::Write| { self.contextualize_error(writer) };
		let error_context = & ErrorContext::new(Some(old_error_context), Some(& error_contextualizer));

		//assert_eq!(tag, self.current_implicit_tag);
		check_tag_compatibility_interior(
			error_context,
			self.spec_funclet,
			self.current_implicit_tag,
			tag,
		)?;//.map_err(|e| e.append_message(format!("While checking that the implicit tag {:?} is compatible with {:?}", self.current_implicit_tag, tag)))?;

		return Ok(());
	}

	pub fn check_node_tag(&self, old_error_context : &ErrorContext, node_id : ir::NodeId, tag : ir::Tag) -> Result<(), Error> {
		let error_contextualizer = |writer : &mut std::fmt::Write| { self.contextualize_error(writer) };
		let error_context = & ErrorContext::new(Some(old_error_context), Some(& error_contextualizer));

		let Some(scalar) = self.scalar_nodes.get(& node_id) else {
			panic!("Impl node #{} has no tag for this spec\n{}", node_id, error_context)
		};
		//assert_eq!(*scalar, tag);
		check_tag_compatibility_interior(
			error_context,
			self.spec_funclet,
			*scalar,
			tag,
		).map_err(|e| e.append_message(format!("While checking that node #{} has tag {:?}", node_id, tag)))?;

		return Ok(());
	}

	pub fn check_node_is_current_with_implicit(&self, error_context : &ErrorContext, node_id : ir::NodeId) -> Result<(), Error> {
		self.check_node_tag(error_context, node_id, self.current_implicit_tag)
	}

	pub fn check_node_is_readable_at_implicit(&self, error_context : &ErrorContext, node_id : ir::NodeId) -> Result<(), Error> {
		let scalar = self.scalar_nodes.get(& node_id).unwrap();
		if ! scalar.flow.is_readable() {
			return Err(Error::Generic{message: format!("Node is not readable")})
		}
		let tag = ir::Tag{quot: self.current_implicit_tag.quot, flow: scalar.flow};
		//assert_eq!(*scalar, tag);
		check_tag_compatibility_interior(
			error_context,
			self.spec_funclet,
			*scalar,
			tag,
		).map_err(|e| e.append_message(format!("While checking that node #{} has tag {:?}", node_id, tag)))?;

		return Ok(());
	}

	pub fn check_node_is_readable_at(&self, error_context : &ErrorContext, node_id : ir::NodeId, reader_tag : ir::Tag) -> Result<(), Error> {
		let scalar = self.scalar_nodes.get(& node_id).unwrap();
		if ! scalar.flow.is_readable() {
			return Err(Error::Generic{message: format!("Node is not readable")})
		}
		assert_eq!(reader_tag.flow, ir::Flow::Have);
		let tag = ir::Tag{quot: reader_tag.quot, flow: scalar.flow};
		//assert_eq!(*scalar, tag);
		check_tag_compatibility_interior(
			error_context,
			self.spec_funclet,
			*scalar,
			tag,
		).map_err(|e| e.append_message(format!("While checking that node #{} has tag {:?}", node_id, tag)))?;

		return Ok(());
	}

	pub fn update_node_current_with_implicit(&mut self, node_id : ir::NodeId) {
		self.update_scalar_node(node_id, self.current_implicit_tag.quot, self.current_implicit_tag.flow)
	}

	pub fn check_join_tags(&self, error_context : &ErrorContext, node_id : ir::NodeId, input_tags : &[ir::Tag], implicit_in_tag : ir::Tag) -> Result<(), Error> {
		let join = self.join_nodes.get(& node_id).unwrap();
		//assert_eq!(*scalar, tag);
		for index in 0 .. join.input_tags.len() {
			check_tag_compatibility_interior(
				error_context,
				self.spec_funclet,
				input_tags[index],
				join.input_tags[index],
			).map_err(|e| e.append_message(format!("While checking that join #{} input #{} has tag {:?}", node_id, index, input_tags[index])))?;
		}

		check_tag_compatibility_interior(
			error_context,
			self.spec_funclet,
			implicit_in_tag,
			join.implicit_tag,
		).map_err(|e| e.append_message(format!("While checking that join #{} has implicit input tag {:?}", node_id, implicit_in_tag)))?;

		return Ok(());
	}

	fn transition_tag_forwards(tag : &mut ir::Tag, from_spec_node_id : ir::NodeId, to_spec_node_id : ir::NodeId) -> Result<(), Error> {
		match tag.quot {
			ir::Quotient::Node{node_id} if node_id == from_spec_node_id => {
				match tag.flow {
					ir::Flow::None => (),
					ir::Flow::Have => {
						tag.quot = ir::Quotient::Node{node_id: to_spec_node_id};
					}
					ir::Flow::Need => panic!("Flow::Need cannot advance forwards"),
					ir::Flow::Met => panic!("Flow::Met cannot advance"),
				}
			}
			_ => (),
		}

		return Ok(());
	}

	pub fn transition_state_forwards(&mut self, from_spec_node_id : ir::NodeId, to_spec_node_id : ir::NodeId) -> Result<(), Error> {
		for (impl_node_id, scalar) in self.scalar_nodes.iter_mut() {
			Self::transition_tag_forwards(scalar, from_spec_node_id, to_spec_node_id)?;
		}

		Self::transition_tag_forwards(&mut self.current_implicit_tag, from_spec_node_id, to_spec_node_id)?;
		return Ok(());
	}

	pub fn transition_state_subset_forwards(&mut self, impl_node_ids : &[ir::NodeId], from_spec_node_id : ir::NodeId, to_spec_node_id : ir::NodeId) -> Result<(), Error> {
		for impl_node_id in impl_node_ids.iter() {
			let scalar = self.scalar_nodes.get_mut(impl_node_id).unwrap();
			Self::transition_tag_forwards(scalar, from_spec_node_id, to_spec_node_id)?;
		}
		return Ok(());
	}

	/*fn check_single_output(&self, spec_node_id : ir::NodeId, impl_node_id : ir::NodeId) {
		let tag = self.scalar_nodes[&impl_node_id].tag;
		match tag {
			ir::Tag::None => (),
			ir::Tag::Node { node_id } => {
				assert_eq!(spec_node_id, node_id);
			}
			_ => panic!("{:?} is unsupported in the body of a funclet", tag),
		}
	}

    fn check_multiple_output(&self, spec_node_id : ir::NodeId, output_impl_node_ids : &[ir::NodeId]) {
		for (output_index, output_impl_node_id) in output_impl_node_ids.iter().enumerate() {
			let tag = self.scalar_nodes[output_impl_node_id].tag;
			match tag {
				ir::Tag::None => (),
				ir::Tag::Node { node_id } => {
					if let ir::Node::ExtractResult {
						node_id: node_id_2,
						index,
					} = &self.spec_funclet.nodes[node_id]
					{
						assert_eq!(output_index, *index);
						assert_eq!(spec_node_id, *node_id_2);
					}
				}
				_ => panic!("{:?} is unsupported in the body of a funclet", tag),
			}
		}
    }*/

   /* pub fn check_do_outputs(&self, spec_node_id : ir::NodeId, output_impl_node_ids : &[ir::NodeId]) {
		let encoded_node = & self.spec_funclet.nodes[spec_node_id];
        let output_is_tuple = match encoded_node {
            // Single return nodes
            ir::Node::Constant { .. } => false,
            ir::Node::Select { .. } => false,
            // Multiple return nodes
            ir::Node::CallValueFunction { .. } => true,
            _ => panic!("Unsupported node: {:?}", encoded_node),
        };

        if output_is_tuple {
			self.check_multiple_output(spec_node_id, output_impl_node_ids);
        } else {
            assert_eq!(output_impl_node_ids.len(), 1);
			self.check_single_output(spec_node_id, output_impl_node_ids[0]);
        }
    }

	fn check_multiple_inputs(&self, spec_node_id : ir::NodeId, input_spec_node_ids : &[ir::NodeId], input_impl_node_ids : &[ir::NodeId]) {
		assert_eq!(input_spec_node_ids.len(), input_impl_node_ids.len());
		for (input_index, input_impl_node_id) in input_impl_node_ids.iter().enumerate() {
			let impl_tag = self.scalar_nodes[input_impl_node_id].tag;
			check_tag_compatibility_interior(&self.spec_funclet, impl_tag, ir::ValueTag::Node { node_id: input_spec_node_ids[input_index] });
		}
	}

	pub fn check_do_inputs(&self, spec_node_id : ir::NodeId, external_function_id_opt : Option<ir::ffi::ExternalFunctionId>, input_impl_node_ids : &[ir::NodeId]) {
		let encoded_node = & self.spec_funclet.nodes[spec_node_id];
		match encoded_node {
			ir::Node::Constant { .. } => {
				self.check_multiple_inputs(spec_node_id, &[], input_impl_node_ids);
				assert!(external_function_id_opt.is_none());
			}
			ir::Node::Select {
				condition,
				true_case,
				false_case,
			} => {
				self.check_multiple_inputs(spec_node_id, &[*condition, *true_case, *false_case], input_impl_node_ids);
				assert!(external_function_id_opt.is_none());
			}
			ir::Node::CallValueFunction {
				function_id,
				arguments,
			} => {
				let external_function_id = external_function_id_opt.unwrap();
				assert!(self.program.function_classes[*function_id].external_function_ids.contains(& external_function_id));
				let function = &self.program.native_interface.external_functions[external_function_id.0];
				// To do
			}
			_ => panic!("Unsupported node: {:?}", encoded_node)
		}
	}*/

	// Is this a valid transition?
	/*pub fn check_forward_do(&self, spec_node_id : ir::NodeId, input_impl_node_ids : &[ir::NodeId], input_forwarding_remaps : &[Option<usize>], output_impl_node_ids : &[ir::NodeId])
	{
		
	}*/

	// Do the transition, clobbering outputs

	// These two have very different semantics (output tags are irrelevant to advance)
}

fn concretize_input_to_internal_tag(tag: ir::Tag) -> ir::Tag {
    let quot = match tag.quot {
        ir::Quotient::None => ir::Quotient::None,
        ir::Quotient::Node { node_id } => ir::Quotient::Node { node_id },
        ir::Quotient::Input { index } => ir::Quotient::Node { node_id: index },
        ir::Quotient::Output { index } => ir::Quotient::Output { index },
    };
	ir::Tag{quot, flow: tag.flow}
}

fn check_tag_compatibility_enter(
	error_context : &ErrorContext,
    input_spec_node_ids : &[ir::NodeId],
    caller_tag: ir::Tag,
    callee_tag: ir::Tag,
) -> Result<(), Error>  {
	assert_eq!(caller_tag.flow, callee_tag.flow);
    match (caller_tag.quot, callee_tag.quot) {
        (ir::Quotient::None, ir::Quotient::None) => (),
        (ir::Quotient::Node { node_id }, ir::Quotient::Input { index }) => {
            assert_eq!(input_spec_node_ids[index], node_id);
        }
        _ => return Err(Error::Generic{message: format!(
            "Ill-formed: {:?} to {:?} via enter",
            caller_tag, callee_tag
        )}),
    }
	return Ok(());
}

// Check value tag in callee (source) scope transfering to caller (destination) scope
fn check_tag_compatibility_exit(
	error_context : &ErrorContext,
    caller_spec_funclet: &ir::Funclet,
    caller_spec_node_id: ir::NodeId,
    source_tag: ir::Tag,
    destination_tag: ir::Tag,
) -> Result<(), Error>  {
	assert_eq!(source_tag.flow, ir::Flow::Have);
	assert_eq!(destination_tag.flow, ir::Flow::Have);
    match (source_tag.quot, destination_tag.quot) {
        (ir::Quotient::None, ir::Quotient::None) => (),
        (ir::Quotient::Output {index: output_index}, ir::Quotient::Node { node_id }) => {
            let node = &caller_spec_funclet.nodes[node_id];
            if let ir::Node::ExtractResult {node_id: call_node_id, index} = node {
                assert_eq!(*index, output_index);
                assert_eq!(*call_node_id, caller_spec_node_id);
            }
            else {
                panic!(
                    "Target operation is not a result extraction: #{:?} {:?}",
                    node_id, node
                );
            }
        }
        _ => panic!(
            "Ill-formed: {:?} to {:?}",
            source_tag, destination_tag
        ),
    };

	return Ok(());
}

fn check_tag_compatibility_interior_cast(
	error_context : &ErrorContext,
    current_spec_funclet: &ir::Funclet,
    source_tag: ir::Tag,
    destination_tag: ir::Tag,
    casts : &[(ir::NodeId, ir::NodeId)]
) -> Result<(), Error> {
	match (source_tag, destination_tag) {
		(ir::Tag{quot: ir::Quotient::Node{node_id: src_node_id}, flow: src_flow}, ir::Tag{quot: ir::Quotient::Node{node_id: dst_node_id}, flow: dst_flow}) => {
			if src_flow == dst_flow && casts.contains(&(src_node_id, dst_node_id)) {
				return Ok(());
			}
		}
		_ => (),
	}

    check_tag_compatibility_interior(
		error_context,
        current_spec_funclet,
        source_tag,
        destination_tag,
    )?;

	return Ok(());
}

// Check value tag transition in same scope
fn check_tag_compatibility_interior(
	error_context : &ErrorContext,
    current_value_funclet: &ir::Funclet,
    source_tag: ir::Tag,
    destination_tag: ir::Tag
) -> Result<(), Error> {
	assert_eq!(source_tag.flow, destination_tag.flow);
	let flow = source_tag.flow;

    match (source_tag.quot, destination_tag.quot) {
        (ir::Quotient::None, ir::Quotient::None) => (),
        (_, ir::Quotient::None) if flow.is_droppable() => (),
        (ir::Quotient::None, _) if flow.is_duplicable() => (),
		// Input and the first few nodes are equivalent
        ( ir::Quotient::Input { index }, ir::Quotient::Node { node_id: remote_node_id } ) if flow == ir::Flow::Have => {
            if let ir::Node::Phi { index: phi_index } =
                &current_value_funclet.nodes[remote_node_id]
            {
                assert_eq!(*phi_index, index);
            } else {
                panic!("While checking interior compatibility of {:?} to {:?}: {:?} is not a phi", source_tag, destination_tag, current_value_funclet.nodes[remote_node_id]);
            }
        }
        ( ir::Quotient::Node { node_id: remote_node_id }, ir::Quotient::Input { index } ) if flow == ir::Flow::Need => {
            if let ir::Node::Phi { index: phi_index } =
                &current_value_funclet.nodes[remote_node_id]
            {
                assert_eq!(*phi_index, index);
            } else {
                panic!("While checking interior compatibility of {:?} to {:?}: {:?} is not a phi", source_tag, destination_tag, current_value_funclet.nodes[remote_node_id]);
            }
        }
        (ir::Quotient::Node { node_id }, ir::Quotient::Node { node_id: node_id_2 }) => {
            assert_eq!(node_id, node_id_2, "\n{}", error_context);
			/*if node_id != node_id_2 {

				//return Err(Error::Generic{message: format!("Tag of node #{} is not compatibile with tag of node #{}\n{}", node_id, node_id_2, error_context)});
			}*/
        }
		// An output is not necessarily equivalent to a node (there are some monad things going on)
        ( ir::Quotient::Node { node_id }, ir::Quotient::Output { index } ) if flow == ir::Flow::Have => {
            match &current_value_funclet.tail_edge {
                ir::TailEdge::Return { return_values } => { error_ifn_eq!(error_context, return_values[index], node_id)?; }
                _ => panic!("Not a unit"),
            }
        }
        ( ir::Quotient::Output { index }, ir::Quotient::Node { node_id } ) if flow == ir::Flow::Need => {
            match &current_value_funclet.tail_edge {
                ir::TailEdge::Return { return_values } => assert_eq!(return_values[index], node_id),
                _ => panic!("Not a unit"),
            }
        }
        (
            ir::Quotient::Output { index },
            ir::Quotient::Output { index: index_2 },
        ) => {
            //assert_eq!(funclet_id, funclet_id_2);
            assert_eq!(index, index_2);
        }
        _ => panic!(
            "Ill-formed: {:?} to {:?}",
            source_tag, destination_tag
        ),
    }

	Ok(())
}

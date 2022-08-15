use crate::ir;

pub fn concretize_input_to_internal_timeline_tag(program : & ir::Program, timeline_tag : ir::TimelineTag) -> ir::TimelineTag
{
	match timeline_tag
	{
		ir::TimelineTag::None => ir::TimelineTag::None,
		ir::TimelineTag::Input{funclet_id, index} => ir::TimelineTag::Operation{remote_node_id : ir::RemoteNodeId{funclet_id, node_id : index}},
		ir::TimelineTag::Operation{remote_node_id} => ir::TimelineTag::Operation{remote_node_id},
		ir::TimelineTag::Output{funclet_id, index} => ir::TimelineTag::Output{funclet_id, index},
		_ => panic!("Unimplemented")
	}
}

// Are these timeline tags equivalent?
pub fn check_timeline_tag_compatibility_interior(program : & ir::Program, source_timeline_tag : ir::TimelineTag, destination_timeline_tag : ir::TimelineTag)
{
	match (source_timeline_tag, destination_timeline_tag)
	{
		(ir::TimelineTag::None, ir::TimelineTag::None) => (),
		/*(ir::TimelineTag::Input{funclet_id : input_funclet_id, index : input_index}, ir::TimelineTag::Output{funclet_id : output_funclet_id, index : output_index}) =>
		{
			assert_eq!(input_funclet_id, output_funclet_id);
			assert_eq!(input_index, output_index);

			let destination_timeline_funclet = & program.funclets[& output_funclet_id];
			assert_eq!(destination_timeline_funclet.kind, ir::FuncletKind::Timeline);

			let node_id = match & destination_timeline_funclet.tail_edge
			{
				ir::TailEdge::Return { return_values } => return_values[output_index],
				_ => panic!("Not a unit")
			};

			if let ir::Node::Phi{index : phi_index} = & destination_timeline_funclet.nodes[node_id]
			{
				assert_eq!(* phi_index, input_index);
			}
			else
			{
				panic!("Not a phi");
			}
		}*/
		(ir::TimelineTag::Input{funclet_id, index}, ir::TimelineTag::Operation{remote_node_id}) =>
		{
			assert_eq!(remote_node_id.funclet_id, funclet_id);

			let destination_timeline_funclet = & program.funclets[& funclet_id];
			assert_eq!(destination_timeline_funclet.kind, ir::FuncletKind::Timeline);

			if let ir::Node::Phi{index : phi_index} = & destination_timeline_funclet.nodes[remote_node_id.node_id]
			{
				assert_eq!(* phi_index, index);
			}
			else
			{
				panic!("Not a phi");
			}
		}
		(ir::TimelineTag::Operation{remote_node_id}, ir::TimelineTag::Operation{remote_node_id : remote_node_id_2}) =>
		{
			assert_eq!(remote_node_id, remote_node_id_2);
		}
		(ir::TimelineTag::Operation{remote_node_id}, ir::TimelineTag::Output{funclet_id, index}) =>
		{
			assert_eq!(remote_node_id.funclet_id, funclet_id);

			let source_timeline_funclet = & program.funclets[& funclet_id];
			assert_eq!(source_timeline_funclet.kind, ir::FuncletKind::Timeline);

			match & source_timeline_funclet.tail_edge
			{
				ir::TailEdge::Return { return_values } => assert_eq!(return_values[index], remote_node_id.node_id),
				_ => panic!("Not a unit")
			}
		}
		(ir::TimelineTag::Output{funclet_id, index}, ir::TimelineTag::Output{funclet_id : funclet_id_2, index : index_2}) =>
		{
			assert_eq!(funclet_id, funclet_id_2);
			assert_eq!(index, index_2);
		}
		_ => panic!("Ill-formed: {:?} to {:?}", source_timeline_tag, destination_timeline_tag)
	}
}

// check control flow legality

pub fn check_next_timeline_tag_on_submit(program : & ir::Program, timeline_event : ir::RemoteNodeId, current_timeline_tag : ir::TimelineTag) -> ir::TimelineTag
{
	// To do: have timeline tag for both gpu and local
	let destination_timeline_funclet = & program.funclets[& timeline_event.funclet_id];
	assert_eq!(destination_timeline_funclet.kind, ir::FuncletKind::Timeline);

	let (here_place, there_place, local_past) = match & destination_timeline_funclet.nodes[timeline_event.node_id]
	{
		ir::Node::SubmissionEvent{here_place, there_place, local_past} => 
		{
			(* here_place, * there_place, * local_past)
		}
		_ => panic!("Not a submission event node")
	};

	match current_timeline_tag
	{
		ir::TimelineTag::Input{funclet_id, index} =>
		{
			assert_eq!(timeline_event.funclet_id, funclet_id);

			let destination_timeline_funclet = & program.funclets[& funclet_id];
			assert_eq!(destination_timeline_funclet.kind, ir::FuncletKind::Timeline);

			if let ir::Node::Phi{index : phi_index} = & destination_timeline_funclet.nodes[local_past]
			{
				assert_eq!(* phi_index, index);
			}
			else
			{
				panic!("Not a phi");
			}
		}
		ir::TimelineTag::Operation{remote_node_id} =>
		{
			assert_eq!(remote_node_id.funclet_id, timeline_event.funclet_id);
			assert_eq!(local_past, remote_node_id.node_id);
		}
		_ => panic!("Timeline tag must be operation or input")
	}

	ir::TimelineTag::Operation { remote_node_id : timeline_event }
}

pub fn check_next_timeline_tag_on_sync(program : & ir::Program, timeline_event : ir::RemoteNodeId, current_timeline_tag : ir::TimelineTag) -> ir::TimelineTag
{
	// To do: have timeline tag for both gpu and local
	let destination_timeline_funclet = & program.funclets[& timeline_event.funclet_id];
	assert_eq!(destination_timeline_funclet.kind, ir::FuncletKind::Timeline);

	let (here_place, there_place, local_past, remote_local_past) = match & destination_timeline_funclet.nodes[timeline_event.node_id]
	{
		ir::Node::SynchronizationEvent{here_place, there_place, local_past, remote_local_past} => 
		{
			(* here_place, * there_place, * local_past, * remote_local_past)
		}
		_ => panic!("Not a submission event node")
	};

	match current_timeline_tag
	{
		ir::TimelineTag::Input{funclet_id, index} =>
		{
			assert_eq!(timeline_event.funclet_id, funclet_id);

			let destination_timeline_funclet = & program.funclets[& funclet_id];
			assert_eq!(destination_timeline_funclet.kind, ir::FuncletKind::Timeline);

			if let ir::Node::Phi{index : phi_index} = & destination_timeline_funclet.nodes[local_past]
			{
				assert_eq!(* phi_index, index);
			}
			else
			{
				panic!("Not a phi");
			}
		}
		ir::TimelineTag::Operation{remote_node_id} =>
		{
			assert_eq!(remote_node_id.funclet_id, timeline_event.funclet_id);
			assert_eq!(local_past, remote_node_id.node_id);
		}
		_ => panic!("Timeline tag must be operation or input")
	}

	ir::TimelineTag::Operation { remote_node_id : timeline_event }
}

/*pub fn check_timeline_tag_compatibility_on_submit(program : & ir::Program, timeline_event : ir::RemoteNodeId, source_timeline_tag : ir::TimelineTag, destination_timeline_tag : ir::TimelineTag)
{
	match (source_timeline_tag, destination_timeline_tag)
	{
		(ir::TimelineTag::Operation{remote_node_id}, ir::TimelineTag::Operation{remote_node_id : remote_node_id_2}) =>
		{
			assert_eq!(remote_node_id.funclet_id, remote_node_id_2.funclet_id);
			assert_eq!(timeline_event, * remote_node_id_2);

			let destination_timeline_funclet = & program.funclets[& timeline_event.funclet_id];
			assert_eq!(destination_timeline_funclet.kind, ir::FuncletKind::Timeline);

			match & destination_timeline_funclet.nodes[timeline_event.node_id]
			{
				ir::Node::SubmissionEvent{here_place, there_place, local_past, remote_local_past} => 
				{
					assert_eq!(local_past, );
				}
				_ => panic!("Not a submission event node")
			}
		}
		_ => panic!("Ill-formed: {:?} to {:?}", source_timeline_tag, destination_timeline_tag)
	}
}*/


use crate::ir;

pub fn concretize_input_to_internal_spatial_tag(program : & ir::Program, spatial_tag : ir::SpatialTag) -> ir::SpatialTag
{
	match spatial_tag
	{
		ir::SpatialTag::None => ir::SpatialTag::None,
		ir::SpatialTag::Input{funclet_id, index} => ir::SpatialTag::Operation{remote_node_id : ir::RemoteNodeId{funclet_id, node_id : index}},
		ir::SpatialTag::Operation{remote_node_id} => ir::SpatialTag::Operation{remote_node_id},
		ir::SpatialTag::Output{funclet_id, index} => ir::SpatialTag::Output{funclet_id, index},
		_ => panic!("Unimplemented")
	}
}

pub fn check_spatial_tag_compatibility_interior(program : & ir::Program, source_tag : ir::SpatialTag, destination_tag : ir::SpatialTag)
{
	match (source_tag, destination_tag)
	{
		(ir::SpatialTag::None, ir::SpatialTag::None) => (),
		(ir::SpatialTag::Input{funclet_id, index}, ir::SpatialTag::Operation{remote_node_id}) =>
		{
			assert_eq!(remote_node_id.funclet_id, funclet_id);

			let destination_funclet = & program.funclets[& funclet_id];
			assert_eq!(destination_funclet.kind, ir::FuncletKind::Spatial);

			if let ir::Node::Phi{index : phi_index} = & destination_funclet.nodes[remote_node_id.node_id]
			{
				assert_eq!(* phi_index, index);
			}
			else
			{
				panic!("Not a phi");
			}
		}
		(ir::SpatialTag::Operation{remote_node_id}, ir::SpatialTag::Operation{remote_node_id : remote_node_id_2}) =>
		{
			assert_eq!(remote_node_id, remote_node_id_2);
		}
		(ir::SpatialTag::Operation{remote_node_id}, ir::SpatialTag::Output{funclet_id, index}) =>
		{
			assert_eq!(remote_node_id.funclet_id, funclet_id);

			let source_funclet = & program.funclets[& funclet_id];
			assert_eq!(source_funclet.kind, ir::FuncletKind::Spatial);

			match & source_funclet.tail_edge
			{
				ir::TailEdge::Return { return_values } => assert_eq!(return_values[index], remote_node_id.node_id),
				_ => panic!("Not a unit")
			}
		}
		(ir::SpatialTag::Output{funclet_id, index}, ir::SpatialTag::Output{funclet_id : funclet_id_2, index : index_2}) =>
		{
			assert_eq!(funclet_id, funclet_id_2);
			assert_eq!(index, index_2);
		}
		_ => panic!("Ill-formed: {:?} to {:?}", source_tag, destination_tag)
	}
}
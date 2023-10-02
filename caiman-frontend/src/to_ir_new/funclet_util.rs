use caiman::assembly::ast as asm;
use caiman::ir;
use super::value_funclets::ValueFunclet;

/*
    let mut commands: Vec<Option<asm::Command>> =
        nodes.into_iter().map(|c| c.map(|nn| asm::Command::Node(nn))).collect();
    commands.push(Some(asm::Command::TailEdge(tail_edge)));
*/

pub fn make_asm_funclet(
    kind: ir::FuncletKind,
    header: asm::FuncletHeader,
    named_nodes: Vec<Option<asm::NamedNode>>,
    tail_edge: asm::TailEdge,
) -> asm::Funclet
{
    let mut commands: Vec<Option<asm::Command>> =
        named_nodes.into_iter().map(|c| c.map(|nn| asm::Command::Node(nn))).collect();
    commands.push(Some(asm::Command::TailEdge(tail_edge)));
    asm::Funclet { kind, header, commands }
}

pub fn vf_node_with_name<'a>(vf: &'a ValueFunclet, name: &str) -> Option<&'a asm::NamedNode>
{
    for com_opt in vf.0.commands.iter() {
        if let Some(asm::Command::Node(nn)) = com_opt {
            if let Some(nn_name) = &nn.name {
                if nn_name.0 == name {
                    return Some(nn);
                }
            }
        }
    }
    None
}

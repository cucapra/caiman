use caiman::assembly::ast as asm;

pub fn label_node(x: &str) -> asm::NodeId { asm::NodeId(x.to_string()) }

pub fn label_call_node(result_node: &asm::NodeId) -> asm::NodeId 
{
    asm::NodeId(result_node.clone().0 + "_call")
}

//pub fn label_node(n: usize) -> asm::NodeId { asm::NodeId(format!("node{}", n)) }

pub fn label_slot(n: usize) -> String { format!("slot{}", n) }

pub fn label_event(n: usize) -> String { format!("event{}", n) }


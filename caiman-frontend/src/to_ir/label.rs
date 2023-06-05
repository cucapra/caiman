use caiman::assembly::ast as asm;

pub fn label_node(n: usize) -> asm::NodeId { asm::NodeId(format!("node{}", n)) }

pub fn label_slot(n: usize) -> String { format!("slot{}", n) }

pub fn label_event(n: usize) -> String { format!("event{}", n) }


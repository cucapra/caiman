// TODO change like, everything about this (might need to make an object that stores 
// what slot numbers are already taken)

pub fn label_node_index(n: usize) -> String { format!("node{}", n + 1) }

pub fn label_slot(n: usize) -> String { format!("slot{}", n) }


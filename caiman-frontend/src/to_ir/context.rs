use super::label;
use caiman::assembly::ast as asm;
use caiman::ir;

pub struct NodeContext
{
    commands: Vec<Option<asm::NamedNode>>,
}

impl NodeContext
{
    pub fn new() -> Self { Self { commands: Vec::new() } }

    pub fn add_node(&mut self, node_op: Option<asm::Node>)
    {
        let name = label::label_node(self.commands.len());
        self.commands.push(
            node_op.map(|node| {
                asm::NamedNode {
                    name,
                    node,
                }
            }));
    }

    pub fn into_commands(self) -> Vec<Option<asm::NamedNode>> { self.commands }
}


pub struct Context
{
    assembly_types: Vec<asm::TypeDecl>,
    slot_label_index: usize,
    event_label_index: usize,
}

impl Context
{
    pub fn new() -> Self
    {
        Context {
            assembly_types: Vec::new(),
            slot_label_index: 0,
            event_label_index: 0,
        }
    }

    pub fn add_ffi_type(&mut self, t: asm::FFIType)
    {
        let t_in_types = self.assembly_types.iter().any(|td| {
            if let asm::TypeDecl::FFI(td_t) = td
            {
                t == *td_t
            }
            else
            {
                false
            }
        });
        if !t_in_types
        {
            self.assembly_types.push(asm::TypeDecl::FFI(t));
        }
    }
    
    pub fn add_slot(
        &mut self,
        storage_type: asm::TypeId,
        queue_place: ir::Place,
        queue_stage: ir::ResourceQueueStage,
    ) -> String
    {
        let slot_str = self.label_slot();
        self.slot_label_index += 1;

        self.assembly_types.push(asm::TypeDecl::Local(asm::LocalType {
            name: slot_str.clone(),
            data: asm::LocalTypeInfo::Slot {
                storage_type,
                queue_place,
                queue_stage,
            },
        }));

        slot_str
    }

    pub fn add_event(&mut self, place: ir::Place) -> String
    {
        let event_str = self.label_event();
        self.event_label_index += 1;

        self.assembly_types.push(asm::TypeDecl::Local(asm::LocalType {
            name: event_str.clone(),
            data: asm::LocalTypeInfo::Event {
                place,
            },
        }));

        event_str
    }

    pub fn into_types(self) -> Vec<asm::TypeDecl>
    {
        self.assembly_types
    }

    fn label_slot(&self) -> String { label::label_slot(self.slot_label_index) }

    fn label_event(&self) -> String { label::label_event(self.event_label_index) }
}

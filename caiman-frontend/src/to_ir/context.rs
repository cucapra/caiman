use std::collections::HashMap;

use super::label;
use caiman::assembly_ast as asm;
use caiman::assembly_context as asm_ctx;
use caiman::ir;

pub struct Context
{
    assembly_context: asm_ctx::Context,
    assembly_types: asm::Types,
    node_label_index: usize,
    slot_label_index: usize,
    event_label_index: usize,
}

impl Context
{
    pub fn new() -> Self
    {
        Context {
            assembly_context: asm_ctx::Context::new(),
            assembly_types: Vec::new(),
            node_label_index: 0,
            slot_label_index: 0,
            event_label_index: 0,
        }
    }

    pub fn begin_local_funclet(&mut self, name: String)
    {
        self.node_label_index = 0;
        self.assembly_context.add_local_funclet(name);
    }

    pub fn end_local_funclet(&mut self)
    {
        self.node_label_index = 0;
        self.assembly_context.clear_local_funclet();
    }

    pub fn add_arg(&mut self, name: String)
    {
        self.assembly_context.add_node(name);
    }

    pub fn add_node(&mut self)
    {
        let node_str = self.label_node();
        self.assembly_context.add_node(node_str);
        self.node_label_index += 1;
    }

    pub fn add_ffi_type(&mut self, t: asm::FFIType)
    {
        self.assembly_context.add_ffi_type(t.clone());
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
    
    pub fn add_return(&mut self, ret: String)
    {
        self.assembly_context.add_return(ret);
    }

    pub fn add_slot(
        &mut self,
        typ: asm::Type,
        place: ir::Place,
        stage: ir::ResourceQueueStage,
    ) -> String
    {
        let slot_str = self.label_slot();
        self.assembly_context.add_local_type(slot_str.clone());
        self.slot_label_index += 1;

        let mut data: asm::UncheckedDict = HashMap::new();
        let mut data_insert =
            |s: &str, v| data.insert(asm::Value::ID(s.to_string()), asm::DictValue::Raw(v));
        data_insert("type", asm::Value::Type(typ));
        data_insert("place", asm::Value::Place(place));
        data_insert("stage", asm::Value::Stage(stage));
        self.assembly_types.push(asm::TypeDecl::Local(asm::LocalType {
            type_kind: asm::TypeKind::Slot,
            name: slot_str.clone(),
            data,
        }));

        slot_str
    }

    pub fn add_event(&mut self, place: ir::Place) -> String
    {
        let event_str = self.label_event();
        self.assembly_context.add_local_type(event_str.clone());
        self.event_label_index += 1;

        let mut data: asm::UncheckedDict = HashMap::new();
        let mut data_insert =
            |s: &str, v| data.insert(asm::Value::ID(s.to_string()), asm::DictValue::Raw(v));
        data_insert("place", asm::Value::Place(place));
        self.assembly_types.push(asm::TypeDecl::Local(asm::LocalType {
            type_kind: asm::TypeKind::Event,
            name: event_str.clone(),
            data,
        }));

        event_str
    }

    pub fn into_context_and_types(self) -> (asm_ctx::Context, asm::Types)
    {
        (self.assembly_context, self.assembly_types)
    }

    fn label_node(&self) -> String { label::label_node(self.node_label_index) }

    fn label_slot(&self) -> String { label::label_slot(self.slot_label_index) }

    fn label_event(&self) -> String { label::label_event(self.event_label_index) }
}

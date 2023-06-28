use super::funclet_util;
use super::function_classes::FunctionClassContext;
use super::label;
use super::value_funclets::ValueFunclet;
use crate::syntax::ast;
use caiman::assembly::ast as asm;
use caiman::ir;

pub fn convert_value_type(vt: ast::value::Type) -> asm::TypeId
{
    asm::TypeId::FFI(match vt {
        ast::value::Type::Bool => asm::FFIType::U64,
        ast::value::Type::Num(ast::value::NumberType::I32) => asm::FFIType::I32,
        ast::value::Type::Num(ast::value::NumberType::I64) => asm::FFIType::I64,
    })
}

pub fn type_of_asm_node(
    node: &asm::Node,
    funclet_being_scheduled: &ValueFunclet,
    function_class_ctx: &FunctionClassContext,
) -> Option<asm::TypeId>
{
    match node {
        asm::Node::Constant { value: _, type_id: Some(t) } => Some(t.clone()),
        asm::Node::CallValueFunction { function_id: Some(f_id), .. } => {
            match function_class_ctx.type_of(&f_id.0) {
                None => panic!("Cannot find external function type for {}", f_id),
                Some((_inputs, outputs)) => {
                    // TODO multiple outputs???
                    Some(outputs[0].clone())
                },
            }
        },
        asm::Node::ExtractResult { node_id, index: _ } => {
            node_id.as_ref().and_then(|node_id| {
                match funclet_util::vf_node_with_name(funclet_being_scheduled, &node_id.0) {
                    Some(asm::NamedNode { name: _, node }) => {
                        type_of_asm_node(node, funclet_being_scheduled, function_class_ctx)
                    },
                    _ => None,
                }
            })
        },
        _ => panic!("Typing of node {:?} is either unknowable or unimplemented", node),
    }
}

pub struct TypingContext
{
    assembly_types: Vec<asm::TypeDecl>,
    slot_label_index: usize,
    event_label_index: usize,
}

impl TypingContext
{
    pub fn new() -> Self
    {
        TypingContext { assembly_types: Vec::new(), slot_label_index: 0, event_label_index: 0 }
    }

    pub fn add_value_funclet_types(&mut self, vfs: &Vec<ValueFunclet>)
    {
        for vf in vfs {
            for arg in vf.0.header.args.iter().chain(vf.0.header.ret.iter()) {
                match &arg.typ {
                    asm::TypeId::FFI(ffi_type) => self.add_ffi_type(ffi_type.clone()),
                    _ => panic!("Non-FFI type is somehow in a value funclet"),
                }
            }
        }
    }

    fn add_ffi_type(&mut self, t: asm::FFIType)
    {
        let t_in_types = self.assembly_types.iter().any(|td| {
            if let asm::TypeDecl::FFI(td_t) = td {
                t == *td_t
            } else {
                false
            }
        });
        if !t_in_types {
            self.assembly_types.push(asm::TypeDecl::FFI(t));
        }
    }

    fn search_assembly_types(&self, pred: &dyn Fn(&asm::LocalTypeInfo) -> bool) -> Option<String>
    {
        for td in self.assembly_types.iter() {
            match td {
                asm::TypeDecl::Local(local_type) => {
                    if pred(&local_type.data) {
                        return Some(local_type.name.clone());
                    }
                },
                _ => (),
            }
        }
        None
    }

    pub fn add_slot(
        &mut self,
        storage_type: asm::TypeId,
        queue_place: ir::Place,
        queue_stage: ir::ResourceQueueStage,
    ) -> String
    {
        let is_same_slot = |lti: &asm::LocalTypeInfo| match lti {
            asm::LocalTypeInfo::Slot { storage_type: st, queue_stage: qs, queue_place: qp } => {
                storage_type == *st && queue_stage == *qs && queue_place == *qp
            },
            _ => false,
        };
        if let Some(s) = self.search_assembly_types(&is_same_slot) {
            return s;
        }

        let slot_str = self.label_slot();
        self.slot_label_index += 1;

        self.assembly_types.push(asm::TypeDecl::Local(asm::LocalType {
            name: slot_str.clone(),
            data: asm::LocalTypeInfo::Slot { storage_type, queue_place, queue_stage },
        }));

        slot_str
    }

    pub fn convert_and_add_scheduling_type(
        &mut self,
        st: ast::scheduling::Type,
        value_funclet: &ValueFunclet,
        function_class_ctx: &FunctionClassContext,
    ) -> Result<String, String>
    {
        match st {
            ast::scheduling::Type::Slot(x) => {
                let x_node = funclet_util::vf_node_with_name(value_funclet, &x)
                    .ok_or_else(|| format!("Unknown variable {}", x))?;
                match type_of_asm_node(&x_node.node, value_funclet, function_class_ctx) {
                    Some(x_type) => {
                        Ok(self.add_slot(x_type, ir::Place::Local, ir::ResourceQueueStage::Ready))
                    },
                    None => Err(format!("FFI type for slot {} not given", x)),
                }
            },
        }
    }

    pub fn add_event(&mut self, place: ir::Place) -> String
    {
        let is_same_event = |lti: &asm::LocalTypeInfo| match lti {
            asm::LocalTypeInfo::Event { place: p } => place == *p,
            _ => false,
        };
        if let Some(s) = self.search_assembly_types(&is_same_event) {
            return s;
        }

        let event_str = self.label_event();
        self.event_label_index += 1;

        self.assembly_types.push(asm::TypeDecl::Local(asm::LocalType {
            name: event_str.clone(),
            data: asm::LocalTypeInfo::Event { place },
        }));

        event_str
    }

    pub fn convert_and_add_timeline_type(&mut self, timeline_type: ast::timeline::Type) -> String
    {
        match timeline_type {
            ast::timeline::Type::Event => self.add_event(ir::Place::Local),
        }
    }

    pub fn into_types(self) -> Vec<asm::TypeDecl> { self.assembly_types }

    fn label_slot(&self) -> String { label::label_slot(self.slot_label_index) }

    fn label_event(&self) -> String { label::label_event(self.event_label_index) }
}

/* TODO Remove lots of this! Since we are compiling to caiman assembly and not the IR we can
 * avoid doing lots here */
use super::index::Index;
use super::vil;
use crate::value_language::typing as vl_typing;
use caiman::ir;
use caiman::assembly_ast as asm;
use caiman::stable_vec::StableVec;

#[derive(Eq, PartialEq, Hash)]
pub enum IRType
{
    Native(vl_typing::Type),
    Slot(vl_typing::Type, ir::ResourceQueueStage, ir::Place),
}

pub struct IRTypesIndex
{
    ffi_index: Index<vl_typing::Type>,
    ir_types_index: Index<IRType>,
}

impl IRTypesIndex
{
    pub fn from_vil(program: &vil::Program) -> Self
    {
        let mut ffi_index: Index<vl_typing::Type> = Index::new();
        let mut ir_types_index: Index<IRType> = Index::new();
        for stmt in program.stmts.iter()
        {
            ffi_index.insert(stmt.expr_type);
            ir_types_index.insert(IRType::Native(stmt.expr_type));
            let slot =
                IRType::Slot(stmt.expr_type, ir::ResourceQueueStage::Ready, ir::Place::Local);
            ir_types_index.insert(slot);
        }
        IRTypesIndex { ffi_index, ir_types_index }
    }

    /*pub fn into_usable_for_caiman(self) -> (StableVec<ir::ffi::Type>, StableVec<ir::Type>)
    {
        let type_sv =
            self.ir_types_index.map_into_stable_vec(&|t| convert_ir_type(t, &self.ffi_index));
        let ffi_sv = self.ffi_index.map_into_stable_vec(&convert_ffi_type);
        (ffi_sv, type_sv)
    }*/

    pub fn get_native_type(&self, t: &vl_typing::Type) -> usize
    {
        todo!()
    }
}

pub fn vl_type_to_asm_type(t: &vl_typing::Type) -> asm::Type
{
    use vl_typing::Type::*;
    let ffi = match t 
    {
        I32 => asm::FFIType::I32,
        Bool => asm::FFIType::U64,
    };
    asm::Type::FFI(ffi)
}

fn convert_ffi_type(t: vl_typing::Type) -> ir::ffi::Type
{
    // XXX: is this redundant given that VIL already turns bools into U8, for example?
    use ir::ffi::Type as F;
    use vl_typing::Type as V;
    match t
    {
        V::I32 => F::I32,
        V::Bool => F::U8,
    }
}

fn convert_ir_type(t: IRType, ffi_index: &Index<vl_typing::Type>) -> ir::Type
{
    match t
    {
        IRType::Native(t) =>
        {
            ir::Type::NativeValue { storage_type: ir::ffi::TypeId(ffi_index.get(&t).unwrap()) }
        },
        IRType::Slot(t, queue_stage, queue_place) => ir::Type::Slot {
            storage_type: ir::ffi::TypeId(ffi_index.get(&t).unwrap()),
            queue_stage,
            queue_place,
        },
    }
}

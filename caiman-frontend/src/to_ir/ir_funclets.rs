use caiman::assembly_ast as asm;
use caiman::ir;

pub struct InnerFunclet
{
    pub header : asm::FuncletHeader,
    pub commands : Vec<Option<asm::Node>>,
    pub tail_edge : Option<asm::TailEdge>,
}

pub trait Funclet
{
    fn inner(self) -> InnerFunclet;
    fn kind(&self) -> ir::FuncletKind;
}

pub struct ValueFunclet
{
    //pub involved_variables: Index<String>,
    pub inner_funclet: InnerFunclet,
}

impl Funclet for ValueFunclet
{
    fn inner(self) -> InnerFunclet { self.inner_funclet }
    fn kind(&self) -> ir::FuncletKind { ir::FuncletKind::Value }
}

pub struct ScheduleExplicitFunclet
{
    pub inner_funclet: InnerFunclet,
}

impl Funclet for ScheduleExplicitFunclet
{
    fn inner(self) -> InnerFunclet { self.inner_funclet }
    fn kind(&self) -> ir::FuncletKind { ir::FuncletKind::ScheduleExplicit }
}

pub fn make_asm_funclet<T: Funclet>(f: T) -> asm::FuncletDef
{
    let kind = f.kind();
    let inner = f.inner();
    asm::FuncletDef::Local(asm::Funclet {
        kind,
        header: inner.header,
        commands: inner.commands,
        tail_edge: inner.tail_edge,
    })
}

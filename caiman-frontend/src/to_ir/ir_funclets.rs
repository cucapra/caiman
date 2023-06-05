use caiman::assembly::ast as asm;
use caiman::ir;

pub struct InnerFunclet
{
    pub header : asm::FuncletHeader,
    pub commands : Vec<Option<asm::NamedNode>>,
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

pub fn make_asm_funclet<T: Funclet>(f: T) -> asm::Funclet
{
    let kind = f.kind();
    let inner = f.inner();
    let mut commands : Vec<Option<asm::Command>> = inner.commands.into_iter().map(|c| c.map(|nn| asm::Command::Node(nn))).collect();
    match inner.tail_edge {
        Some(te) => commands.push(Some(asm::Command::TailEdge(te))),
        // XXX do we push none always? The whole tail edges thing may need changing
        None => commands.push(None),
    };
    asm::Funclet {
        kind,
        header: inner.header,
        commands,
    }
}

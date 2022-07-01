// Ast

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type
{
    F32,
    F64,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    // TODO others
}

pub type FuncType = (Vec<Type>, Vec<Type>);

#[derive(Debug, Clone)]
pub enum NodeType
{
    Phi(usize),
    Extract(String, usize),

    // For constants, parsing the value as 
    // a String so that we can use the correct
    // to-string function depending on the type
    Constant(String, Type),

    // Calling can either be ValueFunction or CPU
    // (I have given them the same syntax for now)
    Call(String, Vec<String>),
    // GPU call differs only because it requires
    // dimensions info
    // Dimensions info is FIRST
    GPUCall(String, Vec<String>, Vec<String>),

    // TODO others 
}

pub type Node = (String, NodeType);

#[derive(Debug, Clone)]
pub enum FuncletTail
{
    Return(Vec<String>),
    Yield(Vec<String>, Vec<String>, Vec<String>),
}

#[derive(Debug, Clone)]
pub struct ResourceBinding
{
    pub group : usize,
    pub binding : usize,
    pub input : Option<usize>,
    pub output : Option<usize>
}

#[derive(Debug, Clone)]
pub enum Declaration
{
    // For now: funclet kind will be a bool (true is inline)
    //   but this is of course subject to change; I will just
    //   make every non-inline funclet MixedImplicit
    Funclet(bool, String, FuncType, Vec<Node>, FuncletTail),
    CPU(String, FuncType),
    GPU(String, String, FuncType, Vec<ResourceBinding>, Option<String>),
    ValueFunction(String, Option<String>, FuncType),
    Pipeline(String, String),
}

pub type Program = Vec<Declaration>;



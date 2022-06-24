// Ast

#[derive(Debug)]
pub enum Type
{
    I32,
    // TODO others
}

pub type FuncType = (Vec<Type>, Vec<Type>);

#[derive(Debug)]
pub enum NodeType
{
    Phi(usize),
    // TODO others 
}

pub type Node = (String, NodeType);

#[derive(Debug)]
pub enum FuncletTail
{
    Return(Vec<String>),
    Yield(Vec<String>, Vec<String>, Vec<String>),
}

#[derive(Debug)]
pub enum Declaration
{
    // For now: funclet kind will be a bool (true is inline)
    //   but this is of course subject to change; I will just
    //   make every non-inline funclet MixedImplicit
    Funclet(bool, String, FuncType, Vec<Node>, FuncletTail),
    CPU(String, FuncType),
    // TODO: GPU! The reason I am holding off on it is because of the
    // part where you just dump WGSL code into it
    ValueFunction(String, Option<String>, FuncType),
    Pipeline(String, String),
}

//pub type Program = Vec<Declaration>;


// AST for the combo value-scheduling language

use crate::error::Info;

pub type Var = String;

mod value
{
    use super::{Info, Var};

    #[derive(Clone, Debug)]
    pub enum NumberType
    {
        I32,
        I64,
    }

    #[derive(Clone, Debug)]
    pub enum Type
    {
        Num(NumberType),
        Bool,
    }

    #[derive(Clone, Debug)]
    pub enum ExprKind
    {
        Var(Var),
        Num(String, NumberType),
        Bool(bool),
        // TODO: many more :D
    }

    pub type Expr = (Info, ExprKind);

    #[derive(Clone, Debug)]
    pub enum StmtKind
    {
        Let(Var, Expr),
        Returns(Expr),
    }

    pub type Stmt = (Info, StmtKind);
}

mod scheduling
{
    use super::{Info, Var};

    #[derive(Clone, Debug)]
    pub enum Type 
    {
        // Implicitly local and ready!!!
        Slot(Var),
    }

    #[derive(Clone, Debug)]
    pub enum FullSchedulable {
        Primitive,
    }

    #[derive(Clone, Debug)]
    pub struct ScheduledExpr {
        info: Info,
        label: Var,
        // TODO sub exprs vec in here ?? (see old scheduling AST)
        full: FullSchedulable,
    }

    #[derive(Clone, Debug)]
    pub enum StmtKind {
        Let(Var, Box<Stmt>),
        Return(Var),
    }

    pub type Stmt = (Info, StmtKind);

    #[derive(Clone, Debug)]
    pub struct SchedulingFunclet {
        info: Info,
        name: String,
        input: Vec<Type>,
        output: Type,
        timeline_funclet: Option<String>,
        spatial_funclet: Option<String>,
        // TODO: tags?????
        statements: Vec<Stmt>,
    }
}

mod timeline {
    use super::{Info, Var};

    #[derive(Clone, Debug)]
    pub enum Type 
    {
        // Implicitly Local
        Event,
    }

    #[derive(Clone, Debug)]
    pub enum StmtKind {
        Return(Var),
    }

    pub type Stmt = (Info, StmtKind);
}

#[derive(Clone, Debug)]
pub enum DeclKind
{
    ValueFunclet
    {
        name: String,
        input: Vec<value::Type>,
        output: value::Type,
        statements: Vec<value::Stmt>,
    },
    FunctionClass(Vec<String>),
    SchedulingImpl
    {
        value_funclet_name: String,
        scheduling_funclets: Vec<scheduling::SchedulingFunclet>,
    },
    TimelineFunclet {
        name: String,
        input: Vec<timeline::Type>,
        output: timeline::Type,
        statements: Vec<timeline::Stmt>,
    },
    // TODO spatial funclet stuff
    SpatialFunclet,
    Pipeline
    {
        name: String,
        funclet_to_run: String,
    },
}

pub type Decl = (Info, DeclKind);

pub type Program = Vec<Decl>;

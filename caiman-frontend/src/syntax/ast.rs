// AST for the combo value-scheduling language

use crate::error::Info;

pub type Var = String;

pub type Arg<T> = (String, T);

pub mod value
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
        // TODO function literal allowed instead of having to use a var?
        App(String, Vec<Expr>),
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

pub mod scheduling
{
    use super::{Info, Var, Arg};

    #[derive(Clone, Debug)]
    pub enum Type
    {
        // Implicitly local and ready!!!
        Slot(Var),
    }

    #[derive(Clone, Debug)]
    pub enum FullSchedulable
    {
        Primitive,
    }

    #[derive(Clone, Debug)]
    pub struct ScheduledExpr
    {
        pub info: Info,
        pub value_var: Var,
        // TODO sub exprs vec in here ?? (see old scheduling AST)
        pub full: FullSchedulable,
    }

    #[derive(Clone, Debug)]
    pub enum StmtKind
    {
        Let(Var, ScheduledExpr),
        // Should we rly return var??? or just like the expr ??? unsure
        Return(Var),
    }

    pub type Stmt = (Info, StmtKind);

    #[derive(Clone, Debug)]
    pub struct SchedulingFunclet
    {
        pub info: Info,
        pub name: String,
        pub input: Vec<Arg<Type>>,
        pub output: Type,
        pub timeline_funclet: Option<String>,
        pub spatial_funclet: Option<String>,
        // TODO: tags?????
        pub statements: Vec<Stmt>,
    }
}

pub mod timeline
{
    use super::{Info, Var};

    #[derive(Clone, Debug)]
    pub enum Type
    {
        // Implicitly Local
        Event,
    }

    #[derive(Clone, Debug)]
    pub enum StmtKind
    {
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
        input: Vec<Arg<value::Type>>,
        output: (Option<String>, value::Type),
        statements: Vec<value::Stmt>,
    },
    FunctionClass
    {
        name: String,
        functions: Vec<String>,
    },
    SchedulingImpl
    {
        value_funclet_name: String,
        scheduling_funclets: Vec<scheduling::SchedulingFunclet>,
    },
    TimelineFunclet
    {
        name: String,
        input: Vec<Arg<timeline::Type>>,
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
    ExternCPU
    {
        name: String,
        input: Vec<value::Type>,
        output: value::Type,
    },
}

pub type Decl = (Info, DeclKind);

pub type Program = Vec<Decl>;

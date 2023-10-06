// AST for the combo value-scheduling language

use crate::error::Info;

pub type Var = String;

pub type Arg<T> = (String, T);

pub mod value
{
    use super::{Info, Var};

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum NumberType
    {
        I32,
        I64,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum Type
    {
        Num(NumberType),
        Bool,
    }

    #[derive(Clone, Debug)]
    pub enum Binop
    {
        Plus,
    }

    #[derive(Clone, Debug)]
    pub enum ExprKind
    {
        Var(Var),
        Num(String, NumberType),
        Bool(bool),
        // TODO function literal allowed instead of having to use a var?
        App(String, Vec<Expr>),
        Binop(Binop, Box<Expr>, Box<Expr>),
        // TODO: many more :D
    }

    impl Default for ExprKind
    {
        fn default() -> Self { Self::Bool(false) }
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
    use super::{Arg, Info, Var};

    // Literally just option but don't want to get them confused
    #[derive(Clone, Debug)]
    pub enum Hole<T> 
    {
        Filled(T),
        Vacant,
    }

    impl<T> Hole<T> {
        pub fn to_option_move(self) -> Option<T>
        {
            match self {
                Hole::Filled(t) => Some(t),
                Hole::Vacant => None,
            }
        }

        pub fn to_option(&self) -> Option<&T>
        {
            match self {
                Hole::Filled(t) => Some(t),
                Hole::Vacant => None,
            }
        }

        pub fn from_option(o: Option<T>) -> Self
        {
            match o {
                None => Hole::Vacant,
                Some(t) => Hole::Filled(t),
            }
        }

        pub fn map<U, F>(self, f: F) -> Hole<U>
            where F: Fn(T) -> U
        {
            if let Hole::Filled(t) = self {
                Hole::Filled(f(t))
            } else { Hole::Vacant }
        }

        /*pub fn apply_option_method<U, F>(&self, f: F) -> Hole<U>
            where F: Fn(Option<T>) -> Option<U>
        {
            Hole::from_option(f(self.clone().to_option_move()))
        }*/
    }

    pub fn fill<T>(t: T) -> Hole<T>
    {
        Hole::Filled(t)
    }

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
        // XXX do we want the programmer to have to specify the vars used even though it can
        // likely be inferred??? Will keep it this way for now. In any case I imagine Dietrich
        // would want me to keep it this way and then they can infer it if the programmer uses
        // the currently-unimplemented ? expression
        Call(Hole<Var>, Vec<Hole<Var>>),
        CallExternal(Hole<Var>, Vec<Hole<Var>>),
    }

    #[derive(Clone, Debug)]
    pub enum ExprKind
    {
        Simple
        {
            value_var: Var,
            // TODO sub exprs vec in here ?? (see old scheduling AST)
            full: FullSchedulable,
        },
    }

    pub type Expr = (Info, Hole<ExprKind>);

    #[derive(Clone, Debug)]
    pub enum StmtKind
    {
        Let(Var, Expr),
        // Should we rly return var??? or just like the expr??? unsure
        Return(Var),
    }

    pub type Stmt = (Info, Hole<StmtKind>);

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

pub mod spatial
{
    use super::{Info, Var};

    #[derive(Clone, Debug)]
    pub enum Type
    {
        // Implicitly Local
        BufferSpace,
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
    SpatialFunclet
    {
        name: String,
        input: Vec<Arg<spatial::Type>>,
        output: spatial::Type,
        statements: Vec<spatial::Stmt>,
    },
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

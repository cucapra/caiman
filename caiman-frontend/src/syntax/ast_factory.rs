use super::ast::*;
use crate::error::Info;

macro_rules! factory {
    ($rt:ty, $f:ident ( $($x:ident : $t:ty),* ) => $e:expr) => {
        pub fn $f (&self, l : usize, $($x : $t,)* r : usize) -> $rt {
            (self.info(l, r), $e)
        }
    }
}

pub struct ASTFactory
{
    line_ending_byte_offsets: Vec<usize>,
}

impl ASTFactory
{
    pub fn new(_filename: &str, s: &str) -> Self
    {
        Self {
            line_ending_byte_offsets: s
                .as_bytes()
                .iter()
                .enumerate()
                .filter_map(|(idx, b)| if *b == b'\n' { Some(idx) } else { None })
                .collect(),
        }
    }

    pub fn line_and_column(&self, u: usize) -> (usize, usize)
    {
        if let Some(b) = self.line_ending_byte_offsets.last() {
            if u > *b {
                panic!("Byte offset too big: {}", u);
            }
        }
        self.line_ending_byte_offsets.iter().enumerate().map(|(l, c)| (l + 1, c)).fold(
            (1, u), // Case where offset is on line one
            |curr, (l, c)| if u > *c { (l + 1, u - c) } else { curr },
        )
    }

    fn info(&self, l: usize, r: usize) -> Info
    {
        Info { location: (self.line_and_column(l), self.line_and_column(r)) }
    }


    // VALUE FUNCLET
    factory!(
        Decl, 
        value_funclet(
            name: String, 
            input: Vec<Arg<value::Type>>, 
            output: (Option<String>, value::Type),
            statements: Vec<value::Stmt>)
        => DeclKind::ValueFunclet { name, input, output, statements }
    );

    factory!(value::Stmt, value_let(x: String, e: value::Expr) => value::StmtKind::Let(x, e));
    factory!(value::Stmt, value_returns(e: value::Expr) => value::StmtKind::Returns(e));

    factory!(value::Expr, value_var(x: String) => value::ExprKind::Var(x));
    factory!(value::Expr, value_bool(b: bool) => value::ExprKind::Bool(b));
    factory!(value::Expr, 
        value_number(n: (String, value::NumberType)) => value::ExprKind::Num(n.0, n.1));
    factory!(value::Expr, value_app(f: String, es: Vec<value::Expr>) 
        => value::ExprKind::App(f, es));
    factory!(value::Expr, value_bop(b: value::Binop, e1: value::Expr, e2: value::Expr)
        => value::ExprKind::Binop(b, Box::new(e1), Box::new(e2)));

    // FUNCTION CLASS
    factory!(Decl, function_class(name: String, functions: Vec<String>) 
        => DeclKind::FunctionClass { name, functions });

    // SCHEDULING
    factory!(
        Decl, 
        schedule_block(
            value_funclet_name: String, 
            scheduling_funclets: Vec<scheduling::SchedulingFunclet>)
        => DeclKind::SchedulingImpl { value_funclet_name, scheduling_funclets }
    );

    pub fn scheduling_funclet(
        &self, 
        l : usize, 
        name: String,
        input: Vec<Arg<scheduling::Type>>,
        output: scheduling::Type,
        timespace: Option<(Option<String>, Option<String>)>,
        statements: Vec<scheduling::Stmt>,
        r : usize,
    ) -> scheduling::SchedulingFunclet 
    {
        let (timeline_funclet, spatial_funclet) = match timespace {
            None => (None, None),
            Some(p) => p,
        };
        scheduling::SchedulingFunclet {
            info: self.info(l, r),
            name,
            input,
            output,
            timeline_funclet, 
            spatial_funclet,
            statements,
        }
    }

    factory!(scheduling::Stmt, 
        sch_let(x: String, se: scheduling::ScheduledExpr) => scheduling::StmtKind::Let(x, se));
    factory!(scheduling::Stmt, 
        sch_return(x: String) => scheduling::StmtKind::Return(x));

    pub fn sch_expr(
        &self,
        l : usize, 
        value_var: String,
        full: scheduling::FullSchedulable,
        r : usize,
    ) -> scheduling::ScheduledExpr
    {
        scheduling::ScheduledExpr 
        {
            info: self.info(l, r),
            value_var,
            full,
        }
    }

    // TIMELINE
    factory!(
        Decl, 
        timeline_funclet(
            name: String, 
            input: Vec<Arg<timeline::Type>>, 
            output: timeline::Type, 
            statements: Vec<timeline::Stmt>) 
        => DeclKind::TimelineFunclet { name, input, output, statements });

    factory!(timeline::Stmt, time_return(x: String) => timeline::StmtKind::Return(x));

    // PIPELINE
    factory!(Decl, pipeline(name: String, funclet_to_run: String) => 
        DeclKind::Pipeline { name, funclet_to_run });

    // EXTERN CPU
    factory!(Decl, extern_cpu(name: String, input: Vec<value::Type>, output: value::Type)
        => DeclKind::ExternCPU { name, input, output });
}

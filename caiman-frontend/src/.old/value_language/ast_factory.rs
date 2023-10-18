// The idea behind using this module is to 1. Easily convert
// LALRpop's less-than-ideal byte offset data into line and
// column number, and 2. Keep the parser code clean by
// using LALRpop's <> syntax. It is not a very elegant or
// visually-appealing file.

// Credit to the following for the code that
// calculates line and column number from byte offset:
// https://github.com/sampsyo/bril/blob/main/bril-rs/bril2json/src/lib.rs

use crate::error::Info;
use crate::spec::nodes::FunctionalExprNodeKind;
use crate::value_language::ast::*;
use crate::value_language::typing::Type;

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
                .filter_map(|(idx, b)| if *b == b'\n' { Some(idx) } else { None } )
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

    factory!(ParsedExpr, binop(b: Binop, e1: ParsedExpr, e2: ParsedExpr) 
        => ExprKind::Binop(b, Box::new(e1), Box::new(e2)));

    factory!(ParsedExpr, unop(u: Unop, e: ParsedExpr) => ExprKind::Unop(u, Box::new(e)));

    factory!(ParsedExpr, if_expr(e1: ParsedExpr, e2: ParsedExpr, e3: ParsedExpr)
         => ExprKind::If(Box::new(e1), Box::new(e2), Box::new(e3)));

    factory!(ParsedExpr, ir_node_expr(node: FunctionalExprNodeKind, args: Vec<ParsedExpr>)
        => ExprKind::IRNode(node, args));

    factory!(ParsedExpr, num(n: String) => ExprKind::Num(n));

    factory!(ParsedExpr, var(i: String) => ExprKind::Var(i));

    factory!(ParsedExpr, unit() => ExprKind::Unit);

    factory!(ParsedExpr, bool_expr(b: bool) => ExprKind::Bool(b));

    factory!(ParsedExpr, tuple(es: Vec<ParsedExpr>) => ExprKind::Tuple(es));

    factory!(ParsedExpr, ecall(name: String, es: Vec<ParsedExpr>) => ExprKind::Call(name, es));

    factory!(ParsedExpr, labeled(e: ParsedExpr, label: String) => 
        ExprKind::Labeled(label, Box::new(e)));

    factory!(ParsedStmt, let_stmt(vwt: VarWithType, e: ParsedExpr) => StmtKind::Let(vwt, e));

    factory!(ParsedStmt, let_function(
        f: String,
        params: Vec<VarWithType>,
        ret: Type,
        v: Vec<ParsedStmt>,
        ret_value: ParsedExpr) => StmtKind::LetFunction(f, params, ret, v, ret_value));

    //factory!(ParsedStmt, if_stmt(e: ParsedExpr, v: Vec<ParsedStmt>) => StmtKind::If(e, v));

    //factory!(ParsedStmt, while_stmt(e: ParsedExpr, v: Vec<ParsedStmt>) => StmtKind::While(e, v));

    //factory!(ParsedStmt, print(e: ParsedExpr) => StmtKind::Print(e));

    //factory!(ParsedStmt, assign(x: String, e: ParsedExpr) => StmtKind::Assign(x, e));

    //factory!(ParsedStmt, ccall(name: String, es: Vec<ParsedExpr>) => StmtKind::Call(name, es));
}

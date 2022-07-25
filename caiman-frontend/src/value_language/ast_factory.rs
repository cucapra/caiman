// The idea behind using this module is to 1. Easily convert
// LALRpop's less-than-optimal byte offset data into line and
// column number, and 2. Keep the parser code clean by
// using LALRpop's <> syntax. It is not a very elegant or
// visually-appealing file. 

// Credit to the following for the code that 
// calculates line and column number from byte offset:
// https://github.com/sampsyo/bril/blob/main/bril-rs/bril2json/src/lib.rs

use crate::value_language::ast::*;

pub struct ASTFactory 
{ 
    line_ending_byte_offsets: Vec<usize>,
}

impl ASTFactory
{
    
    pub fn new(s: &str) -> Self { 
        Self {
            line_ending_byte_offsets: s
                .as_bytes()
                .iter()
                .enumerate()
                .filter_map(
                    |(idx, b)| if *b == b'\n' { Some(idx) } else { None }
                )
                .collect()
        } 
    }

    pub fn line_and_column(&self, u: usize) -> (usize, usize)
    {
        if let Some(b) = self.line_ending_byte_offsets.last() 
        {
            if u > *b { panic!("Byte offset too big: {}", u); }
        }
        self.line_ending_byte_offsets
            .iter()
            .enumerate()
            .map(|(l, c)| (l + 1, c))
            .fold(
                (1, u), // Case where offset is on line one
                |curr, (l, c)| if u > *c { (l + 1, u - c) } else { curr },
            )
    }

    fn info(&self, l: usize, r: usize) -> Info
    {
        (self.line_and_column(l), self.line_and_column(r))
    }

    pub fn binop(
        &self,
        l: usize,
        e1: ParsedExpr, 
        b: Binop, 
        e2: ParsedExpr,
        r: usize,
    ) -> ParsedExpr
    {
        (self.info(l, r), ExprKind::Binop(b, Box::new(e1), Box::new(e2)))
    }

    pub fn unop(
        &self,
        l: usize,
        u: Unop,
        e: ParsedExpr, 
        r: usize,
    ) -> ParsedExpr
    {
        (self.info(l, r), ExprKind::Unop(u, Box::new(e)))
    }

    pub fn num(&self, l: usize, n: String, r: usize) -> ParsedExpr
    {
        (self.info(l, r), ExprKind::Num(n))
    }

    pub fn var(&self, l: usize, i: String, r: usize) -> ParsedExpr
    {
        (self.info(l, r), ExprKind::Var(i))
    }
    
    pub fn bool_expr(&self, l: usize, b: bool, r: usize) -> ParsedExpr
    {
        (self.info(l, r), ExprKind::Bool(b))
    }

    pub fn input(&self, l: usize, r: usize) -> ParsedExpr
    {
        (self.info(l, r), ExprKind::Input())
    }

    pub fn ecall(
        &self, 
        l: usize, 
        name: String,
        es: Vec<ParsedExpr>,
        r: usize,
    ) -> ParsedExpr
    {
        (self.info(l, r), ExprKind::Call(name, es))
    }

    pub fn labeled(
        &self, 
        l: usize, 
        label: String,
        e: ParsedExpr,
        r: usize,
    ) -> ParsedExpr
    {
        (self.info(l, r), ExprKind::Labeled(label, Box::new(e)))
    }

    pub fn if_stmt(
        &self, 
        l: usize, 
        e: ParsedExpr, 
        v: Vec<ParsedStmt>, 
        r: usize,
    ) -> ParsedStmt
    {
        (self.info(l, r), StmtKind::If(e, v))
    }

    pub fn while_stmt(
        &self, 
        l: usize, 
        e: ParsedExpr, 
        v: Vec<ParsedStmt>, 
        r: usize,
    ) -> ParsedStmt
    {
        (self.info(l, r), StmtKind::While(e, v))
    }

    pub fn print(&self, l: usize, e: ParsedExpr, r: usize) -> ParsedStmt
    {
        (self.info(l, r), StmtKind::Print(e))
    }

    pub fn let_stmt(
        &self,
        l: usize,
        m: bool,
        x: String,
        e: ParsedExpr,
        r: usize,
    ) -> ParsedStmt
    {
        (self.info(l, r), StmtKind::Let(m, x, e))
    }

    pub fn assign(
        &self,
        l: usize,
        x: String,
        e: ParsedExpr,
        r: usize,
    ) -> ParsedStmt
    {
        (self.info(l, r), StmtKind::Assign(x, e))
    }

    pub fn function(
        &self,
        l: usize,
        f: String,
        params: Vec<String>,
        v: Vec<ParsedStmt>,
        r: usize,
    ) -> ParsedStmt
    {
        (self.info(l, r), StmtKind::Function(f, params, v))
    }

    pub fn ccall(
        &self, 
        l: usize, 
        name: String,
        es: Vec<ParsedExpr>,
        r: usize,
    ) -> ParsedStmt
    {
        (self.info(l, r), StmtKind::Call(name, es))
    }

    pub fn return_stmt(&self, l: usize, e: ParsedExpr, r: usize) -> ParsedStmt
    {
        (self.info(l, r), StmtKind::Return(e))
    }
}

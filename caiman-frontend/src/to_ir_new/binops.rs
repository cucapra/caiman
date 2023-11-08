use std::collections::HashMap;

use super::{function_classes::FunctionClassContext, typing};
use crate::parse::ast;
use caiman::assembly::ast as asm;

struct ExternalBinop {
    input1: ast::value::Type,
    input2: ast::value::Type,
    output: ast::value::Type,
    binop_symbol: ast::value::Binop,
}

impl ExternalBinop {
    fn name(&self) -> String {
        let type_name = |t: &ast::value::Type| match t {
            ast::value::Type::Num(nt) => match nt {
                ast::value::NumberType::I32 => "i32",
                ast::value::NumberType::I64 => "i64",
            },
            ast::value::Type::Bool => "bool",
        };
        let bop_name = match self.binop_symbol {
            ast::value::Binop::Plus => "plus",
        };
        format!(
            "std_binop_{}_{}_{}_to_{}",
            bop_name,
            type_name(&self.input1),
            type_name(&self.input2),
            type_name(&self.output)
        )
    }

    fn make_function_class(
        self,
        function_classes: &mut Vec<asm::FunctionClass>,
        function_class_ctx: &mut FunctionClassContext,
    ) {
        let name = asm::FunctionClassId(self.name());
        let name_raw = self.name();
        let input_types = vec![
            typing::convert_value_type(self.input1),
            typing::convert_value_type(self.input2),
        ];
        let output_types = vec![typing::convert_value_type(self.output)];
        function_class_ctx.add_type(&name_raw, input_types.clone(), output_types.clone());
        function_classes.push(asm::FunctionClass {
            name,
            input_types,
            output_types,
        });
    }
}

struct Context {
    map: HashMap<String, ast::value::Type>,
    function_map: HashMap<String, ast::value::Type>,
}

impl Context {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            function_map: HashMap::new(),
        }
    }

    fn insert(&mut self, x: &str, t: ast::value::Type) {
        if self.map.insert(x.to_string(), t).is_some() {
            panic!("Shadowed variable {}", x)
        }
    }

    fn insert_function(&mut self, f: &str, t: ast::value::Type) {
        if self.function_map.insert(f.to_string(), t).is_some() {
            panic!("Duplicate function name {}", f)
        }
    }

    fn get(&self, x: &str) -> &ast::value::Type {
        self.map
            .get(x)
            .unwrap_or_else(|| panic!("Unbound variable {}", x))
    }

    fn get_function(&self, x: &str) -> &ast::value::Type {
        self.function_map
            .get(x)
            .unwrap_or_else(|| panic!("Unbound variable {}", x))
    }

    fn type_of_expr(&self, e: &ast::value::Expr) -> ast::value::Type {
        let (info, kind) = e;
        use ast::value::ExprKind::*;
        use ast::value::Type as VT;
        match kind {
            Var(x) => self.get(x).clone(),
            Num(n, t) => VT::Num(t.clone()),
            Bool(_) => VT::Bool,
            App(f, _es) => {
                // No regard for if this is actually well-typed haha
                self.get_function(f).clone()
            }
            Binop(bop, e1, e2) => self.type_of_bop(*info, bop, e1, e2),
        }
    }

    fn type_of_bop(
        &self,
        info: crate::error::Info,
        bop: &ast::value::Binop,
        e1: &ast::value::Expr,
        e2: &ast::value::Expr,
    ) -> ast::value::Type {
        use ast::value::Binop as B;
        use ast::value::Type as VT;
        let t1 = self.type_of_expr(e1);
        let t2 = self.type_of_expr(e2);
        match (bop, &t1, &t2) {
            // XXX Totally arbitrary that first num is picked, could change if necessary
            // to e.g. failing if they're unequal
            (B::Plus, VT::Num(_), VT::Num(_)) => t1,
            _ => panic!(
                "At {:?}: Type mismatch for binop {:?} on types {:?} and {:?}",
                info, bop, t1, t2
            ),
        }
    }
}

fn externalize_expr(
    e: &mut ast::value::Expr,
    ctx: &Context,
    external_bops: &mut Vec<ExternalBinop>,
) {
    let (info, kind) = e;
    match kind {
        // The important branch
        ast::value::ExprKind::Binop(bop, ref mut e1, ref mut e2) => {
            let input1 = ctx.type_of_expr(e1);
            let input2 = ctx.type_of_expr(e2);
            let binop_symbol = bop.clone();
            let output = ctx.type_of_bop(*info, bop, e1, e2);
            let ebop = ExternalBinop {
                input1,
                input2,
                output,
                binop_symbol,
            };

            let function_name = ebop.name();
            external_bops.push(ebop);

            externalize_expr(e1, ctx, external_bops);
            externalize_expr(e2, ctx, external_bops);
            let e1 = std::mem::take(e1);
            let e2 = std::mem::take(e2);

            *kind = ast::value::ExprKind::App(function_name, vec![*e1, *e2]);
        }

        // For some reason, former me decided to externalize all function applications for no
        // reason. Leaving it here in case it's ever needed
        /*
           // All branches below just apply recursively to their subexpressions
         ast::value::ExprKind::App(_, ref mut es) => {
            for e in es.iter_mut() {
                externalize_expr(e, ctx, external_bops);
            }
        },*/
        _ => (),
    }
}

pub fn externalize_binops(
    program: &mut ast::Program,
    function_classes: &mut Vec<asm::FunctionClass>,
    function_class_ctx: &mut FunctionClassContext,
) {
    let mut ctx = Context::new();
    for (_, decl_kind) in program.iter() {
        match decl_kind {
            ast::DeclKind::ExternCPU {
                name: extern_name,
                input: _,
                output,
            } => {
                let mut ctx_name = None;
                for (_, decl_kind) in program.iter() {
                    match decl_kind {
                        ast::DeclKind::FunctionClass {
                            name: class_name,
                            functions,
                        } => {
                            if functions
                                .iter()
                                .find(|s| s.to_string() == extern_name.to_string())
                                .is_some()
                            {
                                ctx_name = Some(class_name.clone());
                            }
                        }
                        _ => (),
                    }
                }
                if let Some(name) = ctx_name {
                    ctx.insert_function(&name, output.clone());
                }
            }
            _ => (),
        }
    }

    let mut external_bops: Vec<ExternalBinop> = Vec::new();
    for (_, decl_kind) in program.iter_mut() {
        match decl_kind {
            ast::DeclKind::ValueFunclet { statements, .. } => {
                for (_, stmt_kind) in statements.iter_mut() {
                    match stmt_kind {
                        ast::value::StmtKind::Let(x, ref mut e) => {
                            let t = ctx.type_of_expr(e);

                            externalize_expr(e, &ctx, &mut external_bops);

                            ctx.insert(x, t);
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }

    for ebop in external_bops.into_iter() {
        ebop.make_function_class(function_classes, function_class_ctx);
    }
}

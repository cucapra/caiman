use super::funclet_util::make_asm_funclet;
use super::function_classes::FunctionClassContext;
use super::{label, typing};
use crate::error::Info;
use crate::syntax::ast;
use caiman::assembly::ast as asm;
use caiman::ir::FuncletKind;

pub struct ValueFunclet(pub asm::Funclet);

pub fn lower_value_funclets(
    function_class_ctx: &FunctionClassContext,
    program: &ast::Program,
) -> Vec<ValueFunclet>
{
    program
        .iter()
        .filter_map(|(info, decl)| match decl {
            ast::DeclKind::ValueFunclet { name, input, output, statements } => Some(
                lower_value_funclet(function_class_ctx, *info, name, input, output, statements),
            ),
            _ => None,
        })
        .collect()
}

enum NodeLabel
{
    Name(String),
    StmtIndex(usize),
}

enum ExprTranslation
{
    NewExpr
    {
        name: asm::NodeId,
        pre_output: Vec<asm::NamedNode>,
        output: asm::Node,
        post_output: Vec<asm::NamedNode>,
        sub_exprs: Vec<ExprTranslation>,
    },
    NodeLink(asm::NodeId),
}

impl ExprTranslation
{
    fn to_named_nodes(self) -> Vec<Option<asm::NamedNode>>
    {
        if let ExprTranslation::NewExpr { name, pre_output, output, post_output, sub_exprs } = self
        {
            let mut nns: Vec<Option<asm::NamedNode>> = Vec::new();
            for sub_expr in sub_exprs.into_iter() {
                let mut sub_expr_nns = sub_expr.to_named_nodes();
                nns.append(&mut sub_expr_nns);
            }
            let mut pre_opts = pre_output.into_iter().map(|nn| Some(nn)).collect();
            let mut post_opts = post_output.into_iter().map(|nn| Some(nn)).collect();
            nns.append(&mut pre_opts);
            nns.push(Some(asm::NamedNode { name, node: output }));
            nns.append(&mut post_opts);
            nns
        } else {
            vec![]
        }
    }
}

fn make_constant_node(name: asm::NodeId, typ: ast::value::Type, value: String) -> ExprTranslation
{
    ExprTranslation::NewExpr {
        name,
        pre_output: vec![],
        output: asm::Node::Constant {
            value: Some(value),
            type_id: Some(typing::convert_value_type(typ)),
        },
        post_output: vec![],
        sub_exprs: vec![],
    }
}

fn translate_expr(expr: &ast::value::Expr, label: NodeLabel) -> ExprTranslation
{
    let (_info, kind) = expr;
    let name = match label {
        NodeLabel::Name(x) => label::label_node(&x),
        NodeLabel::StmtIndex(i) => label::label_node(&format!("stmt{}", i)),
    };
    use ast::value::ExprKind::*;
    use ast::value::Type;
    match kind {
        Var(x) => ExprTranslation::NodeLink(label::label_node(x)),
        Num(n, nt) => make_constant_node(name, Type::Num(nt.clone()), n.clone()),
        Bool(b) => make_constant_node(name, Type::Bool, (if *b { "1" } else { "0" }).to_string()),
        App(f, es) => {
            let mut sub_expr_ids: Vec<asm::NodeId> = Vec::new();
            let mut sub_exprs: Vec<ExprTranslation> = Vec::new();
            for (i, e) in es.iter().enumerate() {
                let label = NodeLabel::Name(format!("{}_subexp{}", name.0, i));
                let et = translate_expr(e, label);
                sub_expr_ids.push(match &et {
                    ExprTranslation::NewExpr { name, .. } => name.clone(),
                    ExprTranslation::NodeLink(id) => id.clone(),
                });
                sub_exprs.push(et);
            }
            let function_call = asm::Node::CallValueFunction {
                function_id: Some(asm::FunctionClassId(f.clone())),
                arguments: Some(sub_expr_ids.into_iter().map(|n| Some(n)).collect()),
            };
            let fcall_node_name = label::label_call_node(&name);
            let extract_result = asm::Node::ExtractResult {
                node_id: Some(fcall_node_name.clone()),
                index: Some(0),
            };
            ExprTranslation::NewExpr {
                name,
                pre_output: vec![asm::NamedNode { name: fcall_node_name, node: function_call }],
                output: extract_result,
                post_output: vec![],
                sub_exprs,
            }
        },
    }
}

fn lower_value_funclet(
    function_class_ctx: &FunctionClassContext,
    _info: Info,
    name: &str,
    input: &Vec<ast::Arg<ast::value::Type>>,
    output: &(Option<String>, ast::value::Type),
    // TODO: maybe convert these statements to a different form first like VIL.
    // How the rest of the to-funclet translation goes (with header stuff) should be the
    // same though!
    statements: &Vec<ast::value::Stmt>,
) -> ValueFunclet
{
    let mut returned_variable = None;
    let mut nodes: Vec<Option<asm::NamedNode>> = Vec::new();
    for (_i, (stmt_info, stmt_kind)) in statements.iter().enumerate() {
        match stmt_kind {
            ast::value::StmtKind::Let(x, e) => {
                let trans_expr = translate_expr(e, NodeLabel::Name(x.clone()));
                nodes.append(&mut trans_expr.to_named_nodes());
            },
            ast::value::StmtKind::Returns((_, ast::value::ExprKind::Var(x))) => {
                returned_variable = Some(label::label_node(x))
            },
            ast::value::StmtKind::Returns(_) => {
                println!(
                    "WARNING: Something other than a var was returned at {:?}. This is currently \
                     unsupported.",
                    stmt_info
                );
            },
        }
    }

    let tail_edge = asm::TailEdge::Return { return_values: Some(vec![returned_variable]) };

    let function_class =
        function_class_ctx.get(name).unwrap_or(asm::FunctionClassId(name.to_string()));
    let header = asm::FuncletHeader {
        name: asm::FuncletId(name.to_string()),
        args: input
            .iter()
            .map(|(name, typ)| asm::FuncletArgument {
                name: Some(asm::NodeId(name.to_string())),
                typ: typing::convert_value_type(typ.clone()),
                tags: Vec::new(),
            })
            .collect(),
        ret: vec![asm::FuncletArgument {
            name: output.0.clone().map(|s| asm::NodeId(s)),
            typ: typing::convert_value_type(output.1.clone()),
            tags: Vec::new(),
        }],
        binding: asm::FuncletBinding::ValueBinding(asm::FunctionClassBinding {
            default: true,
            function_class,
        }),
    };

    ValueFunclet(make_asm_funclet(FuncletKind::Value, header, nodes, tail_edge))
}

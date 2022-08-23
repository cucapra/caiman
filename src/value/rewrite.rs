use super::*;
use crate::ir;
use egg::rewrite as rw;
use std::convert::TryInto;

#[rustfmt::skip]
pub fn arithmetic() -> Vec<GraphRewrite> { vec![
    rw!("commutative add";  "(+ ?a ?b)"         => "(+ ?b ?a)"),
    rw!("associative add";  "(+ ?a (+ ?b ?c))"  => "(+ (+ ?a ?b) ?c)"),
    rw!("cvt add to sub";   "(+ ?a ?b)"         => "(- ?a (neg ?b))" if is_signed("?b")),
    rw!("cvt sub to add";   "(- ?a ?b)"         => "(+ ?a (neg ?b))" if is_signed("?b")),

    rw!("sub itself"; "(- ?a ?a)" => { CtdiApp::new("?a", 0) }),

    // You might think constant folding would perform these. Well, caiman's constant folding 
    // currently only works when *all* dependencies are constants, but these rewrites are
    // unique because they can be applied even when ?a isn't known.
    rw!("add 0"; "(+ ?a ?z)" => "?a" if is_zero("?z")),
    rw!("sub 0"; "(- ?a ?z)" => "?a" if is_zero("?z")),

    // (- csi{value=0} ?a) => (neg a) can be done by "sub to add" & "*int add 0"
]}

#[rustfmt::skip]
pub fn logical() -> Vec<GraphRewrite> { vec![
    rw!("commutative or"; "(|| ?a ?b)" => "(|| ?b ?a)"),
    rw!("commutative and"; "(&& ?a ?b)" => "(&& ?b ?a)"),
    rw!("associative or"; "(|| ?a (|| ?b ?c))" => "(|| (|| ?a ?b) ?c)"), 
    rw!("associative and"; "(&& ?a (&& ?b ?c))" => "(&& (&& ?a ?b) ?c)"), 
    rw!("distribute and"; "(&& ?a (|| ?b ?c))" => "(|| (&& ?a ?b) (&& ?a ?c))"),
    rw!("distribute or"; "(|| ?a (&& ?b ?c))" => "(&& (|| ?a ?b) (|| ?a ?c))"),
    rw!("logical or true"; "(|| ?a ?t)" => "?t" if is_true("?t")),
    rw!("logical and false"; "(&& ?a ?f)" => "?f" if is_false("?f"))
]}

use constant::Constant as Cs;
use ir::Type as Ty;

/// Returns true if the eclass has a single output, AND `f` returns true.
fn single_output(
    v: egg::Var,
    f: impl Fn(&ir::Type) -> bool,
) -> impl Fn(&mut Graph, egg::Id, &egg::Subst) -> bool {
    move |egraph, _, subst| {
        let id = subst[v];
        match egraph[id].data.single() {
            Some(o) => f(egraph
                .analysis
                .lookup_type(o.type_id)
                .expect("unknown type")),
            None => false,
        }
    }
}
fn is_signed(var: &str) -> impl Fn(&mut Graph, egg::Id, &egg::Subst) -> bool {
    single_output(var.parse().unwrap(), |ty| {
        matches!(ty, Ty::I8 | Ty::I16 | Ty::I32 | Ty::I64)
    })
}
fn is_unsigned(var: &str) -> impl Fn(&mut Graph, egg::Id, &egg::Subst) -> bool {
    single_output(var.parse().unwrap(), |ty| {
        matches!(ty, Ty::U8 | Ty::U16 | Ty::U32 | Ty::U64)
    })
}

fn is_zero(var: &str) -> impl Fn(&mut Graph, egg::Id, &egg::Subst) -> bool {
    let var = var.parse().unwrap();
    move |egraph, _, subst| {
        let cs = match egraph[subst[var]].data.constant() {
            Some(cs) => cs,
            None => return false,
        };
        match cs {
            Cs::U8(0) | Cs::U16(0) | Cs::U32(0) | Cs::U64(0) => true,
            Cs::I8(0) | Cs::I16(0) | Cs::I32(0) | Cs::I64(0) => true,
            _ => false,
        }
    }
}
fn is_true(var: &str) -> impl Fn(&mut Graph, egg::Id, &egg::Subst) -> bool {
    let var = var.parse().unwrap();
    move |egraph, _, subst| matches!(egraph[subst[var]].data.constant(), Some(Cs::Bool(true)))
}
fn is_false(var: &str) -> impl Fn(&mut Graph, egg::Id, &egg::Subst) -> bool {
    let var = var.parse().unwrap();
    move |egraph, _, subst| matches!(egraph[subst[var]].data.constant(), Some(Cs::Bool(false)))
}

struct CtdiApp {
    var: egg::Var,
    val: i128,
}
impl CtdiApp {
    fn new(var: &str, val: i128) -> Self {
        let var = var.parse().unwrap();
        Self { var, val }
    }
}
impl egg::Applier<Node, Analysis> for CtdiApp {
    fn apply_one(
        &self,
        egraph: &mut egg::EGraph<Node, Analysis>,
        _eclass: egg::Id,
        subst: &egg::Subst,
        _searcher_ast: Option<&egg::PatternAst<Node>>,
        _rule_name: egg::Symbol,
    ) -> Vec<egg::Id> {
        let id = subst[self.var];
        let type_id = egraph[id]
            .data
            .single()
            .expect("can't constant fold aggregates")
            .type_id;
        let ty = egraph.analysis.lookup_type(type_id).expect("unknown type");
        let cs = match ty {
            Ty::I8 => Cs::I8(self.val.try_into().expect("out of range constant")),
            Ty::I16 => Cs::I16(self.val.try_into().expect("out of range constant")),
            Ty::I32 => Cs::I32(self.val.try_into().expect("out of range constant")),
            Ty::I64 => Cs::I64(self.val.try_into().expect("out of range constant")),
            Ty::U8 => Cs::U8(self.val.try_into().expect("out of range constant")),
            Ty::U16 => Cs::U16(self.val.try_into().expect("out of range constant")),
            Ty::U32 => Cs::U32(self.val.try_into().expect("out of range constant")),
            Ty::U64 => Cs::U64(self.val.try_into().expect("out of range constant")),
            _ => panic!("not an integer"),
        };
        let folded = egraph.add(cs.to_node(type_id));
        if egraph.union(id, folded) {
            vec![folded]
        } else {
            vec![]
        }
    }
}

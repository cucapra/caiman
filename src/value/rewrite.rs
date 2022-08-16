use super::{Graph, GraphRewrite};
use crate::ir;
use egg::rewrite as rw;

#[rustfmt::skip]
pub fn arithmetic() -> Vec<GraphRewrite> { vec![
    rw!("commutative add";  "(+ ?a ?b)"         => "(+ ?b ?a)"),
    rw!("associative add";  "(+ ?a (+ ?b ?c))"  => "(+ (+ ?a ?b) ?c)"),
    rw!("cvt add to sub";   "(+ ?a ?b)"         => "(- ?a (neg ?b))" if is_signed("?b")),
    rw!("cvt sub to add";   "(- ?a ?b)"         => "(+ ?a (neg ?b))" if is_signed("?b")),

    rw!("sub itself signed"; "(- ?a ?a)" => "csi{value=0}" if is_signed("?a")),
    rw!("sub itself unsigned"; "(- ?a ?a)" => "cui{value=0}" if is_unsigned("?a")),

    // You might think constant folding would perform these. Well, caiman's constant folding 
    // currently only works when *all* dependencies are constants, but these rewrites are
    // unique because they can be applied even when ?a isn't known.
    rw!("uint add 0"; "(+ ?a cui{value=0})" => "?a"),
    rw!("sint add 0"; "(+ ?a csi{value=0})" => "?a"),
    rw!("uint sub 0"; "(- ?a cui{value=0})" => "?a"),
    rw!("sint sub 0"; "(- ?a csi{value=0})" => "?a"),
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
    rw!("logical or true"; "(|| ?a cb{value=true})" => "cb{value=true}"),
    rw!("logical and false"; "(&& ?a cb{value=false})" => "cb{value=false}")
]}

use ir::Type as Ty;

/// Returns true if the eclass has a single output, AND `f` returns true.
fn single_output(
    v: egg::Var,
    f: impl Fn(&ir::Type) -> bool,
) -> impl Fn(&mut Graph, egg::Id, &egg::Subst) -> bool {
    move |egraph, _, subst| {
        let id = subst[v];
        let type_id = match egraph[id].data.output_types.as_deref() {
            Some([type_id]) => *type_id,
            Some(_) => return false, // aggregate (think ID list)
            None => panic!("eclass with no type ID made it to the rewrite stage"),
        };
        let ty = egraph.analysis.lookup_type(type_id).expect("unknown type");
        f(ty)
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

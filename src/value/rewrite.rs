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

fn is_signed(var: &str) -> impl Fn(&mut Graph, egg::Id, &egg::Subst) -> bool {
    let var = var.parse().unwrap();
    move |egraph, _, subst| {
        let type_id = match egraph[subst[var]].data.output_types.get(0) {
            Some(type_id) => *type_id,
            None => return false, // aggregate (think ID list)
        };
        matches!(
            egraph.analysis.lookup_type(type_id).expect("unknown type"),
            ir::Type::I8 | ir::Type::I16 | ir::Type::I32 | ir::Type::I64
        )
    }
}

fn is_unsigned(var: &str) -> impl Fn(&mut Graph, egg::Id, &egg::Subst) -> bool {
    let var = var.parse().unwrap();
    move |egraph, _, subst| {
        let type_id = match egraph[subst[var]].data.output_types.get(0) {
            Some(type_id) => *type_id,
            None => return false, // aggregate (think ID list)
        };
        matches!(
            egraph.analysis.lookup_type(type_id).expect("unknown type"),
            ir::Type::U8 | ir::Type::U16 | ir::Type::U32 | ir::Type::U64
        )
    }
}

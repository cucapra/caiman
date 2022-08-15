use super::{constant::Constant, GraphInner, GraphRewrite};
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
    // unique in that they can be applied even when ?a isn't known.
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

fn is_signed(var: &str) -> impl Fn(&mut GraphInner, egg::Id, &egg::Subst) -> bool {
    let var = var.parse().unwrap();
    move |egraph, _, subst| match egraph[subst[var]].data.constant {
        Some(Constant::I8(_) | Constant::I16(_) | Constant::I32(_) | Constant::I64(_)) => true,
        _ => false,
    }
}

fn is_unsigned(var: &str) -> impl Fn(&mut GraphInner, egg::Id, &egg::Subst) -> bool {
    let var = var.parse().unwrap();
    move |egraph, _, subst| match egraph[subst[var]].data.constant {
        Some(Constant::U8(_) | Constant::U16(_) | Constant::U32(_) | Constant::U64(_)) => true,
        _ => false,
    }
}

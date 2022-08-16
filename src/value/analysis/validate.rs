use std::convert::TryFrom;

use super::*;

pub fn validate(runner: &GraphRunner) {
    for eclass in runner.egraph.classes() {
        validate_eclass(&runner.egraph, eclass);
    }
}
fn validate_eclass(egraph: &Graph, eclass: &GraphClass) {
    assert!(
        eclass.data.output_types.is_some(),
        "all eclasses must have type info"
    );
    for node in eclass.nodes.iter() {
        validate_node(egraph, eclass, node);
    }
}
/// Invariants: The node's eclass has *some* associated type information.
fn validate_node(egraph: &Graph, eclass: &GraphClass, node: &Node) {
    match &node.kind {
        NodeKind::IdList => (),
        NodeKind::Operation { kind } => validate_operation(egraph, eclass, kind, &node.deps),
        NodeKind::Param { funclet_id, index } => {
            validate_param(egraph, eclass, *funclet_id, *index, &node.deps)
        }
    }
}
// Invariants: those from validate_node
fn validate_param(
    egraph: &Graph,
    eclass: &GraphClass,
    funclet_id: ir::FuncletId,
    index: usize,
    deps: &[egg::Id],
) {
    // TODO: We should also validate the index
    assert!(deps.is_empty(), "param node has no deps");
    assert!(
        egraph.analysis.blocks.contains_key(&funclet_id),
        "orphaned param"
    );
}

// Invariants: those from validate_node
fn validate_operation(egraph: &Graph, eclass: &GraphClass, kind: &OperationKind, deps: &[egg::Id]) {
    use constant::Constant as Cs;
    use ir::Type as Ty;
    use OperationKind as Ok;
    match kind {
        Ok::None {} => {
            assert!(deps.is_empty());
            assert!(eclass.data.output_types.as_ref().unwrap().is_empty(),);
        }
        &Ok::ConstantBool { value, type_id } => {
            assert!(deps.is_empty());

            let eclass_type_id = extract_single_type(eclass);
            assert_eq!(eclass_type_id, type_id);
            let ty = egraph.analysis.lookup_type(type_id).expect("unknown type");
            assert!(matches!(ty, Ty::Bool));

            assert!(eclass.data.constant == Some(Cs::Bool(value)));
        }
        &Ok::ConstantInteger { value, type_id } => {
            assert!(deps.is_empty());

            let eclass_type_id = extract_single_type(eclass);
            assert_eq!(eclass_type_id, type_id);
            let ty = egraph.analysis.lookup_type(type_id).expect("unknown type");

            match ty {
                Ty::I8 => {
                    let value = i8::try_from(value).expect("out-of-range constant");
                    assert!(eclass.data.constant == Some(Cs::I8(value)));
                }
                Ty::I16 => {
                    let value = i16::try_from(value).expect("out-of-range constant");
                    assert!(eclass.data.constant == Some(Cs::I16(value)));
                }
                Ty::I32 => {
                    let value = i32::try_from(value).expect("out-of-range constant");
                    assert!(eclass.data.constant == Some(Cs::I32(value)));
                }
                Ty::I64 => {
                    assert!(eclass.data.constant == Some(Cs::I64(value)));
                }
                _ => panic!("incompatible type for ConstantInteger"),
            }
        }
        &Ok::ConstantUnsignedInteger { value, type_id } => {
            assert!(deps.is_empty());

            let eclass_type_id = extract_single_type(eclass);
            assert_eq!(eclass_type_id, type_id);
            let ty = egraph.analysis.lookup_type(type_id).expect("unknown type");

            match ty {
                Ty::U8 => {
                    let value = u8::try_from(value).expect("out-of-range constant");
                    assert!(eclass.data.constant == Some(Cs::U8(value)));
                }
                Ty::U16 => {
                    let value = u16::try_from(value).expect("out-of-range constant");
                    assert!(eclass.data.constant == Some(Cs::U16(value)));
                }
                Ty::U32 => {
                    let value = u32::try_from(value).expect("out-of-range constant");
                    assert!(eclass.data.constant == Some(Cs::U32(value)));
                }
                Ty::U64 => {
                    assert!(eclass.data.constant == Some(Cs::U64(value)));
                }
                _ => panic!("incompatible type for ConstantInteger"),
            }
        }
        &Ok::ExtractResult { index } => {
            assert!(deps.len() == 1);
            let aggregate = &egraph[deps[0]];
            match aggregate.data.output_types.as_ref() {
                Some(o) => assert!(o.len() > index, "try to extract nonexistent result"),
                None => panic!("all eclasses must have type info"),
            }
        }
        Ok::Unop { kind } => {
            // TODO: check compatibility with the operation
            // TODO: typecheck output?
            assert!(deps.len() == 1);
        }
        Ok::Binop { kind } => {
            // TODO: same as unop
            assert!(deps.len() == 2);
            let d0 = &egraph[deps[0]];
            let d1 = &egraph[deps[1]];
            match (d0.data.output_types.as_ref(), d1.data.output_types.as_ref()) {
                (Some(t0), Some(t1)) => assert_eq!(t0, t1, "binop operand types differ"),
                _ => panic!("all eclasses must have type info"),
            }
        }
        Ok::CallValueFunction { function_id } => {
            // TODO: what to validate? we probably need some external data structure
        }
        Ok::CallExternalCpu {
            external_function_id,
        } => {
            // TODO: blocked on ffi interface (checking output types, etc)
        }
        Ok::CallExternalGpuCompute {
            external_function_id,
        } => {
            // TODO: blocked on ffi interface (checking output types, etc)
        }
    }
}

fn extract_single_type(eclass: &GraphClass) -> ir::TypeId {
    match eclass.data.output_types.as_deref().unwrap() {
        [single] => *single,
        _ => panic!("single output type expected"),
    }
}

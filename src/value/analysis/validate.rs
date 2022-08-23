use std::convert::TryFrom;

use super::*;

pub fn validate(runner: &GraphRunner) {
    for eclass in runner.egraph.classes() {
        validate_eclass(&runner.egraph, eclass);
    }
}
fn validate_eclass(egraph: &Graph, eclass: &GraphClass) {
    assert!(eclass.data.finished(), "unfinished eclass");
    for node in eclass.nodes.iter() {
        validate_node(egraph, eclass, node);
    }
}
/// Invariants: The node's eclass has *some* associated type information.
fn validate_node(egraph: &Graph, eclass: &GraphClass, node: &Node) {
    match &node.kind {
        NodeKind::IdList => validate_idlist(egraph, eclass, &node.deps),
        NodeKind::Operation { kind } => validate_operation(egraph, eclass, kind, &node.deps),
        NodeKind::Param { funclet_id, index } => {
            validate_param(egraph, eclass, *funclet_id, *index, &node.deps)
        }
    }
}
// Invariants: those from validate_node
fn validate_idlist(egraph: &Graph, eclass: &GraphClass, deps: &[GraphId]) {}
// Invariants: those from validate_node
fn validate_param(
    egraph: &Graph,
    eclass: &GraphClass,
    funclet_id: ir::FuncletId,
    index: usize,
    deps: &[GraphId],
) {
    // TODO: We should also validate the index
    assert!(deps.is_empty(), "param node has no deps");
    assert!(
        egraph.analysis.blocks.contains_key(&funclet_id),
        "orphaned param"
    );
}

// Invariants: those from validate_node
fn validate_operation(egraph: &Graph, eclass: &GraphClass, kind: &OperationKind, deps: &[GraphId]) {
    use constant::Constant as Cs;
    use ir::Type as Ty;
    use OperationKind as Ok;
    match kind {
        &Ok::ConstantBool { value, type_id } => {
            assert!(deps.is_empty());

            let eclass_type_id = extract_single_type(eclass);
            assert_eq!(eclass_type_id, type_id);
            let ty = egraph.analysis.lookup_type(type_id).expect("unknown type");
            assert!(matches!(ty, Ty::Bool));

            assert_eq!(eclass.data.constant(), Some(&Cs::Bool(value)));
        }
        &Ok::ConstantInteger { value, type_id } => {
            assert!(deps.is_empty());

            let eclass_type_id = extract_single_type(eclass);
            assert_eq!(eclass_type_id, type_id);
            let ty = egraph.analysis.lookup_type(type_id).expect("unknown type");

            let cs = eclass.data.constant();
            match ty {
                Ty::I8 => {
                    let value = i8::try_from(value).expect("out-of-range constant");
                    assert_eq!(cs, Some(&Cs::I8(value)));
                }
                Ty::I16 => {
                    let value = i16::try_from(value).expect("out-of-range constant");
                    assert_eq!(cs, Some(&Cs::I16(value)));
                }
                Ty::I32 => {
                    let value = i32::try_from(value).expect("out-of-range constant");
                    assert_eq!(cs, Some(&Cs::I32(value)));
                }
                Ty::I64 => {
                    assert!(cs == Some(&Cs::I64(value)));
                }
                _ => panic!("incompatible type for ConstantInteger"),
            }
        }
        &Ok::ConstantUnsignedInteger { value, type_id } => {
            assert!(deps.is_empty());

            let eclass_type_id = extract_single_type(eclass);
            assert_eq!(eclass_type_id, type_id);
            let ty = egraph.analysis.lookup_type(type_id).expect("unknown type");

            let cs = eclass.data.constant();
            match ty {
                Ty::U8 => {
                    let value = u8::try_from(value).expect("out-of-range constant");
                    assert_eq!(cs, Some(&Cs::U8(value)));
                }
                Ty::U16 => {
                    let value = u16::try_from(value).expect("out-of-range constant");
                    assert_eq!(cs, Some(&Cs::U16(value)));
                }
                Ty::U32 => {
                    let value = u32::try_from(value).expect("out-of-range constant");
                    assert_eq!(cs, Some(&Cs::U32(value)));
                }
                Ty::U64 => {
                    assert_eq!(cs, Some(&Cs::U64(value)));
                }
                _ => panic!("incompatible type for ConstantInteger"),
            }
        }
        &Ok::ExtractResult { index } => {
            assert!(deps.len() == 1);
            let aggregate = &egraph[deps[0]];
            let os = aggregate
                .data
                .multiple()
                .expect("expected multiple output dep for ExtractResult");
            assert!(os.len() > index, "extract nonexistent result");
            assert_eq!(
                &os[index],
                eclass
                    .data
                    .single()
                    .expect("extractresult should have single type"),
                "type disagreement in ExtractNode"
            );
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
            match (d0.data.single(), d1.data.single()) {
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
    eclass
        .data
        .single()
        .expect("expected single output")
        .type_id
}

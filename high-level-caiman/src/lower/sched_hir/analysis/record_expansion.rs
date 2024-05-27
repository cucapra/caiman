use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    enum_cast,
    lower::sched_hir::{cfg::Cfg, HirBody, HirInstr, Terminator, TripleTag},
    parse::ast::DataType,
};

use super::{analyze, Fact, Forwards};

/// Dataflow for transforming the encoded variables to become scoped.
/// Ex: `x_gpu` of encoder `e_0` will become `e_0::x_gpu`
#[derive(Clone, Debug, PartialEq, Eq)]
struct EncodeTransform {
    fence_map: HashMap<String, String>,
    data_types: Rc<RefCell<HashMap<String, DataType>>>,
}

impl EncodeTransform {
    pub fn top(data_types: &HashMap<String, DataType>) -> Self {
        Self {
            fence_map: HashMap::new(),
            data_types: Rc::new(RefCell::new(data_types.clone())),
        }
    }
}

impl Fact for EncodeTransform {
    fn meet(mut self, other: &Self) -> Self {
        for (k, v) in &other.fence_map {
            assert!(!self.fence_map.contains_key(k) || self.fence_map.get(k).unwrap() == v);
        }
        self.fence_map
            .extend(other.fence_map.iter().map(|(x, y)| (x.clone(), y.clone())));
        self
    }

    fn transfer_instr(&mut self, stmt: crate::lower::sched_hir::HirInstr<'_>, _: usize) {
        match stmt {
            HirInstr::Stmt(HirBody::BeginEncoding {
                encoder,
                device_vars,
                ..
            }) => {
                if let DataType::Encoder(Some(dt)) = &self
                    .data_types
                    .borrow()
                    .get(encoder)
                    .unwrap_or_else(|| panic!("Missing type for {encoder}"))
                {
                    if let DataType::RemoteObj { all, .. } = &**dt {
                        device_vars.clear();
                        for var in all.keys() {
                            device_vars
                                .push((format!("{encoder}::{var}"), TripleTag::new_unspecified()));
                        }
                    }
                }
            }
            HirInstr::Stmt(HirBody::Submit { dest, src, .. }) => {
                self.fence_map.insert(dest.clone(), src.clone());
            }
            HirInstr::Tail(Terminator::Return { dests, rets, .. }) => {
                for ((dest, _), src) in dests.iter().zip(rets.iter()) {
                    if let Some(src) = self.fence_map.get(src) {
                        let mut src = src.clone();
                        while let Some(new_src) = self.fence_map.get(&src) {
                            src = new_src.clone();
                        }
                        self.fence_map.insert(dest.clone(), src);
                    }
                }
            }
            HirInstr::Stmt(HirBody::Sync { dests, srcs, .. }) => {
                dests.process(|record| {
                    let dt = self.data_types.borrow();
                    let record_type = dt.get(record).unwrap();
                    let rt = enum_cast!(DataType::Record, record_type);
                    rt.iter()
                        .map(|(name, _)| {
                            (format!("{record}::{name}"), TripleTag::new_unspecified())
                        })
                        .collect()
                });
                srcs.process(|src| {
                    let dt = self.data_types.borrow();
                    let mut ret = vec![src.clone()];
                    let src_type = dt.get(src).unwrap();
                    if let DataType::Fence(Some(t)) = src_type {
                        if let DataType::RemoteObj { read, .. } = &**t {
                            for var in read {
                                ret.push(format!(
                                    "{}::{var}",
                                    self.fence_map.get(src).unwrap_or(src)
                                ));
                            }
                        }
                    }
                    ret
                });
            }
            _ => (),
        }
    }

    type Dir = Forwards;
}

/// Uses type information to insert device variables into the `BeginEncoding` operator.
/// Also used type information to insert variables into the `Sync` operator.
pub fn transform_encode_pass(cfg: &mut Cfg, data_types: &HashMap<String, DataType>) {
    // Map from fence to the encoder that holds its variables
    // TODO: do all the record expansion here?
    let top = EncodeTransform::top(data_types);
    let _ = analyze(cfg, &top);
}

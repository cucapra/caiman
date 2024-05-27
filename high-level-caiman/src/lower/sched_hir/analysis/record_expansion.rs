use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    enum_cast,
    lower::sched_hir::{cfg::Cfg, DataMovement, HirBody, HirInstr, Terminator, TripleTag},
    parse::ast::{DataType, FlaggedType},
    typing::{Context, SchedOrExtern},
};

use super::{bft, Fact, Forwards};

/// Dataflow for transforming the encoded variables to become scoped
/// and expanding records into all their fields.
/// Ex: `x_gpu` of encoder `e_0` will become `e_0::x_gpu`
#[derive(Clone, Debug)]
struct EncodeTransform {
    fence_map: HashMap<String, String>,
    data_types: Rc<RefCell<HashMap<String, DataType>>>,
    ctx: Rc<RefCell<Context>>,
}

impl PartialEq for EncodeTransform {
    fn eq(&self, other: &Self) -> bool {
        self.fence_map == other.fence_map
    }
}

impl Eq for EncodeTransform {}

impl EncodeTransform {
    pub fn top(data_types: &HashMap<String, DataType>, ctx: &Context) -> Self {
        Self {
            fence_map: HashMap::new(),
            data_types: Rc::new(RefCell::new(data_types.clone())),
            ctx: Rc::new(RefCell::new(ctx.clone())),
        }
    }

    fn expand_args<T: Iterator<Item = (String, TripleTag)>>(
        &self,
        args: T,
    ) -> Vec<(String, TripleTag)> {
        let mut new_args = Vec::new();
        for (arg, arg_t) in args {
            let mut new_tag = TripleTag::new_unspecified();
            new_tag.timeline = arg_t.timeline.clone();
            new_args.push((arg.clone(), arg_t));
            match &self.data_types.borrow()[&arg] {
                DataType::Fence(Some(t)) | DataType::Encoder(Some(t)) => {
                    if let DataType::RemoteObj { all, .. } = &**t {
                        new_args.extend(all.keys().map(|x| {
                            (
                                format!("{}::{x}", self.fence_map.get(&arg).unwrap_or(&arg)),
                                new_tag.clone(),
                            )
                        }));
                    }
                }
                DataType::Record(fields) => {
                    new_args.extend(fields.iter().map(|(name, _)| {
                        (
                            format!("{}::{name}", self.fence_map.get(&arg).unwrap_or(&arg)),
                            new_tag.clone(),
                        )
                    }));
                }
                _ => {}
            }
        }
        new_args
    }

    fn replace_call_args(&self, args: &[String], sig: &[FlaggedType]) -> Vec<String> {
        let mut new_args = Vec::new();
        for (arg_name, sig) in args.iter().zip(sig) {
            match sig {
                FlaggedType {
                    base: DataType::RemoteObj { all, .. } | DataType::Record(all),
                    ..
                } => {
                    for field_name in all.keys() {
                        new_args.push(format!(
                            "{}::{field_name}",
                            self.fence_map.get(arg_name).unwrap_or(arg_name)
                        ));
                    }
                }
                FlaggedType {
                    base: DataType::Fence(Some(t)) | DataType::Encoder(Some(t)),
                    ..
                } => {
                    new_args.push(arg_name.clone());
                    if let DataType::RemoteObj { all, .. } = &**t {
                        for field_name in all.keys() {
                            new_args.push(format!(
                                "{}::{field_name}",
                                self.fence_map.get(arg_name).unwrap_or(arg_name)
                            ));
                        }
                    } else {
                        panic!("Unexpected inner type of fence/encoder");
                    }
                }
                _ => new_args.push(arg_name.clone()),
            }
        }
        new_args
    }

    fn transfer_tail(&mut self, term: &mut Terminator) {
        match term {
            Terminator::Return { dests, rets, .. } => {
                // we want to expand the final return in the program and NOT the final return
                // in the special final basic block
                if dests.iter().all(|(x, _)| x.starts_with("_out")) && dests.len() > rets.len() {
                    // if we're the final return with something to expand, then the outputs
                    // should outnumber the return arguments since the IO is already
                    // expanded
                    *rets = self
                        .expand_args(
                            std::mem::take(rets)
                                .into_iter()
                                .map(|x| (x, TripleTag::new_unspecified())),
                        )
                        .into_iter()
                        .map(|(x, _)| x)
                        .collect::<Vec<_>>();
                }
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
            Terminator::Call(dests, call) => {
                if let Some(SchedOrExtern::Sched(target_info)) =
                    self.ctx.borrow().scheds.get(&call.target)
                {
                    call.args = self.replace_call_args(&call.args, &target_info.dtype_sig.input);
                    *dests = self.expand_args(std::mem::take(dests).into_iter());
                }
            }
            // the following should not be introduced yet
            Terminator::CaptureCall { .. } | Terminator::Next(..) => {
                panic!("Unexpected terminator, pass out of order")
            }
            Terminator::Yield(_, rets) if !rets.is_empty() => {
                panic!("Unexpected yield, pass out of order")
            }
            _ => (),
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
            HirInstr::Stmt(HirBody::EncodeDo {
                dests,
                func,
                encoder,
                ..
            }) => {
                for (dest, _) in dests {
                    *dest = format!("{encoder}::{dest}");
                }
                for arg in func.args.iter_mut().skip(func.num_dims) {
                    *arg = format!("{encoder}::{arg}");
                }
            }
            HirInstr::Stmt(HirBody::DeviceCopy {
                dest, dir, encoder, ..
            }) => {
                assert_eq!(*dir, DataMovement::HostToDevice);
                *dest = format!("{encoder}::{dest}");
            }
            HirInstr::Stmt(HirBody::Submit { dest, src, .. }) => {
                self.fence_map.insert(dest.clone(), src.clone());
            }
            HirInstr::Tail(term) => self.transfer_tail(term),
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
            HirInstr::Stmt(HirBody::InAnnotation(_, annot) | HirBody::OutAnnotation(_, annot)) => {
                for (arg, _) in annot {
                    if arg.contains("::") {
                        let mut split = arg.split("::");
                        let fence = split.next().unwrap();
                        let var = split.next().unwrap();
                        *arg = format!(
                            "{}::{var}",
                            self.fence_map.get(fence).map_or(fence, String::as_str)
                        );
                    }
                }
            }
            HirInstr::Stmt(_) => (),
        }
    }

    type Dir = Forwards;
}

/// Uses type information to insert device variables into the `BeginEncoding` operator.
/// Also used type information to insert variables into the `Sync` operator and
/// expand record arguments.
pub fn transform_encode_pass(cfg: &mut Cfg, data_types: &HashMap<String, DataType>, ctx: &Context) {
    // Map from fence to the encoder that holds its variables
    // TODO: do all the record expansion here?
    let top = EncodeTransform::top(data_types, ctx);
    bft(cfg, &top);
}

//! Contains logic for expanding records out into their fields and
//! renaming fields to be scoped with their encoder. When expanding
//! function call arguments or final returns, we use the unexpanded
//! signature of the target function to expand the arguments such that
//! we can return subtypes.
//!
//! For example, if we have a function that returns `{x: i32, y: i32}`, then:
//!
//! ```text
//! let res: {g: i32, y: i32, x: i32} = // ...
//! res
//! ```
//!
//! will expand to:
//!
//! ```text
//! let res::g, res::y, res::x = // ...
//! (res::x, res::y)
//! ```
//!
//! This file does not expand records in the inputs and outputs of functions.
//!
//! We do record expansion here because we need dataflow information. For example,
//! if we have a fence that is created by submitting an encoder, then its
//! variables actually belong to the encoder. So we need to expand the variables
//! using the encoder's name.

use std::collections::HashMap;

use crate::{
    enum_cast,
    lower::sched_hir::{cfg::Cfg, DataMovement, HirBody, HirInstr, Terminator, TripleTag},
    parse::ast::{DataType, FlaggedType},
    typing::{Context, SchedOrExtern},
};

use super::{bft_transform, Fact, Forwards};

/// Dataflow for transforming the encoded variables to become scoped
/// and expanding records into all their fields.
/// Ex: `x_gpu` of encoder `e_0` will become `e_0::x_gpu`
#[derive(Clone, Debug)]
struct EncodeTransform<'a> {
    fence_map: HashMap<String, String>,
    data_types: &'a HashMap<String, DataType>,
    ctx: &'a Context,
    // the unexpanded output of the current function
    sig_out: &'a Vec<FlaggedType>,
}

impl<'a> PartialEq for EncodeTransform<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.fence_map == other.fence_map
    }
}

impl<'a> Eq for EncodeTransform<'a> {}

impl<'a> EncodeTransform<'a> {
    pub fn top(
        data_types: &'a HashMap<String, DataType>,
        ctx: &'a Context,
        sig_out: &'a Vec<FlaggedType>,
    ) -> Self {
        Self {
            fence_map: HashMap::new(),
            data_types,
            ctx,
            sig_out,
        }
    }

    /// Expands a list of arguments based on a getter functin returning the datatype
    /// of the argument.
    /// # Arguments
    /// * `args` - The arguments to expand
    /// * `dt_getter` - A function that returns the datatype of an argument given the
    /// argument name and the index of the argument in the list.
    /// # Returns
    /// A list of expanded arguments
    fn expand_arg_helper<T: Iterator<Item = (String, TripleTag)>>(
        &self,
        args: T,
        dt_getter: impl Fn(&str, usize) -> &'a DataType,
    ) -> Vec<(String, TripleTag)> {
        let mut new_args = Vec::new();
        for (id, (arg, arg_t)) in args.enumerate() {
            let mut new_tag = TripleTag::new_unspecified();
            new_tag.timeline = arg_t.timeline.clone();
            match dt_getter(&arg, id) {
                DataType::Fence(Some(t)) | DataType::Encoder(Some(t)) => {
                    new_args.push((arg.clone(), arg_t));
                    if let DataType::RemoteObj { all, .. } = &**t {
                        new_args.extend(all.iter().map(|(x, _)| {
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
                _ => new_args.push((arg.clone(), arg_t)),
            }
        }
        new_args
    }

    /// Expands the arguments of a function call based on the signature of the
    /// target. We expand record arguments in the order of declaration, as such,
    /// `sig` must be the signature of the target so we expand the arguments in
    /// the correct order.
    #[allow(unused)]
    fn expand_args<T: Iterator<Item = (String, TripleTag)>>(
        &self,
        args: T,
    ) -> Vec<(String, TripleTag)> {
        self.expand_arg_helper(args, |arg, _| self.data_types.get(arg).unwrap())
    }

    /// Expands the return values of a function call based on the signature of the
    /// target. We expand record arguments in the order of declaration, as such,
    /// `sig` must be the signature of the target so we expand the arguments in
    /// the correct order.
    fn expand_rets<T: Iterator<Item = (String, TripleTag)>>(
        &self,
        args: T,
        sig: &[FlaggedType],
    ) -> Vec<(String, TripleTag)> {
        self.expand_arg_helper(args, |_, id| &sig[id].base)
    }

    /// Replaces the arguments of a function call based on the signature of the
    /// target. We expand record arguments in the order of declaration, as such,
    /// `sig` must be the signature of the target so we expand the arguments in
    /// the correct order.
    ///
    /// # Arguments
    /// * `args` - The arguments to replace
    /// * `sig` - The signature of the target, excluding the template arguments
    /// * `num_dims` - The number of dimensional arguments of the target. Dimensional
    /// arguments are copied as is to the new args since they must all be `i32`.
    /// Furthermore, dimensional arguments are not expressed in the target signature.
    fn replace_call_args(
        &self,
        args: &[String],
        sig: &[FlaggedType],
        num_dims: usize,
    ) -> Vec<String> {
        let mut new_args = Vec::new();
        for arg_name in args.iter().take(num_dims) {
            new_args.push(arg_name.clone());
        }
        for (arg_name, sig) in args.iter().skip(num_dims).zip(sig) {
            match sig {
                FlaggedType {
                    base: DataType::RemoteObj { all, .. } | DataType::Record(all),
                    ..
                } => {
                    for (field_name, _) in all {
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
                        for (field_name, _) in all {
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

    /// Expands and rename record fields in terminators
    fn transfer_tail(&mut self, term: &mut Terminator) {
        match term {
            Terminator::Return { dests, rets, .. } => {
                // we want to expand the final return in the program and NOT the final return
                // in the special final basic block
                if dests.iter().all(|(x, _)| x.starts_with("_out")) {
                    // use the signature so we expand in the correct order
                    *rets = self.replace_call_args(rets, self.sig_out, 0);
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
                if let Some(SchedOrExtern::Sched(target_info)) = self.ctx.scheds.get(&call.target) {
                    call.args = self.replace_call_args(
                        &call.args,
                        &target_info.dtype_sig.input,
                        call.num_dims,
                    );
                    *dests = self.expand_rets(
                        std::mem::take(dests).into_iter(),
                        &target_info.dtype_sig.output,
                    );
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

    /// Renames annatiions to use the canonical name of a fence or an encoder.
    /// Also copies any timeline annotations to all fields of a record if
    /// that record is a part of a fence or encoder.
    fn expand_annotations(&self, annot: &mut Vec<(String, TripleTag)>) {
        let mut record_fields = Vec::new();
        for (arg, t) in annot.iter_mut() {
            if arg.contains("::") {
                let mut split = arg.split("::");
                let fence = split.next().unwrap();
                let var = split.next().unwrap();
                *arg = format!(
                    "{}::{var}",
                    self.fence_map.get(fence).map_or(fence, String::as_str)
                );
            }
            if let Some(DataType::Fence(Some(ty)) | DataType::Encoder(Some(ty))) =
                self.data_types.get(arg)
            {
                if let DataType::RemoteObj { all, .. } = &**ty {
                    record_fields.extend(all.iter().map(|(x, _)| {
                        (
                            format!("{}::{x}", self.fence_map.get(arg).unwrap_or(arg)),
                            TripleTag {
                                timeline: t.timeline.clone(),
                                ..TripleTag::new_unspecified()
                            },
                        )
                    }));
                }
            }
        }
        record_fields.extend(std::mem::take(&mut *annot).into_iter());
        *annot = record_fields;
    }
}

impl<'a> Fact for EncodeTransform<'a> {
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
                    .get(&encoder.0)
                    .unwrap_or_else(|| panic!("Missing type for {}", encoder.0))
                {
                    if let DataType::RemoteObj { all, .. } = &**dt {
                        device_vars.clear();
                        for (var, _) in all.iter() {
                            device_vars.push((format!("{}::{var}", encoder.0), encoder.1.clone()));
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
                dests.process(|(record, rec_tag)| {
                    let dt = self.data_types;
                    let record_type = dt.get(record).unwrap();
                    let rt = enum_cast!(DataType::Record, record_type);
                    let mut v: Vec<_> = rt
                        .iter()
                        .map(|(name, _)| (format!("{record}::{name}"), rec_tag.clone()))
                        .collect();
                    // sort so that the dest order matches the src order
                    v.sort_by(|(a, _), (b, _)| a.cmp(b));
                    v
                });
                srcs.process(|src| {
                    let dt = self.data_types;
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
                self.expand_annotations(annot);
            }
            HirInstr::Stmt(_) => (),
        }
    }

    type Dir = Forwards;
}

/// Uses type information to insert device variables into the `BeginEncoding` operator.
/// Also used type information to insert variables into the `Sync` operator and
/// expand record arguments.
/// # Arguments
/// * `cfg` - The control flow graph to transform
/// * `data_types` - The data types of the program
/// * `ctx` - The context of the program
/// * `sig_out` - The UNEXPANDED signature of the function
pub fn transform_encode_pass(
    cfg: &mut Cfg,
    data_types: &HashMap<String, DataType>,
    ctx: &Context,
    sig_out: &Vec<FlaggedType>,
) {
    let top = EncodeTransform::top(data_types, ctx, sig_out);
    bft_transform(cfg, &top);
}

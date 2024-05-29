use std::collections::HashMap;
use std::rc::Rc;

use caiman::ir;

use crate::lower::{
    sched_hir::{HirBody, HirFuncCall, HirInstr, Terminator, TripleTag},
    IN_STEM,
};
use crate::parse::ast::{DataType, Flow, Quotient, QuotientReference, SchedTerm, SpecType, Tag};

use super::{Fact, Forwards, RET_VAR};

/// Creates a `none()` tag with the given flow
const fn none_tag(spec_type: SpecType, flow: Flow) -> Tag {
    Tag {
        quot: Some(Quotient::None),
        quot_var: QuotientReference {
            spec_type,
            spec_var: None,
        },
        flow: Some(flow),
    }
}

/// Overrides the unknown information in `tag` with `none()-usable` unless
/// the specified `dtype` is a reference. Then overrrides the spatial information
/// with `none()-save`
fn override_none_usable(
    mut tag: TripleTag,
    dtype: &DataType,
    flags: Option<&ir::BufferFlags>,
) -> TripleTag {
    tag.spatial.override_unknown_info(none_tag(
        SpecType::Spatial,
        if matches!(dtype, DataType::Ref(_)) || matches!(flags, Some(f) if f.storage) {
            Flow::Save
        } else {
            Flow::Usable
        },
    ));
    tag.timeline
        .override_unknown_info(none_tag(SpecType::Timeline, Flow::Usable));
    tag.value
        .override_unknown_info(none_tag(SpecType::Value, Flow::Usable));
    tag
}

/// Overrrides unknown info in `tag` with `none()-save` for spatial,
/// `none()-usable` for timeline, and `none()-dead` for value
fn override_defaults_ref(mut tag: TripleTag) -> TripleTag {
    tag.spatial
        .override_unknown_info(none_tag(SpecType::Spatial, Flow::Save));
    tag.timeline
        .override_unknown_info(none_tag(SpecType::Timeline, Flow::Usable));
    tag.value
        .override_unknown_info(none_tag(SpecType::Value, Flow::Dead));
    tag
}

/// Tag analysis for determining tags
/// Top: empty set
/// Meet: union
#[derive(Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct TagAnalysis {
    tags: HashMap<String, TripleTag>,
    /// For an output fact, thse are the input tags to be overridden
    input_overrides: HashMap<String, TripleTag>,
    data_types: Rc<HashMap<String, DataType>>,
    flags: Rc<HashMap<String, ir::BufferFlags>>,
}

impl PartialEq for TagAnalysis {
    fn eq(&self, other: &Self) -> bool {
        self.tags == other.tags && self.input_overrides == other.input_overrides
    }
}

impl Eq for TagAnalysis {}

impl TagAnalysis {
    /// Constructs a new top element
    pub fn top(
        input: &[(String, TripleTag)],
        out: &[TripleTag],
        data_types: &HashMap<String, DataType>,
        flags: &HashMap<String, ir::BufferFlags>,
    ) -> Self {
        let mut tags = HashMap::new();
        for (out_idx, out_type) in out.iter().enumerate() {
            tags.insert(
                format!("{RET_VAR}{out_idx}"),
                override_none_usable(
                    out_type.clone(),
                    &data_types[&format!("{RET_VAR}{out_idx}")],
                    flags.get(&format!("{RET_VAR}{out_idx}")),
                ),
            );
        }
        for (arg_name, arg_type) in input {
            let mut tg = arg_type.clone();
            if matches!(data_types.get(arg_name), Some(DataType::Ref(_))) {
                // TODO: the flow itself should be able to be a hole
                // the the future, also assume that it's save if the flow is not specified
                // but the quotient is
                tg.spatial
                    .override_unknown_info(none_tag(SpecType::Spatial, Flow::Save));
                if let Some(flow) = &tg.spatial.flow {
                    assert!(
                        *flow == Flow::Save,
                        "Spatial tags for references must be save"
                    );
                }
            }
            let mut in_tg = override_none_usable(tg, &data_types[arg_name], flags.get(arg_name));
            let mut node_tg = in_tg.clone();
            if in_tg.value.quot.is_none() {
                in_tg.value.quot = Some(Quotient::Input);
            }
            if matches!(node_tg.value.quot, Some(Quotient::Input) | None) {
                node_tg.value.quot = Some(Quotient::Node);
            }
            tags.insert(format!("{IN_STEM}{arg_name}"), in_tg);
            tags.insert(arg_name.clone(), node_tg);
        }
        Self {
            tags,
            input_overrides: HashMap::new(),
            data_types: Rc::new(data_types.clone()),
            flags: Rc::new(flags.clone()),
        }
    }

    /// Gets the type of the specified variable or `None` if we have no concrete
    /// information about it.
    pub fn get_tag(&self, var: &str) -> Option<&TripleTag> {
        self.tags.get(var)
    }

    /// Gets the input override for the specified variable or `None` if it was not
    /// overridden
    pub fn get_input_override(&self, var: &str) -> Option<&TripleTag> {
        self.input_overrides.get(var)
    }
}

impl TagAnalysis {
    /// Transfer function for an HIR body statement
    #[allow(clippy::too_many_lines)]
    fn transfer_stmt(&mut self, stmt: &mut HirBody) {
        use std::collections::hash_map::Entry;
        match stmt {
            HirBody::ConstDecl {
                lhs, lhs_tag, rhs, ..
            } => {
                let mut info = lhs_tag.clone();
                if let SchedTerm::Var { name, .. } = rhs {
                    if let Some(rhs_typ) = self.tags.get(name).cloned() {
                        info.value = rhs_typ.value;
                    }
                }
                self.tags.insert(
                    lhs.clone(),
                    override_none_usable(info, &self.data_types[lhs], self.flags.get(lhs)),
                );
            }
            HirBody::VarDecl {
                lhs, lhs_tag, rhs, ..
            } => {
                let mut info = lhs_tag.clone();
                if rhs.is_none() {
                    info.value = none_tag(SpecType::Value, Flow::Dead);
                } else if let Some(SchedTerm::Var { name, .. }) = rhs {
                    // Taken from RefStore
                    if let Some(rhs_typ) = self.tags.get(name).cloned() {
                        info.value = rhs_typ.value;
                    }
                }
                if info.spatial.flow.is_none() {
                    info.spatial.flow = Some(Flow::Save);
                }
                if info.spatial.quot.is_none() {
                    info.spatial.quot = Some(Quotient::None);
                }
                info = override_none_usable(info, &self.data_types[lhs], self.flags.get(lhs));
                self.tags.insert(lhs.clone(), info);
            }
            HirBody::RefStore {
                lhs, lhs_tags, rhs, ..
            } => {
                let t = self.tags.get_mut(lhs).unwrap();
                t.set_specified_info(lhs_tags.clone());
                if let SchedTerm::Var { name, .. } = rhs {
                    // TODO: check this
                    if let Some(rhs_typ) = self.tags.get(name).cloned() {
                        let t = self.tags.get_mut(lhs).unwrap();
                        t.value = rhs_typ.value;
                    }
                }
            }
            HirBody::DeviceCopy {
                dest,
                dest_tag,
                src,
                ..
            } => {
                let t = self.tags.get_mut(dest).unwrap();
                t.set_specified_info(dest_tag.clone());
                if let Some(rhs_typ) = self.tags.get(src).cloned() {
                    let t = self.tags.get_mut(dest).unwrap();
                    t.value = rhs_typ.value;
                }
            }
            HirBody::RefLoad { dest, src, .. } => {
                let mut tag = self
                    .tags
                    .get(src)
                    .cloned()
                    .unwrap_or_else(|| self.input_overrides.get(src).cloned().unwrap());
                // loading makes things usable
                if tag.spatial.flow == Some(Flow::Save) {
                    tag.spatial.flow = Some(Flow::Usable);
                }
                self.tags.insert(
                    dest.clone(),
                    override_none_usable(tag, &self.data_types[dest], self.flags.get(dest)),
                );
            }
            HirBody::Hole(_) => todo!(),
            HirBody::Op { dests, .. } => {
                for (dest, dest_tag) in dests {
                    self.tags.insert(
                        dest.clone(),
                        override_none_usable(
                            dest_tag.clone(),
                            &self.data_types[dest],
                            self.flags.get(dest),
                        ),
                    );
                }
            }
            HirBody::OutAnnotation(_, tags) => {
                for (v, tag) in tags {
                    match self.tags.entry(v.clone()) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().set_specified_info(tag.clone());
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(tag.clone());
                        }
                    }
                }
            }
            HirBody::InAnnotation(_, tags) => {
                for (v, tag) in tags {
                    self.input_overrides.insert(v.clone(), tag.clone());
                    match self.tags.entry(v.clone()) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().set_specified_info(tag.clone());
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(tag.clone());
                        }
                    }
                }
            }
            HirBody::Phi { .. } => panic!("Phi nodes should be eliminated"),
            HirBody::BeginEncoding {
                encoder,
                tags,
                device_vars,
                ..
            } => {
                self.tags.insert(
                    encoder.clone(),
                    override_none_usable(tags.clone(), &DataType::Encoder(None), None),
                );
                for (var, tag) in device_vars {
                    self.tags
                        .insert(var.clone(), override_defaults_ref(tag.clone()));
                }
            }
            HirBody::Submit { dest, tags, .. } => {
                self.tags.insert(
                    dest.clone(),
                    override_none_usable(tags.clone(), &DataType::Fence(None), None),
                );
            }
            HirBody::Sync { dests, .. } => {
                for (dest, dest_tag) in dests.processed() {
                    self.tags.insert(
                        dest.clone(),
                        override_none_usable(
                            dest_tag.clone(),
                            &self.data_types[dest],
                            self.flags.get(dest),
                        ),
                    );
                }
            }
            HirBody::EncodeDo { dests, .. } => {
                for (dest, dest_tag) in dests {
                    let t = self.tags.get_mut(dest).unwrap();
                    t.set_specified_info(dest_tag.clone());
                    // TODO: check this
                    t.value.flow = Some(Flow::Usable);
                }
            }
        }
    }
}

/// Determines if there is a conflict between the quotient or flow of the
/// value, spatial, or timeline parts of two tags. This does not check the
/// quotient node names.
///
/// # Returns
/// `true` if there is a conflict, `false` otherwise
fn tag_conflict(t: &TripleTag, other: &TripleTag) -> bool {
    matches!((t.value.quot, other.value.quot), (Some(x), Some(y)) if  x != y)
        || matches!((t.spatial.quot, other.spatial.quot), (Some(x), Some(y)) if  x != y)
        || matches!((t.timeline.quot, other.timeline.quot), (Some(x), Some(y)) if  x != y)
        || matches!((t.value.flow, other.value.flow), (Some(x), Some(y)) if  x != y)
        || matches!((t.spatial.flow, other.spatial.flow), (Some(x), Some(y)) if  x != y)
        || matches!((t.timeline.flow, other.timeline.flow), (Some(x), Some(y)) if  x != y)
}

impl Fact for TagAnalysis {
    fn meet(mut self, other: &Self) -> Self {
        for (k, v) in &other.tags {
            use std::collections::hash_map::Entry;
            let contains = self.tags.contains_key(&format!("{IN_STEM}{k}"));
            match self.tags.entry(k.to_string()) {
                Entry::Occupied(mut old_v) => {
                    if old_v.get() != v {
                        old_v.get_mut().override_unknown_info(v.clone());
                        if tag_conflict(old_v.get(), v) {
                            // TODO: the problem is that _out is used to identify the
                            // return value, which might change types in the last
                            // funclet. To avoid overriding the final output type,
                            // we don't do anything when it meets with a different value

                            // we allow conflicts with outputs, and inputs
                            assert!(
                                k.starts_with("_out") || k.starts_with(IN_STEM) || contains,
                                "Unexpected tag conflict with {k}\n{:#?} != {v:#?}",
                                old_v.get(),
                            );
                        }
                        // TODO: we assume quotient node names are solved and don't worry
                        // about those conflicts
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(v.clone());
                }
            }
        }
        self
    }

    fn transfer_instr(&mut self, stmt: HirInstr<'_>, _: usize) {
        match stmt {
            HirInstr::Tail(Terminator::Select { dests, tag, .. }) => {
                tag.override_unknown_info(TripleTag::new_none_usable());
                for (dest, dest_tags) in dests {
                    self.tags.insert(
                        dest.clone(),
                        override_none_usable(
                            dest_tags.clone(),
                            &self.data_types[dest],
                            self.flags.get(dest),
                        ),
                    );
                }
            }
            HirInstr::Tail(Terminator::CaptureCall {
                dests,
                captures,
                call: HirFuncCall { tag, .. },
                ..
            }) => {
                tag.override_unknown_info(TripleTag::new_none_usable());
                for (dest, dest_tags) in dests {
                    self.tags.insert(
                        dest.clone(),
                        override_none_usable(
                            dest_tags.clone(),
                            &self.data_types[dest],
                            self.flags.get(dest),
                        ),
                    );
                }
                for cap in captures.iter() {
                    assert!(
                        self.tags.contains_key(cap),
                        "Capture {cap} is missing a tag",
                    );
                }
            }
            HirInstr::Tail(Terminator::Return { dests, rets, .. }) => {
                assert_eq!(dests.len(), rets.len());
                for ((idx, _), out) in dests.iter().zip(rets.iter()) {
                    let tag = self.tags.get(out).cloned().unwrap();
                    self.tags.insert(
                        idx.clone(),
                        override_none_usable(tag, &self.data_types[idx], self.flags.get(out)),
                    );
                }
            }
            HirInstr::Tail(
                Terminator::None(..)
                | Terminator::Next(..)
                | Terminator::FinalReturn(..)
                | Terminator::Yield(..),
            ) => (),
            HirInstr::Tail(Terminator::Call(..)) => panic!("Call should be eliminated"),
            HirInstr::Stmt(stmt) => self.transfer_stmt(stmt),
        }
    }

    type Dir = Forwards;
}

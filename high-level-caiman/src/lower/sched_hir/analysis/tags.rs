//! Deduces flows and aggregates deduced quotient information via a dataflow pass.
//! This pass is designed to work with `bft_transform`, where we don't meet with
//! top and a block is only analyzed once.
//!
//! This pass determines when a quotient should be input or node and
//! determines flow. At the start of the first block, all tags are
//! input quotients. At the end of the start block, all tags become
//! node quotients.

use std::collections::HashMap;
use std::rc::Rc;

use caiman::explication::Hole;
use caiman::ir;

use crate::error::{hir_to_source_name, Info, LocalError};
use crate::lower::sched_hir::cfg::FINAL_BLOCK_ID;
use crate::lower::sched_hir::HirTerm;
use crate::lower::sched_hir::{
    cfg::START_BLOCK_ID, HirBody, HirFuncCall, HirInstr, Terminator, TripleTag,
};
use crate::parse::ast::{DataType, Flow, Quotient, QuotientReference, SpecType, Tag};
use crate::type_error;

use super::{Fact, Forwards, TransferData, RET_VAR};

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

/// Overrides the unknown information in `tag` with `none()-usable`
/// and overrrides the spatial information with `none()-save`
fn override_none_usable_ref(mut tag: TripleTag) -> TripleTag {
    tag.spatial
        .override_unknown_info(none_tag(SpecType::Spatial, Flow::Save));
    tag.timeline
        .override_unknown_info(none_tag(SpecType::Timeline, Flow::Usable));
    tag.value
        .override_unknown_info(none_tag(SpecType::Value, Flow::Usable));
    tag
}

/// Overrrides unknown info in `tag` with `none()-save` for spatial,
/// `none()-usable` for timeline, and `none()-dead` for value if `override_dead` is true
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
#[derive(Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct FlowAnalysis {
    tags: HashMap<String, TripleTag>,
    /// For an output fact, thse are the input tags to be overridden.
    /// Input overrides are not carried over between blocks
    input_overrides: HashMap<String, TripleTag>,
    data_types: Rc<HashMap<String, DataType>>,
    flags: Rc<HashMap<String, ir::BufferFlags>>,
    /// The tags that are added at the start of the final basic block
    out_tags: Option<Rc<HashMap<String, TripleTag>>>,
    /// The current block this fact is a part of. This is used to
    /// perform operations once per block
    block: Option<usize>,
}

impl PartialEq for FlowAnalysis {
    fn eq(&self, other: &Self) -> bool {
        self.tags == other.tags && self.input_overrides == other.input_overrides
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for FlowAnalysis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TagAnalysis")
            .field("tags", &self.tags)
            .field("input_overrides", &self.input_overrides)
            .finish()
    }
}

impl Eq for FlowAnalysis {}

impl FlowAnalysis {
    /// Constructs tags for special input arguments
    /// # Arguments
    /// * `tags` - The tags to insert into
    /// * `data_types` - The data types of the input arguments
    /// * `input` - The input arguments
    /// * `num_dims` - The number of dimensional template arguments
    fn get_input_tags(
        tags: &mut HashMap<String, TripleTag>,
        data_types: &HashMap<String, DataType>,
        flags: &HashMap<String, ir::BufferFlags>,
        input: &[(String, TripleTag)],
        num_dims: usize,
    ) {
        // input tags all start off as Input quotient and are transformed to node
        // quotients after the first block
        for i in 0..num_dims {
            let mut t = TripleTag::new_none_usable();
            t.value.quot = Some(Quotient::Input);
            t.value.quot_var.spec_var = Some(format!("_dim{i}"));
            tags.insert(format!("_dim{i}"), t.clone());
        }
        for (arg_name, arg_type) in input {
            let mut tg = arg_type.clone();
            if matches!(data_types.get(arg_name), Some(DataType::Ref(_))) {
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
            if in_tg.value.quot.is_none() {
                in_tg.value.quot = Some(Quotient::Input);
            }
            tags.insert(arg_name.clone(), in_tg);
        }
    }
    /// Constructs a new top element
    pub fn top(
        input: &[(String, TripleTag)],
        out: &[TripleTag],
        data_types: &HashMap<String, DataType>,
        flags: &HashMap<String, ir::BufferFlags>,
        num_dims: usize,
    ) -> Self {
        // create tags for outputs
        let mut out_tags = HashMap::new();
        for (out_idx, out_type) in out.iter().enumerate() {
            out_tags.insert(format!("{RET_VAR}{out_idx}"), out_type.clone());
        }
        let mut tags = HashMap::new();
        Self::get_input_tags(&mut tags, data_types, flags, input, num_dims);
        Self {
            tags,
            input_overrides: HashMap::new(),
            data_types: Rc::new(data_types.clone()),
            flags: Rc::new(flags.clone()),
            out_tags: Some(Rc::new(out_tags)),
            block: None,
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

impl FlowAnalysis {
    /// Special processing for the final blocks
    fn special_process_block(&mut self, block_id: usize) {
        use std::collections::hash_map::Entry;
        if block_id == FINAL_BLOCK_ID {
            // the final block adds in the output tags
            // output tags are defined in the final return instruction,
            // but we set the new types as the types of these will
            // change in the final basic block
            if let Some(out_tags) = self.out_tags.take() {
                for (k, v) in out_tags.iter() {
                    match self.tags.entry(k.clone()) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().set_specified_info(v.clone());
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(v.clone());
                        }
                    }
                }
            }
        }
        if self.block != Some(block_id) {
            // input overrides only apply to the block they are a part of
            self.input_overrides.clear();
            self.block = Some(block_id);
        }
    }
    /// Transfer function for an HIR body statement
    #[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
    fn transfer_stmt(&mut self, stmt: &mut HirBody) {
        use std::collections::hash_map::Entry;
        match stmt {
            HirBody::ConstDecl {
                lhs, lhs_tag, rhs, ..
            } => {
                let mut info = lhs_tag.clone();
                if let HirTerm::Var { name, .. } = rhs {
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
                } else if let Some(HirTerm::Var { name, .. }) = rhs {
                    // Taken from RefStore
                    if let Some(rhs_typ) = self.tags.get(name).cloned() {
                        if rhs_typ.value.flow.is_none() {
                            info.value.flow = Some(Flow::Usable);
                        }
                        info.value.set_specified_info(rhs_typ.value);
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
                if let HirTerm::Var { name, .. } = rhs {
                    // TODO: check this
                    if let Some(rhs_typ) = self.tags.get(name).cloned() {
                        let t = self.tags.get_mut(lhs).unwrap();
                        if rhs_typ.value.flow.is_none() {
                            t.value.flow = Some(Flow::Usable);
                        }
                        t.value.set_specified_info(rhs_typ.value);
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
                if let HirTerm::Var { name: src, .. } = src {
                    if let Some(rhs_typ) = self.tags.get(src).cloned() {
                        let t = self.tags.get_mut(dest).unwrap();
                        if rhs_typ.value.flow.is_none() {
                            t.value.flow = Some(Flow::Usable);
                        }
                        t.value.set_specified_info(rhs_typ.value);
                    }
                } else {
                    let t = self.tags.get_mut(dest).unwrap();
                    t.value.flow = Some(Flow::Usable);
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
            HirBody::Hole {
                dests, initialized, ..
            } => {
                for (dest, tag) in dests.iter() {
                    match self.tags.entry(dest.clone()) {
                        Entry::Occupied(mut e) => {
                            e.get_mut().set_specified_info(tag.clone());
                        }
                        Entry::Vacant(e) => {
                            e.insert(tag.clone());
                        }
                    }
                    let dest_tag = self.tags[dest].clone().retain(&[SpecType::Timeline]);
                    match self.data_types.get(dest) {
                        Some(DataType::Encoder(Some(ro))) => {
                            if let DataType::RemoteObj { all, .. } = &**ro {
                                for (e, _) in all {
                                    let full_name = format!("{dest}::{e}");
                                    match self.tags.entry(full_name.clone()) {
                                        Entry::Occupied(mut e) => {
                                            e.get_mut().override_unknown_info(
                                                override_defaults_ref(dest_tag.clone()),
                                            );
                                        }
                                        Entry::Vacant(e) => {
                                            e.insert(override_defaults_ref(dest_tag.clone()));
                                        }
                                    }
                                }
                            }
                        }
                        Some(DataType::Fence(Some(ro))) => {
                            if let DataType::RemoteObj { all, .. } = &**ro {
                                for (e, _) in all {
                                    match self.tags.entry(format!("{dest}::{e}")) {
                                        Entry::Occupied(mut e) => {
                                            e.get_mut().override_unknown_info(
                                                override_none_usable_ref(dest_tag.clone()),
                                            );
                                        }
                                        Entry::Vacant(e) => {
                                            e.insert(override_none_usable_ref(dest_tag.clone()));
                                        }
                                    }
                                }
                            }
                        }
                        Some(DataType::Record(all)) => {
                            for (e, t) in all {
                                match self.tags.entry(format!("{dest}::{e}")) {
                                    Entry::Occupied(mut e) => e.get_mut().override_unknown_info(
                                        override_none_usable(dest_tag.clone(), t, None),
                                    ),
                                    Entry::Vacant(e) => {
                                        e.insert(override_none_usable(dest_tag.clone(), t, None));
                                    }
                                }
                            }
                        }
                        _ => (),
                    }
                }
                for (dest, _) in dests {
                    self.tags.insert(
                        dest.clone(),
                        override_none_usable(
                            self.tags[dest].clone(),
                            self.data_types.get(dest).unwrap(),
                            self.flags.get(dest),
                        ),
                    );
                }
                for init in initialized.iter() {
                    self.tags
                        .entry(init.clone())
                        .or_insert_with(TripleTag::new_none_usable)
                        .value
                        .flow = Some(Flow::Usable);
                }
            }
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
                    match self.input_overrides.entry(v.clone()) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().set_specified_info(tag.clone());
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(tag.clone());
                        }
                    }
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
                device_vars,
                tags,
                ..
            } => {
                tags.override_unknown_info(TripleTag::new_none_usable());
                self.tags.insert(
                    encoder.0.clone(),
                    override_none_usable(encoder.1.clone(), &DataType::Encoder(None), None),
                );
                for (var, tag) in device_vars {
                    self.tags
                        .insert(var.clone(), override_defaults_ref(tag.clone()));
                }
            }
            HirBody::Submit {
                dest, tags, src, ..
            } => {
                self.tags.insert(
                    dest.clone(),
                    override_none_usable(tags.clone(), &DataType::Fence(None), None),
                );
                if let Some(DataType::Fence(Some(t))) = self.data_types.get(dest) {
                    if let DataType::RemoteObj { all, .. } = &**t {
                        for (v, _) in all {
                            let t = self.tags.get_mut(&format!("{src}::{v}")).unwrap();
                            t.set_specified_info(tags.clone());
                        }
                    }
                }
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
                    // TODO: is it always usable?
                    t.value.flow = Some(Flow::Usable);
                }
            }
        }
    }

    /// Performs tag analysis on the block terminator
    fn transfer_tail(&mut self, tail: &mut Terminator, block_id: usize) -> Result<(), LocalError> {
        match tail {
            Terminator::Select { dests, tag, .. } => {
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
            Terminator::CaptureCall {
                dests,
                captures,
                call: HirFuncCall { tag, info, .. },
                ..
            } => {
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
                    if !self.tags.contains_key(cap) {
                        return Err(type_error!(
                            *info,
                            "Captured variable '{}' requires a tag annotation",
                            hir_to_source_name(cap)
                        ));
                    }
                }
            }
            Terminator::Return { dests, rets, .. } => {
                assert_eq!(dests.len(), rets.len());
                for ((dest_name, dest_tag), out) in dests.iter().zip(rets.iter()) {
                    if let Hole::Filled(out) = out {
                        let tag = self.tags.get(out).cloned().unwrap();
                        self.tags.insert(
                            dest_name.clone(),
                            override_none_usable(
                                tag,
                                &self.data_types[dest_name],
                                self.flags.get(out),
                            ),
                        );
                    } else {
                        self.tags.insert(
                            dest_name.clone(),
                            override_none_usable(
                                dest_tag.clone(),
                                &self.data_types[dest_name],
                                self.flags.get(dest_name),
                            ),
                        );
                    }
                }
            }

            Terminator::None(..)
            | Terminator::Next(..)
            | Terminator::FinalReturn(..)
            | Terminator::Yield(..) => (),
            Terminator::Call(..) => panic!("Call should be eliminated"),
        }
        // treat the end of the first block like a special "de-inputifier"
        // where all input quotients become node quotients
        if block_id == START_BLOCK_ID {
            for t in self.tags.values_mut() {
                *t = input_to_node(t.clone());
            }
        }
        Ok(())
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

/// Converts all input tags to node tags
fn input_to_node(mut t: TripleTag) -> TripleTag {
    if t.value.quot == Some(Quotient::Input) {
        t.value.quot = Some(Quotient::Node);
    }
    if t.timeline.quot == Some(Quotient::Input) {
        t.timeline.quot = Some(Quotient::Node);
    }
    if t.spatial.quot == Some(Quotient::Input) {
        t.spatial.quot = Some(Quotient::Node);
    }
    t
}

impl Fact for FlowAnalysis {
    fn meet(mut self, other: &Self, info: Info) -> Result<Self, LocalError> {
        for (k, v) in &other.tags {
            use std::collections::hash_map::Entry;
            match self.tags.entry(k.to_string()) {
                Entry::Occupied(mut old_v) => {
                    if old_v.get() != v {
                        old_v.get_mut().override_unknown_info(v.clone());
                        if tag_conflict(old_v.get(), v) {
                            return Err(type_error!(
                                info,
                                "Flow mismatch when merging control flow paths for '{}'\n{:#?} != {v:#?}",
                                hir_to_source_name(k),
                                old_v.get()
                            ));
                        }
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(v.clone());
                }
            }
        }
        Ok(self)
    }

    fn transfer_instr(&mut self, stmt: HirInstr<'_>, data: TransferData) -> Result<(), LocalError> {
        self.special_process_block(data.block_id);
        match stmt {
            HirInstr::Tail(t) => self.transfer_tail(t, data.block_id)?,
            HirInstr::Stmt(stmt) => self.transfer_stmt(stmt),
        }
        Ok(())
    }

    type Dir = Forwards;
}

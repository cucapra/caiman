use caiman::{assembly::ast as asm, ir};
use std::collections::HashMap;
use std::rc::Rc;

use crate::lower::lower_schedule::tag_to_tag;
use crate::lower::sched_hir::{HirBody, HirInstr, Specs, Terminator, TripleTag};
use crate::parse::ast::SchedTerm;
use crate::typing::SpecType;

use super::{Fact, Forwards, RET_VAR};

/// Assembly-level type information
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagInfo {
    pub value: Option<asm::Tag>,
    pub spatial: Option<asm::Tag>,
    pub timeline: Option<asm::Tag>,
    specs: Rc<Specs>,
}

impl From<&TripleTag> for TagInfo {
    fn from(t: &TripleTag) -> Self {
        Self {
            value: t.value.as_ref().map(tag_to_tag),
            spatial: t.spatial.as_ref().map(tag_to_tag),
            timeline: t.timeline.as_ref().map(tag_to_tag),
            specs: t.specs.clone(),
        }
    }
}

impl From<&mut TripleTag> for TagInfo {
    fn from(t: &mut TripleTag) -> Self {
        From::from(&*t)
    }
}

impl From<TripleTag> for TagInfo {
    fn from(t: TripleTag) -> Self {
        From::from(&t)
    }
}

/// Creates a tag with a none quotient for the given spec and flow
fn none_tag(spec_name: &asm::FuncletId, flow: ir::Flow) -> asm::Tag {
    asm::Tag {
        quot: asm::Quotient::None(Some(asm::RemoteNodeId {
            funclet: Some(spec_name.clone()),
            node: None,
        })),
        flow,
    }
}

impl TagInfo {
    /// Overwrites all of this type info with the tags from `other`. If
    /// `other` does not specify a tag, the tag will NOT be updated.
    pub fn update(&mut self, other: &TripleTag) {
        // TODO: re-evaluate this approach
        if let Some(value) = &other.value {
            self.value = Some(tag_to_tag(value));
        }
        if let Some(spatial) = &other.spatial {
            self.spatial = Some(tag_to_tag(spatial));
        }
        if let Some(timeline) = &other.timeline {
            self.timeline = Some(tag_to_tag(timeline));
        }
    }

    pub fn update_info(&mut self, other: Self) {
        if let Some(value) = other.value {
            self.value = Some(value);
        }
        if let Some(spatial) = other.spatial {
            self.spatial = Some(spatial);
        }
        if let Some(timeline) = other.timeline {
            self.timeline = Some(timeline);
        }
    }

    /// Returns the tag vector for this type. Any unspecified tags will be
    /// assumed to be `none()-usable`
    pub fn tags_vec_default(self) -> Vec<asm::Tag> {
        vec![
            self.value
                .unwrap_or_else(|| none_tag(&self.specs.value, ir::Flow::Usable)),
            self.spatial
                .unwrap_or_else(|| none_tag(&self.specs.spatial, ir::Flow::Usable)),
            self.timeline
                .unwrap_or_else(|| none_tag(&self.specs.timeline, ir::Flow::Usable)),
        ]
    }

    /// Returns the indexed tag vector for this type. Any unspecified tags will be
    /// assumed to be `none()-usable`
    pub fn tag_info_default(self) -> Self {
        Self {
            value: self
                .value
                .or_else(|| Some(none_tag(&self.specs.value, ir::Flow::Usable))),
            spatial: self
                .spatial
                .or_else(|| Some(none_tag(&self.specs.spatial, ir::Flow::Usable))),
            timeline: self
                .timeline
                .or_else(|| Some(none_tag(&self.specs.timeline, ir::Flow::Usable))),
            specs: self.specs,
        }
    }

    /// Returns the default tag for the specified specifcation type.
    /// The default tag is `none()-usable`
    pub fn default_tag(&self, spec_type: SpecType) -> asm::Tag {
        match spec_type {
            SpecType::Value => none_tag(&self.specs.value, ir::Flow::Usable),
            SpecType::Spatial => none_tag(&self.specs.spatial, ir::Flow::Usable),
            SpecType::Timeline => none_tag(&self.specs.timeline, ir::Flow::Usable),
        }
    }
}

/// Tag analysis for determining tags
/// Top: empty set
/// Meet: union
#[derive(Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct TagAnalysis {
    tags: HashMap<String, TagInfo>,
    specs: Rc<Specs>,
    /// For an output fact, thse are the input tags to be overridden
    input_overrides: HashMap<String, TagInfo>,
}

impl PartialEq for TagAnalysis {
    fn eq(&self, other: &Self) -> bool {
        self.tags == other.tags && self.input_overrides == other.input_overrides
    }
}

impl Eq for TagAnalysis {}

impl TagAnalysis {
    /// Constructs a new top element
    pub fn top(specs: &Specs, input: &[(String, TripleTag)], out: &[TripleTag]) -> Self {
        let mut tags = HashMap::new();
        for (out_idx, out_type) in out.iter().enumerate() {
            tags.insert(format!("{RET_VAR}{out_idx}"), TagInfo::from(out_type));
        }
        for (arg_name, arg_type) in input {
            tags.insert(arg_name.clone(), TagInfo::from(arg_type));
        }
        Self {
            tags,
            specs: Rc::new(specs.clone()),
            input_overrides: HashMap::new(),
        }
    }

    /// Gets the type of the specified variable or `None` if we have no concrete
    /// information about it.
    pub fn get_tag(&self, var: &str) -> Option<&TagInfo> {
        self.tags.get(var)
    }

    /// Gets the input override for the specified variable or `None` if it was not
    /// overridden
    pub fn get_input_override(&self, var: &str) -> Option<&TagInfo> {
        self.input_overrides.get(var)
    }
}

/// Gets the remote node id of `q`
#[allow(dead_code)]
const fn remote_node_id(q: &asm::Quotient) -> &asm::Hole<asm::RemoteNodeId> {
    match q {
        asm::Quotient::None(r)
        | asm::Quotient::Node(r)
        | asm::Quotient::Input(r)
        | asm::Quotient::Output(r) => r,
    }
}

/// Sets the remote node id of `q` to `id`
#[allow(dead_code)]
fn set_remote_node_id(q: &mut asm::Quotient, id: asm::Hole<asm::RemoteNodeId>) {
    match q {
        asm::Quotient::None(r)
        | asm::Quotient::Node(r)
        | asm::Quotient::Input(r)
        | asm::Quotient::Output(r) => *r = id,
    }
}

impl TagAnalysis {
    /// Transfer function for an HIR body statement
    fn transfer_stmt(&mut self, stmt: &mut HirBody) {
        use std::collections::hash_map::Entry;
        match stmt {
            HirBody::ConstDecl { lhs, lhs_tag, .. } => {
                self.tags.insert(lhs.clone(), TagInfo::from(lhs_tag));
            }
            HirBody::VarDecl {
                lhs, lhs_tag, rhs, ..
            } => {
                let mut info = TagInfo::from(lhs_tag);
                if rhs.is_none() {
                    if let Some(val) = info.value.as_mut() {
                        val.flow = ir::Flow::Dead;
                    }
                }
                if info.spatial.is_none() {
                    info.spatial = Some(none_tag(&self.specs.spatial, ir::Flow::Save));
                }
                self.tags.insert(lhs.clone(), info);
            }
            HirBody::RefStore {
                lhs, lhs_tags, rhs, ..
            } => {
                // let quot = self.value_quotient(rhs);
                if lhs_tags.is_any_specified() {
                    let t = self.tags.get_mut(lhs).unwrap();
                    t.update(lhs_tags);
                } else if let SchedTerm::Var { name, .. } = rhs {
                    // TODO: this is probably not what we want to do
                    if let Some(rhs_typ) = self.tags.get(name).cloned() {
                        let t = self.tags.get_mut(lhs).unwrap();
                        t.value = rhs_typ.value;
                    }
                }
                // set_remote_node_id(&mut quot, remote_node_id(&t.value.quot).clone());
                // t.value.quot = quot;
            }
            HirBody::RefLoad { dest, src, .. } => {
                let tag = self
                    .tags
                    .get(src)
                    .cloned()
                    .unwrap_or_else(|| self.input_overrides.get(src).cloned().unwrap());
                self.tags.insert(dest.clone(), tag);
            }
            HirBody::Hole(_) => todo!(),
            HirBody::Op { dest, dest_tag, .. } => {
                self.tags.insert(dest.clone(), TagInfo::from(dest_tag));
            }
            HirBody::OutAnnotation(_, tags) => {
                for (v, tag) in tags {
                    match self.tags.entry(v.clone()) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().update(tag);
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(TagInfo::from(tag));
                        }
                    }
                }
            }
            HirBody::InAnnotation(_, tags) => {
                for (v, tag) in tags {
                    self.input_overrides
                        .insert(v.clone(), TagInfo::from(tag.clone()));
                    match self.tags.entry(v.clone()) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().update(tag);
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(TagInfo::from(tag));
                        }
                    }
                }
            }
            HirBody::Phi { .. } => panic!("Phi nodes should be eliminated"),
        }
    }
}

// fn meet_tag_info(a: &mut TagInfo, b: &TagInfo) {}

// fn meet_tag(a: &mut Option<asm::Tag>, b: &asm::Tag) {}

impl Fact for TagAnalysis {
    fn meet(mut self, other: &Self) -> Self {
        for (k, v) in &other.tags {
            use std::collections::hash_map::Entry;
            match self.tags.entry(k.to_string()) {
                Entry::Occupied(old_v) => {
                    if old_v.get() != v {
                        // We can't infer the tag, require it to be specified
                        old_v.remove_entry();
                    }
                    // assert_eq!(old_v.get(), v, "Duplicate key {k} with unequal values");
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
            HirInstr::Tail(Terminator::Call(dests, _) | Terminator::Select { dests, .. }) => {
                for (dest, dest_tags) in dests {
                    self.tags.insert(dest.clone(), TagInfo::from(dest_tags));
                }
            }
            HirInstr::Tail(Terminator::CaptureCall {
                dests, captures, ..
            }) => {
                for (dest, dest_tags) in dests {
                    self.tags.insert(dest.clone(), TagInfo::from(dest_tags));
                }
                for cap in captures.iter() {
                    assert!(
                        self.tags.contains_key(cap),
                        "Capture {cap} is missing a tag",
                    );
                }
            }
            HirInstr::Tail(Terminator::Return { dests, rets }) => {
                assert_eq!(dests.len(), rets.len());
                for ((idx, _), out) in dests.iter().zip(rets.iter()) {
                    let tag = self.tags.get(out).cloned().unwrap();
                    self.tags.insert(idx.clone(), tag);
                }
            }
            HirInstr::Tail(
                Terminator::None | Terminator::Next(..) | Terminator::FinalReturn(_),
            ) => (),
            HirInstr::Stmt(stmt) => self.transfer_stmt(stmt),
        }
    }

    type Dir = Forwards;
}

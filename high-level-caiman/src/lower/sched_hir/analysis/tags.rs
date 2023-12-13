use caiman::{assembly::ast as asm, ir};
use std::collections::HashMap;
use std::rc::Rc;

use crate::lower::global_context::SpecType;
use crate::lower::lower_schedule::tag_to_tag;
use crate::lower::sched_hir::{HirBody, HirInstr, Specs, Terminator};
use crate::parse::ast::{FullType, SchedTerm, Tag, Tags};

use super::{Fact, Forwards, RET_VAR};

/// Assembly-level type information
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagInfo {
    pub value: Option<asm::Tag>,
    pub spatial: Option<asm::Tag>,
    pub timeline: Option<asm::Tag>,
    specs: Specs,
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
    /// Constructs a `TagInfo` from an AST `FullType`.
    pub fn from(t: &FullType, specs: &Specs) -> Self {
        Self::from_tags(&t.tags, specs)
    }

    /// Constructs a `TagInfo` from a vector of tags
    pub fn from_tags(t: &[Tag], specs: &Specs) -> Self {
        let mut value = None;
        let mut spatial = None;
        let mut timeline = None;
        for tag in t {
            match specs.get_spec_type(&tag.quot_var.as_ref().unwrap().spec_name) {
                Some(SpecType::Value) => value = Some(tag_to_tag(tag)),
                Some(SpecType::Spatial) => spatial = Some(tag_to_tag(tag)),
                Some(SpecType::Timeline) => timeline = Some(tag_to_tag(tag)),
                None => panic!("Unknown spec"),
            }
        }
        Self {
            value,
            timeline,
            spatial,
            specs: specs.clone(),
        }
    }

    /// Constructs a `TagInfo` from an optional vector of tags. If `t` is `None`
    pub fn from_maybe_tags(t: &Option<Tags>, specs: &Specs) -> Self {
        t.as_ref().map_or_else(
            || Self {
                value: None,
                timeline: None,
                spatial: None,
                specs: specs.clone(),
            },
            |t| Self::from_tags(t, specs),
        )
    }

    /// Overwrites all of this type info with the tags from `other`. If
    /// `other` does not specify a tag, the tag will NOT be updated.
    pub fn update(&mut self, specs: &Specs, other: &Tags) {
        let mut value = None;
        let mut spatial = None;
        let mut timeline = None;
        for tag in other {
            match specs.get_spec_type(&tag.quot_var.as_ref().unwrap().spec_name) {
                Some(SpecType::Value) => value = Some(tag_to_tag(tag)),
                Some(SpecType::Spatial) => spatial = Some(tag_to_tag(tag)),
                Some(SpecType::Timeline) => timeline = Some(tag_to_tag(tag)),
                None => panic!("Unknwon spec"),
            }
        }
        // TODO: re-evaluate this approach
        if let Some(value) = value {
            self.value = Some(value);
        }
        if let Some(spatial) = spatial {
            self.spatial = Some(spatial);
        }
        if let Some(timeline) = timeline {
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
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
pub struct TagAnalysis {
    tags: HashMap<String, TagInfo>,
    specs: Rc<Specs>,
    /// For an output fact, thse are the input tags to be overridden
    input_overrides: HashMap<String, Vec<Tag>>,
}

impl TagAnalysis {
    /// Constructs a new top element
    pub fn top(specs: &Specs, out: &Option<FullType>) -> Self {
        let mut tags = HashMap::new();
        tags.insert(
            String::from(RET_VAR),
            TagInfo::from(out.as_ref().unwrap(), specs),
        );
        Self {
            tags,
            specs: Rc::new(specs.clone()),
            input_overrides: HashMap::new(),
        }
    }

    /// Gets the type of the specified variable or `None` if we have no concrete
    /// information about it.
    pub fn get_tag(&self, var: &str) -> Option<TagInfo> {
        self.tags.get(var).cloned()
    }

    /// Gets the input override for the specified variable or `None` if it was not
    /// overridden
    pub fn get_input_override(&self, var: &str) -> Option<Vec<Tag>> {
        self.input_overrides.get(var).cloned()
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
                self.tags.insert(
                    lhs.clone(),
                    TagInfo::from(lhs_tag.as_ref().unwrap(), &self.specs),
                );
            }
            HirBody::VarDecl {
                lhs, lhs_tag, rhs, ..
            } => {
                let mut info = TagInfo::from(lhs_tag.as_ref().unwrap(), &self.specs);
                if rhs.is_none() {
                    if let Some(val) = info.value.as_mut() {
                        val.flow = ir::Flow::Dead;
                    }
                    if info.spatial.is_none() {
                        info.spatial = Some(none_tag(&self.specs.spatial, ir::Flow::Save));
                    }
                }
                self.tags.insert(lhs.clone(), info);
            }
            HirBody::RefStore {
                lhs, lhs_tags, rhs, ..
            } => {
                // let quot = self.value_quotient(rhs);
                if let Some(lhs_tags) = lhs_tags {
                    let t = self.tags.get_mut(lhs).unwrap();
                    t.update(&self.specs, lhs_tags);
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
                let tag = self.tags.get(src).cloned().unwrap_or_else(|| {
                    TagInfo::from_tags(self.input_overrides.get(src).unwrap(), &self.specs)
                });
                self.tags.insert(dest.clone(), tag);
            }
            HirBody::Hole(_) => todo!(),
            HirBody::Op { dest, dest_tag, .. } => {
                self.tags.insert(
                    dest.clone(),
                    TagInfo::from(dest_tag.as_ref().unwrap(), &self.specs),
                );
            }
            HirBody::OutAnnotation(_, tags) => {
                for (v, tag) in tags {
                    match self.tags.entry(v.clone()) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().update(&self.specs, tag);
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(TagInfo::from_tags(tag, &self.specs));
                        }
                    }
                }
            }
            HirBody::InAnnotation(_, tags) => {
                for (v, tag) in tags {
                    self.input_overrides.insert(v.clone(), tag.clone());
                }
            }
        }
    }
}

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
            HirInstr::Tail(Terminator::Call(dests, _)) => {
                for (dest, dest_tags) in dests {
                    self.tags.insert(
                        dest.clone(),
                        TagInfo::from_tags(&dest_tags.as_ref().unwrap().tags, &self.specs),
                    );
                }
            }
            HirInstr::Tail(Terminator::CaptureCall {
                dests, captures, ..
            }) => {
                for (dest, dest_tags) in dests {
                    self.tags.insert(
                        dest.clone(),
                        TagInfo::from_tags(&dest_tags.as_ref().unwrap().tags, &self.specs),
                    );
                }
                for cap in captures.iter() {
                    assert!(
                        self.tags.contains_key(cap),
                        "Capture {cap} is missing a tag",
                    );
                }
            }
            HirInstr::Tail(
                Terminator::None
                | Terminator::Next(..)
                | Terminator::FinalReturn
                | Terminator::Select(..)
                | Terminator::Return(..),
            ) => (),
            HirInstr::Stmt(stmt) => self.transfer_stmt(stmt),
        }
    }

    type Dir = Forwards;
}

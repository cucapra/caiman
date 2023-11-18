use caiman::{assembly::ast as asm, ir};
use std::collections::HashMap;
use std::rc::Rc;

use crate::lower::data_type_to_local_type;
use crate::lower::global_context::SpecType;
use crate::lower::hir::{Hir, HirInstr, Specs};
use crate::lower::lower_schedule::{tag_to_quot, tag_to_tag};
use crate::parse::ast::{FullType, SchedTerm};

use super::{Fact, Forwards};

/// Assembly-level type information
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeInfo {
    pub typ: asm::TypeId,
    pub value: asm::Tag,
    pub spatial: asm::Tag,
    pub timeline: asm::Tag,
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

impl TypeInfo {
    /// Constructs a `TypeInfo` from an AST `FullType`. Any unspecified
    /// tags will be assumed to be `none()-usable`
    pub fn from(t: &FullType, specs: &Specs) -> Self {
        let typ = data_type_to_local_type(&t.base.base);
        let mut value = None;
        let mut spatial = None;
        let mut timeline = None;
        for tag in &t.tags {
            match specs.get_spec_type(&tag.quot_var.as_ref().unwrap().spec_name) {
                Some(SpecType::Value) => value = Some(tag_to_tag(tag)),
                Some(SpecType::Spatial) => spatial = Some(tag_to_tag(tag)),
                Some(SpecType::Timeline) => timeline = Some(tag_to_tag(tag)),
                None => panic!("Unknwon spec"),
            }
        }
        Self {
            typ,
            value: value.unwrap_or_else(|| none_tag(&specs.value, ir::Flow::Usable)),
            timeline: timeline.unwrap_or_else(|| none_tag(&specs.timeline, ir::Flow::Usable)),
            spatial: spatial.unwrap_or_else(|| none_tag(&specs.spatial, ir::Flow::Usable)),
        }
    }

    /// Makes the base type of this type info a reference to the existing type
    /// Does not check against references to references
    fn make_ref(mut self) -> Self {
        self.typ = match self.typ {
            asm::TypeId::Local(type_name) => asm::TypeId::Local(format!("&{type_name}")),
            asm::TypeId::FFI(_) => todo!(),
        };
        self
    }
}

/// Tag analysis for determining tags
/// Top: empty set
/// Meet: union
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
pub struct TagAnalysis {
    tags: HashMap<String, TypeInfo>,
    specs: Rc<Specs>,
}

impl TagAnalysis {
    /// Constructs a new top element
    pub fn top(specs: &Specs) -> Self {
        Self {
            tags: HashMap::new(),
            specs: Rc::new(specs.clone()),
        }
    }

    /// Gets the value quotient of a term
    /// # Panics
    /// If the value quotient is not specified in the term and
    /// the term is not a variable with a previously known value quotient
    fn value_quotient(&self, term: &SchedTerm) -> asm::Quotient {
        let tags = term.get_tags();
        if let Some(tags) = tags {
            for t in tags {
                if t.quot_var.as_ref().unwrap().spec_name == self.specs.value.0 {
                    return tag_to_quot(t);
                }
            }
        }
        if let SchedTerm::Var { name, .. } = term {
            return self.tags.get(name).unwrap().value.quot.clone();
        }
        panic!("Quotient not specified nor saved")
    }
}

impl Fact for TagAnalysis {
    fn meet(mut self, other: &Self) -> Self {
        for (k, v) in &other.tags {
            use std::collections::hash_map::Entry;
            match self.tags.entry(k.to_string()) {
                Entry::Occupied(old_v) => assert_eq!(old_v.get(), v),
                Entry::Vacant(entry) => {
                    entry.insert(v.clone());
                }
            }
        }
        self
    }

    fn transfer_instr(&mut self, stmt: HirInstr<'_>) {
        match stmt {
            HirInstr::Tail(..) => (),
            HirInstr::Stmt(Hir::ConstDecl { lhs, lhs_tag, .. }) => {
                self.tags.insert(
                    lhs.clone(),
                    TypeInfo::from(lhs_tag.as_ref().unwrap(), &self.specs),
                );
            }
            HirInstr::Stmt(Hir::VarDecl {
                lhs, lhs_tag, rhs, ..
            }) => {
                let mut info = TypeInfo::from(lhs_tag.as_ref().unwrap(), &self.specs).make_ref();
                if rhs.is_none() {
                    info.value.flow = ir::Flow::Dead;
                    info.spatial.flow = ir::Flow::Save;
                }
                self.tags.insert(lhs.clone(), info);
            }
            HirInstr::Stmt(Hir::Move { lhs, rhs, .. }) => {
                let quot = self.value_quotient(rhs);
                let t = self.tags.get_mut(lhs).unwrap();
                t.value.flow = ir::Flow::Usable;
                t.value.quot = quot;
            }
            HirInstr::Stmt(Hir::Op { .. } | Hir::Hole(_)) => todo!(),
        }
    }

    type Dir = Forwards;
}

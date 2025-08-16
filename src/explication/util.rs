use crate::explication::context::StaticContext;
use crate::explication::expir;
use crate::explication::expir::{
    ExternalFunctionId, FuncletId, FunctionClassId, NodeId, StorageTypeId, TypeId,
};
use crate::explication::Hole;
use crate::ir::ffi;
use crate::stable_vec::StableVec;
use crate::{frontend, ir};
use serde_derive::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Location {
    pub funclet_id: FuncletId,
    pub quot: expir::Quotient,
}

impl Location {
    pub fn new(funclet_id: FuncletId, node_id: NodeId) -> Location {
        Location {
            funclet_id,
            quot: expir::Quotient::Node { node_id },
        }
    }

    // utility function to forcibly return a nodeid
    // if self is an output, looks up the context dependency
    pub fn node_id(&self, context: &StaticContext) -> Option<NodeId> {
        match &self.quot {
            ir::Quotient::Node { node_id } | ir::Quotient::Input { index: node_id } => {
                Some(node_id.clone())
            }
            ir::Quotient::Output { index } => context
                .get_tail_edge_dependencies(&self.funclet_id)
                .get(index.clone())
                .clone()
                .map(|u| u.clone()),
            ir::Quotient::None => None,
        }
    }

    pub fn is_subset_of(&self, other: &Location, context: &StaticContext) -> bool {
        let node_id = self.node_id(context);
        node_id.is_none()
            || (self.funclet_id == other.funclet_id && node_id == other.node_id(context))
    }

    // Converts a location with an `Input` or `Output` to just be `Node`
    pub fn into_node_id(&self, context: &StaticContext) -> Location {
        Location {
            funclet_id: self.funclet_id.clone(),
            quot: match self.node_id(context) {
                None => ir::Quotient::None,
                Some(node_id) => ir::Quotient::Node { node_id },
            },
        }
    }

    // If either Location in a specification is None, returns the other spec
    // If both locations match, returns that location
    // otherwise, returns None
    pub fn intersect(&self, other: &Location) -> Option<Location> {
        if self.funclet_id == other.funclet_id {
            let funclet_id = self.funclet_id.clone();
            match (self.quot, other.quot) {
                (ir::Quotient::None, quot) |
                (quot, ir::Quotient::None) => Some(Location {
                    funclet_id,
                    quot: quot.clone()
                }),
                (ir::Quotient::Node { node_id: n1 }, ir::Quotient::Node { node_id: n2 }) |
                (ir::Quotient::Input { index: n1 }, ir::Quotient::Input { index: n2 }) |
                (ir::Quotient::Output { index: n1 }, ir::Quotient::Output { index: n2 }) => if n1 == n2 {
                    Some(Location {
                        funclet_id,
                        quot: self.quot.clone()
                    })
                } else {
                    None
                },
                _ => None
            }
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocationTriple {
    // we use Option to explicitly mean "don't care"
    // which is distinct from an explicit "Quotient::None"
    pub value: Option<Location>,
    pub timeline: Option<Location>,
    pub spatial: Option<Location>,
}

impl LocationTriple {
    pub fn new() -> LocationTriple {
        LocationTriple {
            value: None,
            timeline: None,
            spatial: None,
        }
    }

    pub fn new_value(value: Location) -> LocationTriple {
        LocationTriple {
            value: Some(value),
            timeline: None,
            spatial: None,
        }
    }
    pub fn new_timeline(timeline: Location) -> LocationTriple {
        LocationTriple {
            value: None,
            timeline: Some(timeline),
            spatial: None,
        }
    }
    pub fn new_spatial(spatial: Location) -> LocationTriple {
        LocationTriple {
            value: None,
            timeline: None,
            spatial: Some(spatial),
        }
    }
    pub fn new_triple_mapped<T>(
        f: T,
        funclet_id: FuncletId,
        node_id: NodeId,
        state: &crate::explication::context::InState,
        context: &StaticContext,
    ) -> LocationTriple
    where
        T: Fn(&expir::FuncletSpec, FuncletId, NodeId, &StaticContext) -> Option<Location>,
    {
        let specs = state.get_funclet_spec_triple(funclet_id, context);
        LocationTriple {
            value: f(specs.0, specs.0.funclet_id_opt.unwrap(), node_id, context),
            timeline: f(specs.1, specs.1.funclet_id_opt.unwrap(), node_id, context),
            spatial: f(specs.2, specs.2.funclet_id_opt.unwrap(), node_id, context),
        }
    }

    // Returns a location triple where we explicitly don't care about Quotient::None
    pub fn triple_ignoring_none(&self) -> LocationTriple {
        let value = match &self.value {
            Some(v) => match v {
                Location {
                    funclet_id,
                    quot: ir::Quotient::None,
                } => None,
                loc => Some(loc.clone()),
            },
            None => None,
        };
        let timeline = match &self.timeline {
            Some(v) => match v {
                Location {
                    funclet_id,
                    quot: ir::Quotient::None,
                } => None,
                loc => Some(loc.clone()),
            },
            None => None,
        };
        let spatial = match &self.spatial {
            Some(v) => match v {
                Location {
                    funclet_id,
                    quot: ir::Quotient::None,
                } => None,
                loc => Some(loc.clone()),
            },
            None => None,
        };
        LocationTriple {
            value,
            timeline,
            spatial,
        }
    }

    // Returns whether this location triple is a subset of the other
    // We define a subset where None < Anything
    // but a non-None location must be equal
    pub fn is_subset_of(&self, other: &LocationTriple, context: &StaticContext) -> bool {
        let value = match (&self.value, &other.value) {
            (Some(loc1), Some(loc2)) => loc1.is_subset_of(loc2, context),
            (Some(loc), None) => loc.quot == expir::Quotient::None,
            _ => true,
        };
        let timeline = match (&self.timeline, &other.timeline) {
            (Some(loc1), Some(loc2)) => loc1.is_subset_of(loc2, context),
            (Some(loc), None) => loc.quot == expir::Quotient::None,
            _ => true,
        };
        let spatial = match (&self.spatial, &other.spatial) {
            (Some(loc1), Some(loc2)) => loc1.is_subset_of(loc2, context),
            (Some(loc), None) => loc.quot == expir::Quotient::None,
            _ => true,
        };
        value && timeline && spatial
    }

    // Converts all locations in this triple from inputs/outputs to nodes
    pub fn into_node_id(&self, context: &StaticContext) -> LocationTriple {
        LocationTriple {
            value: self.value.clone().map(|loc| loc.into_node_id(context)),
            timeline: self.timeline.clone().map(|loc| loc.into_node_id(context)),
            spatial: self.spatial.clone().map(|loc| loc.into_node_id(context)),
        }
    }

    // Returns an intersection if these triples can be intersected
    //   with the given set of specifications
    // If any of the specifications conflict, returns None
    pub fn intersection(&self, other: &LocationTriple) -> Option<LocationTriple> {
        let value = match (&self.value, &other.value) {
            (Some(loc1), Some(loc2)) => loc1.intersect(loc2),
            (None, None) => None,
            (Some(loc), None) | (None, Some(loc)) => Some(loc.clone())
        };
        let timeline = match (&self.timeline, &other.timeline) {
            (Some(loc1), Some(loc2)) => loc1.intersect(loc2),
            (None, None) => None,
            (Some(loc), None) | (None, Some(loc)) => Some(loc.clone())
        };
        let spatial = match (&self.spatial, &other.spatial) {
            (Some(loc1), Some(loc2)) => loc1.intersect(loc2),
            (None, None) => None,
            (Some(loc), None) | (None, Some(loc)) => Some(loc.clone())
        };

        match (value, timeline, spatial) {
            (Some(v), Some(t), Some(s)) => {
                Some(LocationTriple{
                    value: Some(v),
                    timeline: Some(t),
                    spatial: Some(s)
                })
            }
            _ => { None }
        }
    }
}

pub fn find_filled<T>(v: Vec<Hole<T>>) -> Vec<(usize, T)> {
    let mut result = Vec::new();
    for (index, hole) in v.into_iter().enumerate() {
        match hole {
            Hole::Filled(value) => {
                result.push((index, value));
            }
            Hole::Empty => {}
        }
    }
    result
}

pub fn find_filled_hole<T>(h: Hole<Vec<Hole<T>>>) -> Vec<(usize, T)>
where
    T: Clone,
{
    match h {
        Hole::Filled(v) => find_filled(v),
        Hole::Empty => Vec::new(),
    }
}

pub fn assign_or_compare<T>(current: Option<T>, comparison: T) -> Option<T>
where
    T: Eq + core::fmt::Debug,
{
    match current {
        None => Some(comparison),
        Some(value) => {
            assert_eq!(value, comparison);
            Some(value)
        }
    }
}

impl<T> StableVec<T>
where
    T: std::fmt::Debug,
{
    pub fn get_expect(&self, index: usize) -> &T {
        &self.get(index).expect(&format!(
            "Index {} out of bounds for stable vec {:?}",
            index, self
        ))
    }
}

pub fn get_expect_box<T>(data: &Box<[T]>, index: usize) -> &T
where
    T: std::fmt::Debug,
{
    &data.get(index).expect(&format!(
        "Index {} out of bounds for slice {:?}",
        index, data
    ))
}

// used for identifying which spec language to reason about in a given search
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SpecLanguage {
    Value,
    Timeline,
    Spatial,
}

// struct for storing the triple of languages
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpecLanguages {
    pub value: FuncletId,
    pub timeline: FuncletId,
    pub spatial: FuncletId,
}

impl SpecLanguages {
    pub fn get(&self, spec: &SpecLanguage) -> &FuncletId {
        match spec {
            SpecLanguage::Value => &self.value,
            SpecLanguage::Timeline => &self.timeline,
            SpecLanguage::Spatial => &self.spatial,
        }
    }

    pub fn get_mut(&self, spec: &SpecLanguage) -> &FuncletId {
        match spec {
            SpecLanguage::Value => &self.value,
            SpecLanguage::Timeline => &self.timeline,
            SpecLanguage::Spatial => &self.spatial,
        }
    }
}

fn spec_box_read(
    to_read: &Box<[Hole<expir::Tag>]>,
    spec: &expir::FuncletSpec,
    funclet_id: FuncletId,
    node_id: NodeId,
    context: &StaticContext,
) -> Option<Location> {
    let index_error = format!(
        "funclet {} does not have enough arguments for lookup {}",
        context.debug_info.funclet(&funclet_id),
        context.debug_info.node(&funclet_id, node_id)
    );
    match &to_read.get(node_id).expect(&index_error) {
        Hole::Empty => None,
        Hole::Filled(t) => Some(Location {
            funclet_id: spec.funclet_id_opt.unwrap(),
            quot: t.quot.clone(),
        }),
    }
}

pub fn spec_input(
    spec: &expir::FuncletSpec,
    funclet_id: FuncletId,
    node_id: NodeId,
    context: &StaticContext,
) -> Option<Location> {
    spec_box_read(&spec.input_tags, spec, funclet_id, node_id, context)
}

pub fn spec_output(
    spec: &expir::FuncletSpec,
    funclet_id: FuncletId,
    node_id: NodeId,
    context: &StaticContext,
) -> Option<Location> {
    spec_box_read(&spec.output_tags, spec, funclet_id, node_id, context)
}

pub fn get_implicit_time(funclet_id: FuncletId, context: &StaticContext) -> Option<Location> {
    match &context.get_funclet(&funclet_id).spec_binding {
        expir::FuncletSpecBinding::ScheduleExplicit {
            value,
            spatial,
            timeline,
        } => timeline.implicit_in_tag.as_ref().opt().map(|t| Location {
            funclet_id: timeline.funclet_id_opt.unwrap(),
            quot: t.quot.clone(),
        }),
        _ => panic!(
            "Invalid funclet for an implicit time lookup {}, expected a spec funclet",
            context.debug_info.funclet(&funclet_id)
        ),
    }
}

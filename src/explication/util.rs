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
    pub funclet: FuncletId,
    pub node: NodeId,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocationTriple {
    pub value: Option<Location>,
    pub timeline: Option<Location>,
    pub spatial: Option<Location>
}

impl LocationTriple {
    pub fn new_value(value: Location) -> LocationTriple {
        LocationTriple {
            value: Some(value),
            timeline: None,
            spatial: None
        }
    }
    pub fn new_timeline(timeline: Location) -> LocationTriple {
        LocationTriple {
            value: None,
            timeline: Some(timeline),
            spatial: None
        }
    }
    pub fn new_spatial(spatial: Location) -> LocationTriple {
        LocationTriple {
            value: None,
            timeline: None,
            spatial: Some(spatial)
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

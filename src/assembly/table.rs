use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

// Data structure specifically for lowering, intended to go "both ways"
// So you can go from values to indices or the other way around
// Mostly used as the opposite of a StableVec, going from value to index

#[derive(Debug)]
pub struct Table<T>
where
    T: Eq + Hash + Debug + Clone,
{
    values: HashSet<T>,
    indices: Vec<T>,
}

// a Table is basically a vector with no dupes
impl<T> Table<T>
where
    T: Eq + Hash + Debug + Clone,
{
    pub fn new() -> Table<T> {
        Table {
            values: HashSet::new(),
            indices: Vec::new(),
        }
    }

    pub fn contains(&mut self, val: &T) -> bool {
        self.values.contains(val)
    }

    pub fn dummy_push(&mut self, val: T) {
        // Add unnamed element for indexing
        self.indices.push(val);
    }

    pub fn push(&mut self, val: T) {
        let msg = format!("Duplicate add of {:?}", val);
        if !self.try_push(val) {
            panic!(msg)
        }
    }

    pub fn try_push(&mut self, val: T) -> bool {
        self.indices.push(val.clone());
        self.values.insert(val)
    }

    pub fn insert(&mut self, index: usize, val: T) {
        if self.values.contains(&val) {
            panic!("Duplicate add of {:?}", val)
        }
        self.values.insert(val.clone());
        self.indices.insert(index, val);
    }

    pub fn get(&self, val: &T) -> Option<usize> {
        // no need to actually check the Hashset, that's just to avoid dupes
        for item in itertools::enumerate(&self.indices) {
            if item.1 == val {
                return Some(item.0);
            }
        }
        return None;
    }

    pub fn get_index(&self, val: &T) -> Option<usize> {
        self.get(val)
    }

    pub fn get_at_index(&self, index: usize) -> Option<&T> {
        if index >= self.indices.len() {
            None
        } else {
            Some(&self.indices[index])
        }
    }

    pub fn len(&mut self) -> usize {
        return self.indices.len();
    }
}

use caiman::stable_vec::StableVec;
use std::{collections::HashMap, hash::Hash};
use std::cmp::Eq;

pub struct Index<T: Eq + Hash> 
{
    map: HashMap<T, usize>,
    num_elts: usize,
}

impl<T: Eq + Hash> Index<T>
{
    pub fn new() -> Self
    {
        Index { map: HashMap::new(), num_elts: 0 }
    }

    /// Does not insert if t is already in the index
    pub fn insert(&mut self, t: T) 
    {
        if !self.map.contains_key(&t)
        {
            self.map.insert(t, self.num_elts);
            self.num_elts += 1;
        }
    }

    pub fn get(&self, t: &T) -> Option<usize>
    {
        self.map.get(t).map(|u| *u)
    }

    pub fn into_stable_vec(self) -> StableVec<T>
    {
        self.map_into_stable_vec(&|x| x)
    }

    pub fn map_into_stable_vec<U>(self, f: &dyn Fn(T) -> U) -> StableVec<U>
    {
        let mut sv = StableVec::new();
        for (t, _) in self.map.into_iter()
        {
            sv.add(f(t));
        }
        sv
    }

}


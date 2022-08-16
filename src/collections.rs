use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::hash_map::{Entry, HashMap};
use std::collections::BTreeMap;
use std::default::Default;
use std::hash::Hash;
use std::rc::Rc;

enum ShmRecord<K> {
    Scope,
    Insert(Rc<K>),
    Replace(Rc<K>),
}
struct ShmValue<V> {
    value: V,
    depth: usize,
}
pub struct ScopedHashMap<K, V> {
    /// The actual hashmap. Keys are reference countied to avoid potentially costly `clone`-s,
    /// since keys must live in both the hashmap and the `keys` structure.
    hashmap: HashMap<Rc<K>, ShmValue<V>>,
    /// A journal of the actions performed. This allows us to "rewind" when a scope is popped and
    /// undo all the changes we made in the reverse order.
    journal: Vec<ShmRecord<K>>,
    /// A stack of replaced values.
    replaced: Vec<ShmValue<V>>,
    /// Tracks the current depth. This is used to optimize replacing values at the current depth.
    /// The user doesn't expect to recover the old values so they can be discarded.
    depth: usize,
}
impl<K: Eq + Hash, V> ScopedHashMap<K, V> {
    pub fn new() -> Self {
        Self {
            hashmap: HashMap::new(),
            journal: Vec::new(),
            replaced: Vec::new(),
            depth: 0,
        }
    }
    /// Pushes a new scope. The old values will remain available, but may be temporarily
    /// overwritten by new insertions until the scope is popped.
    pub fn push_scope(&mut self) {
        self.journal.push(ShmRecord::Scope);
        self.depth += 1;
    }
    /// Pops the current scope.
    /// # Panics
    /// Panics if no scope has been pushed.
    pub fn pop_scope(&mut self) {
        self.depth -= 1;
        loop {
            match self.journal.pop().expect("no scopes") {
                ShmRecord::Scope => return,
                ShmRecord::Insert(k) => {
                    let removed = self.hashmap.remove(&k);
                    assert!(removed.is_some(), "inserted key not in map");
                }
                ShmRecord::Replace(k) => {
                    let val = self.hashmap.get_mut(&k).expect("replaced key not in map");
                    *val = self.replaced.pop().expect("out of sync w/ journal");
                }
            }
        }
    }
    pub fn get(&self, key: &K) -> Option<&'_ V> {
        self.hashmap.get(key).map(|sv| &sv.value)
    }
    pub fn get_mut(&mut self, key: &K) -> Option<&'_ mut V> {
        self.hashmap.get_mut(key).map(|sv| &mut sv.value)
    }
    pub fn insert(&mut self, key: K, value: V) {
        let key = Rc::new(key);
        let mut value = ShmValue {
            value,
            depth: self.depth,
        };
        match self.hashmap.entry(Rc::clone(&key)) {
            Entry::Occupied(mut existing) => {
                std::mem::swap(&mut value, existing.get_mut());
                // replaced within same depth: no expecation that old val is saved,
                // so no action needs to be taken
                assert!(value.depth <= self.depth, "a scope wasn't cleared");
                if value.depth < self.depth {
                    self.replaced.push(value);
                    self.journal.push(ShmRecord::Replace(key));
                }
            }
            Entry::Vacant(spot) => {
                spot.insert(value);
                self.journal.push(ShmRecord::Insert(key));
            }
        }
    }
    pub fn depth(&self) -> usize {
        self.depth
    }
}

#[derive(Debug, Clone)]
pub struct Arena<T> {
    elements: HashMap<usize, T>,
    unused_ids: Vec<usize>,
    next_id: usize,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        let mut unused_ids = Vec::<usize>::new();
        let mut next_id: usize = 0;
        Self {
            elements: HashMap::<usize, T>::new(),
            unused_ids,
            next_id,
        }
    }

    pub fn from_hash_map(elements: HashMap<usize, T>) -> Self {
        let mut unused_ids = Vec::<usize>::new();
        let mut next_id: usize = 0;
        let mut element_count = elements.len();
        while element_count > 0 {
            // This might have a bug
            if elements.contains_key(&next_id) {
                element_count -= 1;
            } else {
                unused_ids.push(next_id);
            }
            next_id += 1;
        }
        Self {
            elements,
            unused_ids,
            next_id,
        }
    }

    pub fn keys(&self) -> impl std::iter::Iterator<Item = &'_ usize> {
        self.elements.keys()
    }

    fn pop_unused_id(&mut self) -> usize {
        if let Some(id) = self.unused_ids.pop() {
            return id;
        }

        let id = self.next_id;
        self.next_id += 1;
        return id;
    }

    pub fn create(&mut self, value: T) -> usize {
        let id = self.pop_unused_id();
        // Should check if there are no collisions for debugging
        self.elements.insert(id, value);
        id
    }

    pub fn get(&self, index: &usize) -> Option<&'_ T> {
        self.elements.get(index)
    }

    pub fn get_mut(&mut self, index: &usize) -> Option<&'_ mut T> {
        self.elements.get_mut(index)
    }

    pub fn iter<'m>(&'m self) -> Iterator<'m, T> {
        Iterator::<'m, T> {
            iter: self.elements.iter(),
        }
    }

    pub fn iter_mut<'m>(&'m mut self) -> IteratorMut<'m, T> {
        IteratorMut::<'m, T> {
            iter: self.elements.iter_mut(),
        }
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::from_hash_map(Default::default())
    }
}

impl<T> core::ops::Index<&usize> for Arena<T> {
    type Output = T;
    fn index(&self, index: &usize) -> &Self::Output {
        &self.elements.index(index)
    }
}

impl<T> core::ops::IndexMut<&usize> for Arena<T> {
    fn index_mut(&mut self, index: &usize) -> &mut Self::Output {
        self.elements.get_mut(index).unwrap()
    }
}

impl<T> Serialize for Arena<T>
where
    T: Serialize,
{
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> std::result::Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let ordered: BTreeMap<_, _> = self.elements.iter().collect();
        ordered.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Arena<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let elements = HashMap::<usize, T>::deserialize(deserializer)?;
        Ok(Self::from_hash_map(elements))
    }
}

pub struct Iterator<'m, T> {
    iter: std::collections::hash_map::Iter<'m, usize, T>,
}

impl<'m, T> std::iter::Iterator for Iterator<'m, T> {
    type Item = (&'m usize, &'m T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct IteratorMut<'m, T> {
    iter: std::collections::hash_map::IterMut<'m, usize, T>,
}

impl<'m, T> std::iter::Iterator for IteratorMut<'m, T> {
    type Item = (&'m usize, &'m mut T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

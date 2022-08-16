use std::collections::hash_map::{Entry, HashMap};
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

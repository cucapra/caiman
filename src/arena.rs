use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::default::Default;

/// A key used to access an element in the arena. Currently, this is just a `usize` index into a
/// backing array, but this is an implementation detail and may eventually change. In particular
/// this may become an opaque type which holds a generational index for bug-catching purposes.
pub type Key = usize;
const INVALID_KEY: Key = Key::MAX;

#[derive(Clone)]
enum Entry<T> {
    /// An occupied entry.
    Used {
        /// The value contained by the entry.
        contents: T,
        // TODO: Should I implement a generational index for bug-catching? If added, I should make
        // the raw index u32 and the generational index u32 so they can fit in a single 64 bit
        // register, and specifically stick the generational index in the high 32 bits so it can be
        // easily masked out for indexing. The generational index should live outside of the enum.
    },
    /// An unoccupied entry.
    Free {
        /// The index of the next free entry in the data structure's internal storage. This induces
        /// a chain of free entries (a "free list"). The chain terminates when the contained value
        /// is [`INVALID_KEY`] then it is the last entry in the chain.
        next: Key,
    },
}
impl<T> Entry<T> {
    /// If this entry is used, returns an immutable reference to the contents.
    fn used(&self) -> Option<&'_ T> {
        match self {
            Self::Used { contents } => Some(contents),
            Self::Free { .. } => None,
        }
    }
    /// If this entry is unused, returns a mutable reference to the contents.
    fn used_mut(&mut self) -> Option<&'_ mut T> {
        match self {
            Self::Used { contents } => Some(contents),
            Self::Free { .. } => None,
        }
    }
}

/// From an API perspective
#[derive(Clone)]
pub struct Arena2<T> {
    /// The backing storage for the entries.
    storage: Vec<Entry<T>>,
    /// The index of the head of the free list, or [`INVALID_KEY`] if all entries are used.
    /// Invariants:
    /// - This must be either the index of a free entry or [`INVALID_KEY`].
    /// - The `nextFree` member of each entry in the free list must be either the index of a free
    ///   entry or [`INVALID_KEY`].
    free_head: Key,
}
impl<T> Arena2<T> {
    /// Creates a new, empty [`Arena`]. Does not allocate until elements are added.
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
            free_head: INVALID_KEY,
        }
    }
    /// Creates a new, empty [`Arena`] with enough space preallocated to hold `capacity` elements.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            storage: Vec::with_capacity(capacity),
            free_head: INVALID_KEY,
        }
    }
    /// Adds the specified element to the collection and returns an ID identifying it.
    pub fn add(&mut self, element: T) -> Key {
        // If self.free_head is the invalid key (indicating no free entries), this will return None,
        // since the invalid key is greater than the maximum capacity of a Vec (isize::MAX)
        if let Some(entry) = self.storage.get_mut(self.free_head) {
            let key = self.free_head;
            match entry {
                Entry::Free { next } => self.free_head = *next,
                Entry::Used { .. } => unreachable!(),
            };
            *entry = Entry::Used { contents: element };
            return key;
        } else {
            let key = self.storage.len();
            self.storage.push(Entry::Used { contents: element });
            return key;
        }
    }
    /// Removes the element at the specified key from the collection.
    ///
    /// # Panics
    /// Panics if that element does not exist in the collection.
    pub fn remove(&mut self, key: Key) {
        let used = match self.storage.get_mut(key) {
            Some(used @ Entry::Used { .. }) => used,
            _ => panic!("element does not exist"),
        };
        *used = Entry::Free {
            next: self.free_head,
        };
        self.free_head = key;
    }
    /// Returns an immutable reference to an element, or None if the index is out of bounds.
    pub fn get(&self, key: Key) -> Option<&'_ T> {
        self.storage.get(key)?.used()
    }
    /// Returns a mutable reference to an element, or None if the index is out of bounds.
    pub fn get_mut(&mut self, key: Key) -> Option<&'_ mut T> {
        self.storage.get_mut(key)?.used_mut()
    }
    /// An iterator visiting all key-value pairs from lowest key to highest.
    pub fn iter(&self) -> impl std::iter::Iterator<Item = (usize, &'_ T)> {
        self.storage.iter().filter_map(Entry::used).enumerate()
    }
    /// An iterator visiting all key-value pairs from lowest key to highest, with mutable references
    /// to the values.
    pub fn iter_mut(&mut self) -> impl std::iter::Iterator<Item = (usize, &'_ mut T)> {
        self.storage
            .iter_mut()
            .filter_map(Entry::used_mut)
            .enumerate()
    }
}
impl<T: std::default::Default> std::default::Default for Arena2<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: std::fmt::Debug> std::fmt::Debug for Arena2<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}
impl<T> core::ops::Index<&Key> for Arena2<T> {
    type Output = T;
    fn index(&self, key: &Key) -> &Self::Output {
        self.storage
            .get(*key)
            .and_then(Entry::used)
            .expect("invalid index")
    }
}
impl<T> core::ops::IndexMut<&Key> for Arena2<T> {
    fn index_mut(&mut self, key: &Key) -> &mut Self::Output {
        self.storage
            .get_mut(*key)
            .and_then(Entry::used_mut)
            .expect("invalid index")
    }
}
impl<T: Serialize> Serialize for Arena2<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(self.iter())
    }
}

struct Visitor<T> {
    marker: std::marker::PhantomData<fn() -> Arena2<T>>,
}
impl<T> Visitor<T> {
    fn new() -> Self {
        Visitor {
            marker: std::marker::PhantomData,
        }
    }
}
impl<'de, T> serde::de::Visitor<'de> for Visitor<T>
where
    T: Clone + Deserialize<'de>,
{
    type Value = Arena2<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map with nonnegative integer keys")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: serde::de::MapAccess<'de>,
    {
        // Optimistically assume that the keys go from 0..n where n is the total number of keys
        let mut collection = Arena2::with_capacity(access.size_hint().unwrap_or(0));
        let storage = &mut collection.storage;

        //  We directly edit the storage, breaking its invariants! We can't call methods
        //  until we fix up the invariants at the end.
        while let Some((key, contents)) = access.next_entry::<usize, T>()? {
            // Ensure that the given index actually exists, padding the gap with
            let needed_len = key + 1;
            if needed_len >= storage.len() {
                storage.resize(needed_len, Entry::Free { next: INVALID_KEY });
            }
            if (matches!(storage[key], Entry::Used { .. })) {
                return Err(serde::de::Error::duplicate_field("(numeric)"));
            }
            storage[key] = Entry::Used { contents };
        }

        // Linearly scan through the arena and fix up the free list invariants.
        for (i, entry) in storage.iter_mut().enumerate() {
            if let Entry::Free { next } = entry {
                *next = collection.free_head;
                collection.free_head = i;
            }
        }

        Ok(collection)
    }
}

impl<'de, T> Deserialize<'de> for Arena2<T>
where
    T: Clone + Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(Visitor::new())
    }
}

// Specialized PartialEq implementation because we need to ignore free entries
impl<T: PartialEq> PartialEq for Arena2<T> {
    fn eq(&self, other: &Self) -> bool {
        let len = std::cmp::max(self.storage.len(), other.storage.len());
        for i in 0..len {
            match (self.storage.get(i), other.storage.get(i)) {
                (Some(Entry::Used { contents: a }), Some(Entry::Used { contents: b }))
                    if a != b =>
                {
                    return false
                }
                (Some(Entry::Used { .. }), None) | (None, Some(Entry::Used { .. })) => {
                    return false
                }
                _ => continue,
            }
        }
        return true;
    }
}
impl<T: Eq> Eq for Arena2<T> {}

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
        self.elements.serialize(serializer)
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

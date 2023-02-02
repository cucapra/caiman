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

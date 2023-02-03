use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A key used to access an element in the StableVec. Currently, this is just a `usize` index into a
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
        /// is [`INVALID_KEY`].
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

/// From an API perspective, this is like a HashMap where you don't get to choose the keys.
/// O(1) insert, O(1) index, O(1) iteration, O(1) remove, O(|max concurrent entries|) space
#[derive(Clone)]
pub struct StableVec<T> {
    /// The backing storage for the entries.
    storage: Vec<Entry<T>>,
    /// The index of the head of the free list, or [`INVALID_KEY`] if all entries are used.
    /// Invariants:
    /// - This must be either the index of a free entry or [`INVALID_KEY`].
    /// - The `nextFree` member of each entry in the free list must be either the index of a free
    ///   entry or [`INVALID_KEY`].
    free_head: Key,
}
impl<T> StableVec<T> {
    /// Creates a new, empty [`StableVec`]. Does not allocate until elements are added.
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
            free_head: INVALID_KEY,
        }
    }
    /// Creates a new, empty [`StableVec`] with enough space preallocated to hold `capacity` elements.
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
        self.storage
            .iter()
            .enumerate()
            .filter_map(|(a, b)| Some((a, b.used()?)))
    }
    /// An iterator visiting all key-value pairs from lowest key to highest, with mutable references
    /// to the values.
    pub fn iter_mut(&mut self) -> impl std::iter::Iterator<Item = (usize, &'_ mut T)> {
        self.storage
            .iter_mut()
            .enumerate()
            .filter_map(|(a, b)| Some((a, b.used_mut()?)))
    }
}
impl<T> std::default::Default for StableVec<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: std::fmt::Debug> std::fmt::Debug for StableVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}
impl<T> core::ops::Index<Key> for StableVec<T> {
    type Output = T;
    fn index(&self, key: Key) -> &Self::Output {
        self.get(key).expect("invalid index")
    }
}
impl<T> core::ops::IndexMut<Key> for StableVec<T> {
    fn index_mut(&mut self, key: Key) -> &mut Self::Output {
        self.get_mut(key).expect("invalid index")
    }
}
impl<T: Serialize> Serialize for StableVec<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map(self.iter())
    }
}

struct Visitor<T> {
    marker: std::marker::PhantomData<fn() -> StableVec<T>>,
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
    type Value = StableVec<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map with nonnegative integer keys")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: serde::de::MapAccess<'de>,
    {
        // Optimistically assume that the keys go from 0..n where n is the total number of keys
        let mut collection = StableVec::with_capacity(access.size_hint().unwrap_or(0));
        let storage = &mut collection.storage;

        //  We directly edit the storage, breaking its invariants! We can't call methods
        //  until we fix up the invariants at the end.
        while let Some((key, contents)) = access.next_entry::<usize, T>()? {
            // Ensure that the given index actually exists, padding the gap with
            let needed_len = key + 1;
            if needed_len >= storage.len() {
                storage.resize(needed_len, Entry::Free { next: INVALID_KEY });
            }
            if matches!(storage[key], Entry::Used { .. }) {
                return Err(serde::de::Error::duplicate_field("(numeric)"));
            }
            storage[key] = Entry::Used { contents };
        }

        // Linearly scan through the StableVec and fix up the free list invariants.
        for (i, entry) in storage.iter_mut().enumerate() {
            if let Entry::Free { next } = entry {
                *next = collection.free_head;
                collection.free_head = i;
            }
        }

        Ok(collection)
    }
}

impl<'de, T> Deserialize<'de> for StableVec<T>
where
    T: Clone + Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(Visitor::new())
    }
}

// Specialized PartialEq implementation because we need to ignore free entries
impl<T: PartialEq> PartialEq for StableVec<T> {
    fn eq(&self, other: &Self) -> bool {
        let len = std::cmp::max(self.storage.len(), other.storage.len());
        for i in 0..len {
            match (self.storage.get(i), other.storage.get(i)) {
                (Some(Entry::Used { contents: a }), Some(Entry::Used { contents: b }))
                    if a == b =>
                {
                    continue
                }
                (Some(Entry::Free { .. }) | None, Some(Entry::Free { .. }) | None) => continue,
                _ => return false,
            }
        }
        return true;
    }
}
impl<T: Eq> Eq for StableVec<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn add() {
        let mut sv = StableVec::new();
        let seven = sv.add(7i32);
        let four = sv.add(4i32);
        let nfive = sv.add(-5i32);

        assert_eq!(sv[seven], 7);
        assert_eq!(sv[four], 4);
        assert_eq!(sv[nfive], -5);

        assert_eq!(sv.get(seven), Some(&7));
        assert_eq!(sv.get(four), Some(&4));
        assert_eq!(sv.get(nfive), Some(&-5));

        assert_eq!(sv.get_mut(four), Some(&mut 4));
        assert_eq!(sv.get_mut(seven), Some(&mut 7));
        assert_eq!(sv.get_mut(nfive), Some(&mut -5));
    }
    #[test]
    fn mutate() {
        let mut sv = StableVec::new();
        let a = sv.add(7i32);
        let b = sv.add(4i32);

        *sv.get_mut(a).unwrap() = 62;
        sv[b] = 108;

        assert_eq!(sv[a], 62);
        assert_eq!(sv.get(b), Some(&108));
    }
    #[test]
    fn remove() {
        let mut sv = StableVec::new();
        let a = sv.add(1234);
        let b = sv.add(5678);
        let c = sv.add(9101112);
        // In general, I can't test that it will fill the most recently freed entry, since I don't
        // want to make that guarantee. But if there's only one entry free then it must fill it.
        sv.remove(a);
        let d = sv.add(4321);
        // This will break if generational indices are added
        assert_eq!(a, d);
        assert_eq!(sv[d], 4321);
        sv.remove(a);
        let _ = sv.add(1324); // should fill in a again
        sv.remove(b);
        sv.remove(c);
        sv.remove(a);
        let _ = sv.add(1);
        let _ = sv.add(2);
        let _ = sv.add(3);
        let f = sv.add(4);
        // Again, this will break if generational indices are added
        assert_eq!(f, 3);
    }
    #[test]
    #[should_panic]
    fn access_unallocated() {
        let mut sv = StableVec::new();
        let _ = sv.add(8i32);
        sv[33] = 47;
    }
    #[test]
    #[should_panic]
    fn access_freed() {
        let mut sv = StableVec::new();
        let a = sv.add(9i32);
        let _ = sv.add(-80);
        sv.remove(a);
        sv[a] = 10;
    }
    #[test]
    #[should_panic]
    fn remove_unallocated() {
        let mut sv = StableVec::<i32>::new();
        sv.remove(33);
    }
    #[test]
    #[should_panic]
    fn remove_freed() {
        let mut sv = StableVec::new();
        let a = sv.add(9i32);
        let _ = sv.add(-80);
        sv.remove(a);
        sv.remove(a);
    }
    #[test]
    fn basic_equality() {
        let mut sv1 = StableVec::new();
        let _ = sv1.add(8i32);
        let _ = sv1.add(7i32);
        let mut sv2 = StableVec::new();
        let _ = sv2.add(8i32);
        let _ = sv2.add(7i32);
        let mut sv3 = StableVec::new();
        let _ = sv3.add(7i32);
        let _ = sv3.add(7i32);
        let mut sv4 = StableVec::new();
        let _ = sv4.add(8i32);
        let _ = sv4.add(8i32);
        let mut sv5 = StableVec::new();
        let _ = sv5.add(8i32);
        let _ = sv5.add(7i32);
        let _ = sv5.add(6i32);

        assert_eq!(sv1, sv2);
        assert_ne!(sv1, sv3);
        assert_ne!(sv1, sv4);
        assert_ne!(sv1, sv5);
    }
    #[test]
    #[allow(unused_variables)]
    fn freed_equality() {
        let mut sv1 = StableVec::new();
        let a1 = sv1.add(1);
        let b1 = sv1.add(2);
        let c1 = sv1.add(3);
        sv1.remove(a1);
        sv1.remove(c1);

        let mut sv2 = StableVec::new();
        let a2 = sv2.add(1);
        let b2 = sv2.add(2);
        let c2 = sv2.add(3);
        sv2.remove(c2);
        sv2.remove(a2);
        assert_eq!(sv1, sv2);

        let mut sv3 = StableVec::new();
        let a3 = sv3.add(1);
        let b3 = sv3.add(2);
        let c3 = sv3.add(3);
        sv3.remove(a3);
        assert_ne!(sv1, sv3);

        let mut sv4 = StableVec::new();
        let a4 = sv4.add(1);
        let b4 = sv4.add(2);
        let c4 = sv4.add(3);
        let d4 = sv4.add(4);
        sv4.remove(a4);
        sv4.remove(d4);
        sv4.remove(c4);
        assert_eq!(sv1, sv4);

        let mut sv5 = StableVec::new();
        let a5 = sv5.add(1);
        let b5 = sv5.add(2);
        let c5 = sv5.add(2);
        sv5.remove(a5);
        sv5.remove(b5);
        assert_ne!(sv1, sv5);
    }
    #[test]
    fn serde() {
        let mut initial = StableVec::new();
        let _ = initial.add(5);
        let a = initial.add(8);
        let _ = initial.add(7);
        let _ = initial.add(2);
        let b = initial.add(6);
        initial.remove(a);
        initial.remove(b);
        let serialized = ron::to_string(&initial).unwrap();
        let deserialized = ron::from_str(&serialized).unwrap();
        assert_eq!(initial, deserialized);
    }
}

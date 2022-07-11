use crate::dataflow::{traversals, Error, Graph, NodeIndex, ValueDependent};
use std::collections::hash_map::{HashMap, RandomState};
use std::convert::TryFrom;
use std::hash::{BuildHasher, Hash, Hasher};

/// A map from node contents to a canonical node index.
///
/// This is essentially a bespoke chained hashmap. The implementation is simple because it's
/// *tightly* coupled to [`apply`]: it only needs to support [`get_or_insert`], so we don't have
/// to worry about entry deletion, iterators, etc. Three things to note:
/// 1. The chained entries are stored in a continguous vector for cache-friendliness.
///    Links between entries are given by an offset from the start of the vector.
/// 2. Although a hash of the node contents is stored, the "key" is actually the node index.
///    This allows us to get ~O(1) canonical index lookups *without duplicating the keys*.
///    This is the whole reason why we don't just use a `HashMap`.
/// 3. Because of (2), you must pass a reference to the graph to every mutating operation,
///    and you **must not mutate any nodes once their indexes have been added to the map.**
#[derive(Debug)]
struct CanonicalMap {
    buckets: Box<[usize]>,
    num_buckets: usize,
    entries: Vec<(NodeIndex, usize)>,
    random_state: RandomState,
}
impl CanonicalMap {
    const INVALID_ENTRY: usize = usize::MAX;
    /// Creates a new [`CanonicalMap`] with a capacity to hold `capacity` entries.
    fn new(capacity: usize) -> Self {
        let num_buckets = (capacity * 2).next_power_of_two();
        let buckets = vec![Self::INVALID_ENTRY; num_buckets].into_boxed_slice();
        let entries = Vec::with_capacity(capacity);
        let random_state = RandomState::new();
        Self {
            buckets,
            num_buckets,
            entries,
            random_state,
        }
    }
    /// Calculates the bucket index by hashing the contents of the node at `index` in `graph`.
    /// (The signature is weird to avoid ugly syntax elsewhere due to lifetimes.)
    fn bucket_for_index(
        num_buckets: usize,
        random_state: &RandomState,
        index: NodeIndex,
        graph: &Graph,
    ) -> usize {
        debug_assert!(num_buckets.is_power_of_two());
        let mut hasher = random_state.build_hasher();
        graph.node(index).hash(&mut hasher);
        usize::try_from(hasher.finish()).unwrap() & (num_buckets - 1)
    }
    /// Doubles this instance's bucket count and rehashes.
    fn expand(&mut self, graph: &Graph) {
        self.num_buckets *= 2;
        self.buckets = vec![Self::INVALID_ENTRY; self.num_buckets].into_boxed_slice();
        for (i, entry) in self.entries.iter_mut().enumerate() {
            let bucket =
                Self::bucket_for_index(self.num_buckets, &self.random_state, entry.0, graph);
            entry.1 = self.buckets[bucket];
            self.buckets[bucket] = i;
        }
    }
    /// If the contents of the node at `index` in `graph` are already in the map, return the index
    /// those contents map to; otherwise, add a mapping between those contents and `index`.
    fn get_or_insert(&mut self, index: NodeIndex, graph: &Graph) -> Option<NodeIndex> {
        // rehash if necessary -- this is just a rough heuristic.
        if self.entries.len() * 3 > self.num_buckets * 2 {
            self.expand(graph);
        }

        let bucket = Self::bucket_for_index(self.num_buckets, &self.random_state, index, graph);
        let bucket_contents = &mut self.buckets[bucket];

        // walk back linked list, try to find a match
        let mut cur = index;
        let mut prev = *bucket_contents;
        while prev != Self::INVALID_ENTRY {
            let entry = self.entries[prev];
            if graph.node(cur) == graph.node(entry.0) {
                return Some(entry.0);
            }
            cur = entry.0;
            prev = entry.1;
        }

        // no match found, so add it
        self.entries.push((index, *bucket_contents));
        *bucket_contents = self.entries.len() - 1;
        None
    }
}

/// Applies common subexpression elimination to `graph`. CSE is only applied to "reachable" nodes
/// (those iterated over by [`traversals::DependencyFirst`]) — this is probably what you want.
///
/// Currently, this function *does not* utilize mathematical properties such as transitivity.
pub fn apply(graph: &mut Graph) -> Result<(), Error> {
    // Map from nodes (their actual contents!) to a canonical node index
    let mut canonical = CanonicalMap::new(graph.num_nodes());
    // Map from duplicate node index to canonical node index
    let mut dedup = HashMap::new();

    let mut traversal = traversals::DependencyFirst::new(graph);
    while let Some(index) = traversal.next(graph)? {
        // -------- Step 1: Canonicalize this node's dependencies
        let canonicalize = |i| dedup.get(&i).copied().unwrap_or(i);
        graph.node_mut(index).map_dependencies(canonicalize);

        // -------- Step 2: See if there's already a canonical index for this node
        // If so, ensure dependents on this node are remapped to refer to the canonical index
        // If not, this becomes the canonical index
        if let Some(other) = canonical.get_or_insert(index, graph) {
            dedup.insert(index, other);
        }
    }

    // -------- Step 3: Canonicalize the tail edge's dependencies
    // TODO: This should not be necessary after operation-tail unification
    let canonicalize = |i| dedup.get(&i).copied().unwrap_or(i);
    graph.tail_mut().map_dependencies(canonicalize);

    Ok(())
}

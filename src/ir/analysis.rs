use crate::ir;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::RangeInclusive;

pub type LiveRangeMap = HashMap<ir::NodeId, RangeInclusive<ir::NodeId>>;

/// Analyzes the given funclet and returns a hashmap which maps node IDs to their "live range". I
/// define the live range as the range from the first use *after* the node to their last use. A
/// node which is never used after it's created will have a live range of `None`. I define a "use"
/// as a read OR a write, which is once again different from standard use/def terminology.
///
/// Note that this is the live range of the node's result, so it's pretty meaningless for stuff like
/// function dispatches. However, for allocations, this is quite meaningful!
pub fn live_ranges(funclet: &ir::Funclet) -> LiveRangeMap {
    let mut ranges = HashMap::new();

    fn update_live_range(ranges: &mut LiveRangeMap, nodeId: ir::NodeId, referrer: ir::NodeId) {
        match ranges.entry(nodeId) {
            Entry::Occupied(mut existingRange) => {
                existingRange.insert(*existingRange.get().start()..=referrer);
            }
            Entry::Vacant(slot) => {
                slot.insert(referrer..=referrer);
            }
        }
    }

    for (referrer, node) in funclet.nodes.iter().enumerate() {
        // TODO: This is a bit dumb, we discard the map result and just use it for the
        // side effects. Really, there should be a more general way to iterate over references
        let _ = node.map_referenced_nodes(|nodeId| {
            update_live_range(&mut ranges, nodeId, referrer);
            nodeId
        });
    }
    return ranges;
}

#![warn(warnings)]
#![allow(dead_code)]
use crate::ir::*;
use std::cmp::Ordering;
use std::collections::hash_map::{Entry, HashMap};
use thiserror::Error;

/// A "reducible group"; all control flow between funclets within the same reducible group
/// is reducible.
struct RGroup {
    /// The "internal funclets" -- that is, all funclets in the reducible group.
    funclets: Vec<FuncletId>,
    /// The indices of all other groups which reference an internal funclet.
    /// Invariant: no self loops, no duplicates
    incoming: Vec<usize>,
}
// just for mem::take
impl Default for RGroup {
    fn default() -> Self {
        Self {
            funclets: Vec::new(),
            incoming: Vec::new(),
        }
    }
}

fn minimal_rgroups(program: &Program) -> Vec<RGroup> {
    // rgroups, with no incoming info, hashed by their funclet
    let mut rgroups = Vec::new();
    let mut map = HashMap::new();
    for &id in program.funclets.keys() {
        let index = rgroups.len();
        rgroups.push(RGroup {
            funclets: vec![id],
            incoming: Vec::new(),
        });
        map.insert(id, index);
    }
    for id in program.funclets.keys() {
        program.funclets[id].tail_edge.for_each_funclet(|next| {
            if *id != next {
                rgroups[map[&next]].incoming.push(map[id]);
            }
        });
    }
    rgroups
}
fn apply_t2(
    old_id: usize,
    old: &mut [RGroup],
    new: &mut Vec<RGroup>,
    map: &mut HashMap<usize, usize>,
) -> usize {
    let old_pid = match map.entry(old_id) {
        Entry::Occupied(new_id) => return *new_id.get(),
        Entry::Vacant(spot) => {
            let inc = &mut old[old_id].incoming;
            if inc.len() != 1 {
                // multiple predecessors, or none; not a candidate for merge
                let new_id = new.len();
                new.push(std::mem::take(&mut old[old_id]));
                spot.insert(new_id);
                return new_id;
            }
            // this clear handles a degenerate case where each rgroup has exactly one incoming
            // a simple case is a <-> b
            let pid = inc[0];
            inc.clear();
            pid
        }
    };
    // only one predecessor: recurse, then merge our fids with theirs
    let merged_id = apply_t2(old_pid, old, new, map);
    map.insert(old_id, merged_id);
    new[merged_id].funclets.append(&mut old[old_id].funclets);
    return merged_id;
}
fn maximal_rgroups(program: &Program) -> Vec<RGroup> {
    let mut rgroups = minimal_rgroups(program);
    loop {
        let old_len = rgroups.len();
        let mut map = HashMap::new();
        {
            // Apply T2 transformations
            let mut new_rgroups = Vec::with_capacity(old_len);
            for i in 0..rgroups.len() {
                apply_t2(i, &mut rgroups, &mut new_rgroups, &mut map);
            }
            rgroups = new_rgroups;
        }
        for (i, rg) in rgroups.iter_mut().enumerate() {
            // The indices are all wrong after updating...
            rg.incoming.iter_mut().for_each(|id| *id = map[id]);

            // The "no duplicates" invariant might be violated...
            rg.incoming.sort_unstable();
            rg.incoming.dedup();

            // The no self-loops invariant may still be violated;
            // by correcting that, we do the T1 transformation
            if let Ok(j) = rg.incoming.binary_search(&i) {
                rg.incoming.swap_remove(j);
            }
        }
        // Didn't make any progress? The rgroups are maximal
        if rgroups.len() == old_len {
            return rgroups;
        }
    }
}

/// Makes all control flow in `program` reducible via node splitting.
/// Returns a mapping from any duplicate funclets' IDs to their originals' IDs.
pub fn make_reducible(program: &mut Program) -> HashMap<FuncletId, FuncletId> {
    let mut mapping = HashMap::new();
    let mut rgroups = maximal_rgroups(program);
    while let Some(rg) = rgroups.iter().find(|rg| rg.incoming.len() >= 1) {
        // if there is only one incoming, then you could do T2, not maximal
        assert!(rg.incoming.len() != 1);
        for pred in &rg.incoming[1..] {
            let mut copies = HashMap::with_capacity(rg.funclets.len());
            for &funclet in rg.funclets.iter() {
                let copy = program.funclets.create(program.funclets[&funclet].clone());
                copies.insert(funclet, copy);
                mapping.insert(copy, mapping.get(&funclet).copied().unwrap_or(funclet));
            }
            for funclet in rgroups[*pred].funclets.iter().chain(copies.values()) {
                program.funclets[funclet].tail_edge.map_funclets(|id| {
                    copies.get(&id).copied().unwrap_or(id) //
                })
            }
        }
        rgroups = maximal_rgroups(program);
    }
    mapping
}

struct LoopInfo {
    iloop_header: Option<FuncletId>,
    dfsp_index: usize,
}

fn tag_loop_header(
    traversed: &mut HashMap<FuncletId, LoopInfo>,
    target: FuncletId,
    header: Option<FuncletId>,
) {
    let mut cur1 = target;
    if let Some(mut cur2) = header {
        if cur1 == cur2 {
            return;
        }
        while let Some(ih) = traversed[&cur1].iloop_header {
            if ih == cur2 {
                return;
            }
            if traversed[&ih].dfsp_index < traversed[&cur2].dfsp_index {
                traversed.get_mut(&cur1).unwrap().iloop_header = Some(cur2);
                cur1 = cur2;
                cur2 = ih;
            } else {
                cur1 = ih;
            }
        }
        traversed.get_mut(&cur1).unwrap().iloop_header = Some(cur2);
    }
}

fn traverse_loops_dfs(
    funclets: &Arena<Funclet>,
    traversed: &mut HashMap<FuncletId, LoopInfo>,
    current: FuncletId,
    dfsp_index: usize,
) -> Option<FuncletId> {
    // mark as visited by adding an entry to `traversed`...
    // but we don't want to accidentally overwrite any existing iloop info!
    match traversed.entry(current) {
        Entry::Occupied(mut existing) => {
            existing.get_mut().dfsp_index = dfsp_index;
        }
        Entry::Vacant(spot) => {
            spot.insert(LoopInfo {
                iloop_header: None,
                dfsp_index,
            });
        }
    }

    funclets[&current].tail_edge.for_each_funclet(|next| {
        // pretty straightforward adaptation from the paper, except we don't bother with
        // irreducible loops
        match traversed.entry(next) {
            Entry::Vacant(_) => {
                let header = traverse_loops_dfs(funclets, traversed, next, dfsp_index + 1);
                tag_loop_header(traversed, current, header);
            }
            Entry::Occupied(mut entry) => {
                if entry.get_mut().dfsp_index > 0 {
                    // next is a loop header
                    tag_loop_header(traversed, current, Some(next));
                } else if let Some(header) = entry.get_mut().iloop_header {
                    if traversed.get(&header).unwrap().dfsp_index > 0 {
                        tag_loop_header(traversed, current, Some(header))
                    } else {
                        panic!("irreducible loop in CFG (includes node #{})", header);
                    }
                }
            }
        }
    });

    // reset dfsp index & return the innermost loop header
    let entry = traversed.get_mut(&current).unwrap();
    entry.dfsp_index = 0;
    return entry.iloop_header;
}

pub fn identify_loops(funclets: &Arena<Funclet>, entry: FuncletId) {
    // Adapted from http://lenx.100871.net/papers/loop-SAS.pdf
    let mut traversed = HashMap::new();
    traverse_loops_dfs(funclets, &mut traversed, entry, 0);
}

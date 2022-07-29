#![warn(warnings)]
#![allow(dead_code)]
use crate::ir::*;
use std::cmp::Ordering;
use std::collections::hash_map::{Entry, HashMap};

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

struct LoopTag {
    iloop_header: Option<FuncletId>,
    dfsp_index: usize,
    is_header: bool,
}

fn tag_loop_header(
    loops: &mut HashMap<FuncletId, LoopTag>,
    target: FuncletId,
    header: Option<FuncletId>,
) {
    let mut cur1 = target;
    if let Some(mut cur2) = header {
        if cur1 == cur2 {
            return;
        }
        // essentially, we walk up the loop header "linked list" until we reach a header
        // which isn't contained in any loop, and we place it in `cur2`'s loop
        // note that cur2 may not be `header`: if header is an inner loop, that can swap places
        // with cur2 and let cur2 propagate up (the paper calls it "weaving")
        while let Some(ih) = loops[&cur1].iloop_header {
            if ih == cur2 {
                return;
            }
            if loops[&ih].dfsp_index < loops[&cur2].dfsp_index {
                loops.get_mut(&cur1).unwrap().iloop_header = Some(cur2);
                cur1 = cur2;
                cur2 = ih;
            } else {
                cur1 = ih;
            }
        }
        loops.get_mut(&cur1).unwrap().iloop_header = Some(cur2);
    }
}

fn traverse_loops_dfs(
    funclets: &Arena<Funclet>,
    loops: &mut HashMap<FuncletId, LoopTag>,
    current: FuncletId,
    dfsp_index: usize,
) -> Option<FuncletId> {
    // mark as visited by adding an entry to `traversed`...
    // but we don't want to accidentally overwrite any existing iloop info!
    match loops.entry(current) {
        Entry::Occupied(mut existing) => {
            existing.get_mut().dfsp_index = dfsp_index;
        }
        Entry::Vacant(spot) => {
            spot.insert(LoopTag {
                iloop_header: None,
                is_header: false,
                dfsp_index,
            });
        }
    }

    funclets[&current].tail_edge.for_each_funclet(|next| {
        // pretty straightforward adaptation from the paper, except we don't bother with
        // irreducible loops
        match loops.entry(next) {
            Entry::Vacant(_) => {
                let header = traverse_loops_dfs(funclets, loops, next, dfsp_index + 1);
                tag_loop_header(loops, current, header);
            }
            Entry::Occupied(mut entry) => {
                if entry.get_mut().dfsp_index > 0 {
                    entry.get_mut().is_header = true;
                    tag_loop_header(loops, current, Some(next));
                } else if let Some(header) = entry.get_mut().iloop_header {
                    if loops.get(&header).unwrap().dfsp_index > 0 {
                        tag_loop_header(loops, current, Some(header))
                    } else {
                        panic!("irreducible loop in CFG (includes node #{})", header);
                    }
                }
            }
        }
    });

    // reset dfsp index & return the innermost loop header
    let entry = loops.get_mut(&current).unwrap();
    entry.dfsp_index = 0;
    return entry.iloop_header;
}

pub struct Loops {
    /// Sorted list of loop headers.
    headers: Vec<FuncletId>,
    header_map: HashMap<FuncletId, Option<FuncletId>>,
}
impl Loops {
    pub fn headers(&self) -> &'_ [FuncletId] {
        self.headers.as_ref()
    }
    /// Returns the loop headers of the loops the given funclet is contained in,
    /// ordered from innermost to outermost.
    pub fn headers_for(&self, mut funclet: FuncletId) -> Vec<FuncletId> {
        let mut output = Vec::new();
        if let Ok(_) = self.headers.binary_search(&funclet) {
            output.push(funclet);
        }
        while let Some(next) = self.header_map[&funclet] {
            output.push(next);
            funclet = next;
        }
        output
    }
}
pub fn identify_loops(program: &Program, entry: FuncletId) -> Loops {
    // Adapted from http://lenx.100871.net/papers/loop-SAS.pdf
    let mut loops = HashMap::new();
    traverse_loops_dfs(&program.funclets, &mut loops, entry, 1);

    let mut headers = Vec::new();
    let mut header_map = HashMap::new();

    for (id, tag) in loops.into_iter() {
        header_map.insert(id, tag.iloop_header);
        if tag.is_header {
            headers.push(id);
        }
    }

    headers.sort_unstable();

    Loops {
        headers,
        header_map,
    }
}

#[derive(Clone)]
struct AnalysisEntry {
    // The ID of the ir funclet which this analysis node corresponds to.
    id: FuncletId,
    // The index of this funclet's immediate dominator.
    // For the entry node, this will be its own id (since it doesn't have an idom)
    idom: usize,
    // A list of this funclet's immediate predecessors.
    preds: Vec<usize>,
}

fn recalc_idom(nodes: &[AnalysisEntry], mut dom_a: usize, mut dom_b: usize) -> usize {
    loop {
        match dom_a.cmp(&dom_b) {
            Ordering::Less => dom_b = nodes[dom_b].idom,
            Ordering::Greater => dom_a = nodes[dom_a].idom,
            Ordering::Equal => return dom_a,
        }
    }
}

pub fn analyze(program: &Program, entry: FuncletId) -> Analysis {
    // This is loosely based on http://www.hipersoft.rice.edu/grads/publications/dom14.pdf
    let mut lookup = HashMap::new();
    let mut entries: Vec<AnalysisEntry> = Vec::new();

    let mut stack = Vec::new();
    stack.push((entry, 0));

    while let Some((id, pred_index)) = stack.pop() {
        let index = match lookup.entry(id) {
            // already visited this funclet... might need to recalc immediate dominator
            Entry::Occupied(entry) => {
                let index: usize = *entry.get();
                entries[index].preds.push(pred_index);
                let old_idom = entries[index].idom;
                let new_idom = recalc_idom(&entries, old_idom, pred_index);
                if old_idom == new_idom {
                    continue;
                }
                entries[index].idom = new_idom;
                index
            }
            // unvisited funclet, set its idom to the previous node & add its children
            Entry::Vacant(spot) => {
                let index = entries.len();
                spot.insert(index);
                entries.push(AnalysisEntry {
                    id,
                    idom: pred_index,
                    preds: vec![pred_index],
                });
                index
            }
        };
        program.funclets[&id]
            .tail_edge
            .for_each_funclet(|target| stack.push((target, index)));
    }
    // fixup the entry node, which had a dummy pred added
    entries[0].preds.remove(0);
    Analysis { entries, lookup }
}

pub struct Analysis {
    // The entry at index 0 is the "entry" entry
    entries: Vec<AnalysisEntry>,
    // A map from funclets to their entry index
    lookup: HashMap<FuncletId, usize>,
}
impl Analysis {
    pub fn immediate_dominator(&self, fid: FuncletId) -> Option<FuncletId> {
        let index = self.lookup[&fid];
        if index == 0 {
            return None;
        }
        let idom = self.entries[index].idom;
        Some(self.entries[idom].id)
    }
    pub fn dominators(&self, id: FuncletId) -> impl '_ + Iterator<Item = &'_ FuncletId> {
        Dominators::new(&self, id)
    }
}
struct Dominators<'a> {
    analysis: &'a Analysis,
    cur_index: Option<usize>,
}
impl<'a> Dominators<'a> {
    fn new(analysis: &'a Analysis, id: FuncletId) -> Self {
        let cur_index = Some(analysis.lookup[&id]);
        Self {
            analysis,
            cur_index,
        }
    }
}
impl<'a> Iterator for Dominators<'a> {
    type Item = &'a FuncletId;
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.cur_index?;
        let entry = &self.analysis.entries[index];
        self.cur_index = (index != 0).then(|| entry.idom);
        Some(&entry.id)
    }
}

/// Creates a "dummy program" with control flow given by the specified CFG.
pub fn program_from_cfg(nodes: &[&[usize]]) -> Program {
    // NOTE: correctness here depends on arenas using sequential IDs
    let mut funclets = Arena::new();
    let make_jump = |target: &FuncletId| Jump {
        target: *target,
        args: Box::new([]),
    };
    for node in nodes {
        let tail_edge = match node {
            [] => TailEdge::Return {
                return_values: Box::new([]),
            },
            [next] => TailEdge::Jump(make_jump(next)),
            rest => TailEdge::Switch {
                key: 0,
                cases: rest.iter().map(make_jump).collect(),
            },
        };
        let funclet = Funclet {
            tail_edge,
            // everything below this are dummy values
            kind: FuncletKind::MixedExplicit,
            input_types: Box::new([]),
            output_types: Box::new([]),
            // we add one constant integer so switch nodes actually have a key
            nodes: Box::new([Node::ConstantInteger {
                value: 0,
                type_id: 0,
            }]),
            input_resource_states: Default::default(),
            output_resource_states: Default::default(),
            local_meta_variables: Default::default(),
        };
        funclets.create(funclet);
    }
    let mut types = Arena::new();
    types.create(Type::U32);

    Program {
        funclets,
        types,
        external_cpu_functions: Vec::new(),
        external_gpu_functions: Vec::new(),
        value_functions: Arena::new(),
        pipelines: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    mod reducible {
        use crate::ir::{utils, FuncletId};
        macro_rules! make_tests {
            ($($name:ident [$({
                next: $tail:expr,
                loops: $loops:expr,
                doms: $doms:expr
            }),+ $(,)?]),* $(,)?) => {$(
                #[test]
                fn $name() {
                    let mut program = utils::program_from_cfg(&[ $($tail),* ]);
                    let reduce_output = utils::make_reducible(&mut program);
                    assert!(reduce_output.is_empty());
                    let expected: &[(&[FuncletId], &[FuncletId])] = &[ $( ($loops, $doms) ),* ];
                    let loops = utils::identify_loops(&program, 0);
                    let analysis = utils::analyze(&program, 0);
                    for i in 0..expected.len() {
                        let (expected_loops, expected_doms) = expected[i];

                        let actual_loops = loops.headers_for(i);
                        assert_eq!(expected_loops, actual_loops.as_slice());

                        let mut actual_doms: Vec<_> = analysis.dominators(i).copied().collect();
                        actual_doms.sort_unstable();
                        assert_eq!(expected_doms, actual_doms.as_slice());

                        let endoms = expected_doms.len();
                        let expected_idom = (endoms >= 2).then(|| expected_doms[endoms - 2]);
                        assert_eq!(expected_idom,  analysis.immediate_dominator(i))
                    }
                }
            )*};
        }

        make_tests!(
            simple_return [ {next: &[], loops: &[], doms: &[0]} ],
            simple_jump [
                {next: &[1], loops: &[], doms: &[0]},
                {next: &[2], loops: &[], doms: &[0, 1]},
                {next: &[], loops: &[], doms: &[0, 1, 2]}
            ],
            simple_switch [
                {next: &[1, 2, 3], loops: &[], doms: &[0]},
                {next: &[], loops: &[], doms: &[0, 1]},
                {next: &[4,5], loops: &[], doms: &[0, 2]},
                {next: &[], loops: &[], doms: &[0, 3]},
                {next: &[], loops: &[], doms: &[0, 2, 4]},
                {next: &[], loops: &[], doms: &[0, 2, 5]}
            ],
            entry_loop_inf [
                {next: &[0], loops: &[0], doms: &[0]}
            ],
            entry_loop [
                {next: &[0, 1], loops: &[0], doms: &[0]},
                {next: &[], loops: &[], doms: &[0, 1]}
            ],
            nested_loop_1 [
                {next: &[1], loops: &[0], doms: &[0]},
                {next: &[1, 2], loops: &[1, 0], doms: &[0, 1]},
                {next: &[0, 3], loops: &[0], doms: &[0, 1, 2]},
                {next: &[], loops: &[], doms: &[0, 1, 2, 3]}
            ],
            nested_loop_2 [
                {next: &[0, 1], loops: &[0], doms: &[0]},
                {next: &[0, 1, 2], loops: &[1, 0], doms: &[0, 1]},
                {next: &[0, 1, 2, 3], loops: &[2, 1, 0], doms: &[0, 1, 2]},
                {next: &[], loops: &[], doms: &[0, 1, 2, 3]},
            ],
            diamond [
                {next: &[1, 2], loops: &[], doms: &[0]},
                {next: &[3], loops: &[], doms: &[0, 1]},
                {next: &[3], loops: &[], doms: &[0, 2]},
                {next: &[], loops: &[], doms: &[0, 3]}
            ],
            pogo [
                {next: &[1], loops: &[], doms: &[0]},
                {next: &[2], loops: &[1], doms: &[0, 1]},
                {next: &[1], loops: &[1], doms: &[0, 1, 2]}
            ],
            switch_fallthrough [
                {next: &[1, 2, 3, 4], loops: &[], doms: &[0]},
                {next: &[2], loops: &[], doms: &[0, 1]},
                {next: &[3], loops: &[], doms: &[0, 2]},
                {next: &[4], loops: &[], doms: &[0, 3]},
                {next: &[], loops: &[], doms: &[0, 4]}
            ],
            switch_fallthrough_reverse [
                {next: &[1, 2, 3, 4], loops: &[], doms: &[0]},
                {next: &[], loops: &[], doms: &[0, 1]},
                {next: &[1], loops: &[], doms: &[0, 2]},
                {next: &[2], loops: &[],  doms: &[0, 3]},
                {next: &[3], loops: &[], doms: &[0, 4]}
            ],
            ece5775_lec6_pg22 [
                {next: &[1], loops: &[], doms: &[0]},                           // = 0
                {next: &[2], loops: &[], doms: &[0, 1]},                        // = 1
                {next: &[3, 10], loops: &[2], doms: &[0, 1, 2]},                // = 2
                {next: &[4], loops: &[2], doms: &[0, 1, 2, 3]},                 // = 3
                {next: &[5, 9], loops: &[4, 2], doms: &[0, 1, 2, 3, 4]},        // = 4
                {next: &[6, 7], loops: &[4, 2], doms: &[0, 1, 2, 3, 4, 5]},     // = 5
                {next: &[8], loops: &[4, 2], doms: &[0, 1, 2, 3, 4, 5, 6]},     // = 6
                {next: &[8], loops: &[4, 2], doms: &[0, 1, 2, 3, 4, 5, 7]},     // = 7
                {next: &[4], loops: &[4, 2], doms: &[0, 1, 2, 3, 4, 5, 8]},     // = 8
                {next: &[2], loops: &[2], doms: &[0, 1, 2, 3, 4, 9]},           // = 9
                {next: &[], loops: &[], doms: &[0, 1, 2, 10]}                   // = 10
            ]
        );
    }
    mod irreducible {
        use crate::ir::{utils, FuncletId};
        // we use simpler property-based tests for irreducibles
        // since there are many valid possible outputs and I don't
        // want to bless any of them as canonical yet
        // ..
        // basically, we ensure make_reducible doesn't crash,
        // and that it's idempotent. Not too strong, I know...
        macro_rules! make_tests {
            ($($name:ident [$({
                next: $tail:expr,
                doms: $doms:expr
            }),+ $(,)?]),* $(,)?) => {$(
                #[test]
                fn $name() {
                    let mut program = utils::program_from_cfg(&[ $($tail),* ]);
                    let expected: &[&[FuncletId]] = &[ $($doms),* ];
                    let analysis = utils::analyze(&program, 0);
                    for i in 0..expected.len() {
                        let expected_doms = expected[i];
                        let mut actual_doms: Vec<_> = analysis.dominators(i).copied().collect();
                        actual_doms.sort_unstable();
                        assert_eq!(expected_doms, actual_doms.as_slice());

                        let endoms = expected_doms.len();
                        let expected_idom = (endoms >= 2).then(|| expected_doms[endoms - 2]);
                        assert_eq!(expected_idom,  analysis.immediate_dominator(i))
                    }
                    let first_output = utils::make_reducible(&mut program);
                    assert!(!first_output.is_empty());
                    let second_output = utils::make_reducible(&mut program);
                    assert!(second_output.is_empty());
                }
            )*};
        }

        make_tests!(
            triangle [
                {next: &[1, 2], doms: &[0]},        // = 0
                {next: &[2], doms: &[0, 1]},        // = 1
                {next: &[1], doms: &[0, 2]}         // = 2
            ],
            dom14_fig2 [
                {next: &[1, 2], doms: &[0]},        // = 0 (dom14:2:5)
                {next: &[3], doms: &[0, 1]},        // = 1 (dom14:2:4)
                {next: &[4], doms: &[0, 2]},        // = 2 (dom14:2:3)
                {next: &[4], doms: &[0, 3]},        // = 3 (dom14:2:1)
                {next: &[3], doms: &[0, 4]}         // = 4 (dom14:2:2)
            ],
            dom14_fig4 [
                {next: &[1, 2], doms: &[0]},        // = 0 (dom14:4:6)
                {next: &[3], doms: &[0, 1]},        // = 1 (dom14:4:5)
                {next: &[4, 5], doms: &[0, 2]},     // = 2 (dom14:4:4)
                {next: &[4], doms: &[0, 3]},        // = 3 (dom14:4:1)
                {next: &[3, 5], doms: &[0, 4]},     // = 4 (dom14:4:2)
                {next: &[4], doms: &[0, 5]},        // = 5 (dom14:4:3)
            ],
            circle_like [
                {next: &[1, 5], doms: &[0]},        // = 0
                {next: &[0, 2], doms: &[0, 1]},     // = 1
                {next: &[1, 3], doms: &[0, 2]},     // = 2
                {next: &[2, 4, 6], doms: &[0, 3]},  // = 3
                {next: &[3, 5], doms: &[0, 4]},     // = 4
                {next: &[0, 4], doms: &[0, 5]},     // = 5
                {next: &[], doms: &[0, 3, 6]}       // = 6
            ]
        );
    }
}

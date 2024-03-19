use std::collections::{BTreeMap, HashSet};

use crate::parse::ast::{
    NestedExpr, Program, SchedExpr, SchedFuncCall, SchedLiteral, SchedStmt, SchedTerm, TopLevel,
};

/// Call graph of the scheduling function calls in the program
pub struct CallGraph<'a> {
    // btree map for deterministic iteration so yields are inserted in the same
    // place every time
    /// Map from source function name to a list of (edge_id, destination function name)
    /// Edge ids are indices into the `edges` vector
    adj: BTreeMap<String, Vec<(String, usize)>>,
    edges: Vec<&'a mut SchedFuncCall>,
}

impl<'a> CallGraph<'a> {
    /// Constructs a new call graph from a program. Requires that the program is
    /// flattened.
    pub fn new(program: &'a mut Program) -> Self {
        let mut adj: BTreeMap<_, Vec<(String, usize)>> = BTreeMap::new();
        let mut edges = Vec::new();
        for decl in program {
            if let TopLevel::SchedulingFunc {
                name, statements, ..
            } = decl
            {
                Self::search_for_calls(name, statements.iter_mut(), &mut adj, &mut edges);
            }
        }
        Self { adj, edges }
    }

    /// Searches for calls in the scheduling function named `func_name` and adds
    /// them to the adjacency list and the list of edges.
    /// Require the statements we are iterating over to be flattened
    fn search_for_calls<T: Iterator<Item = &'a mut SchedStmt>>(
        func_name: &str,
        stmts: T,
        adj: &mut BTreeMap<String, Vec<(String, usize)>>,
        edges: &mut Vec<&'a mut SchedFuncCall>,
    ) {
        for stmt in stmts {
            match stmt {
                SchedStmt::Call(_, call)
                | SchedStmt::Decl {
                    expr: Some(SchedExpr::Term(SchedTerm::Call(_, call))),
                    ..
                }
                | SchedStmt::Assign {
                    rhs: SchedExpr::Term(SchedTerm::Call(_, call)),
                    ..
                }
                | SchedStmt::Return(_, SchedExpr::Term(SchedTerm::Call(_, call))) => {
                    if let NestedExpr::Term(SchedTerm::Var { name: dest, .. }) = &*call.target {
                        let edge_id = edges.len();
                        adj.entry(func_name.to_string())
                            .or_default()
                            .push((dest.clone(), edge_id));
                        edges.push(call);
                    } else {
                        panic!("Call target is not a variable");
                    }
                }
                SchedStmt::Seq { block, .. } => {
                    Self::search_for_calls(func_name, std::iter::once(block.as_mut()), adj, edges);
                }
                SchedStmt::Block(_, stmts) => {
                    Self::search_for_calls(func_name, stmts.iter_mut(), adj, edges);
                }
                SchedStmt::Return(
                    _,
                    SchedExpr::Term(SchedTerm::Lit {
                        lit: SchedLiteral::Tuple(terms),
                        ..
                    }),
                ) => {
                    // should already be flattened
                    assert!(
                        terms
                            .iter()
                            .all(|term| matches!(term, SchedExpr::Term(SchedTerm::Var { .. }))),
                        "Tuple return value should be flattened"
                    );
                }
                SchedStmt::If {
                    true_block,
                    false_block,
                    ..
                } => {
                    Self::search_for_calls(func_name, true_block.iter_mut(), adj, edges);
                    Self::search_for_calls(func_name, false_block.iter_mut(), adj, edges);
                }
                _ => (),
            }
        }
    }

    /// Performs a depth first search starting from `node` and marks all backedges
    /// in the graph
    /// # Arguments
    /// * `node` - The node to start the search from
    /// * `visited` - The set of nodes that have been visited (updated by the function)
    /// * `backedges` - The set of backedges in the graph (updated by the function)
    fn dfs(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        finished: &mut HashSet<String>,
        backedges: &mut HashSet<usize>,
    ) {
        visited.insert(node.to_string());
        if self.adj.contains_key(node) {
            for (dest, edge_id) in &self.adj[node] {
                if !finished.contains(dest) {
                    if visited.contains(dest) {
                        backedges.insert(*edge_id);
                    } else {
                        self.dfs(dest, visited, finished, backedges);
                    }
                }
            }
        }
        finished.insert(node.to_string());
    }

    /// Finds all backedges in the call graph by iterating over the adjacency list
    /// and performing a depth first search from each node
    fn find_all_backedges(&self) -> HashSet<usize> {
        let mut backedges = HashSet::new();
        let mut completed = HashSet::new();
        for node in self.adj.keys() {
            if !completed.contains(node) {
                self.dfs(node, &mut HashSet::new(), &mut completed, &mut backedges);
            }
        }
        backedges
    }

    /// Inserts yield calls into the program to break all cycles
    /// in the call graph. Mutates the program in place. May insert more yields
    /// than optimal.
    pub fn insert_yields(&mut self) {
        // This is a simple method which puts a yield at every backedge in the call graph.
        // It puts in more yields than necessary since finding the minimal set of yields
        // is the minimum feedback arc set problem which is NP-hard.
        let _debug: Vec<(_, Vec<_>)> = self
            .adj
            .iter()
            .map(|(k, v)| (k, v.iter().collect()))
            .collect();
        let backedges = self.find_all_backedges();
        for edge in backedges {
            let call = self.edges.get_mut(edge).unwrap();
            call.yield_call = true;
        }
    }
}

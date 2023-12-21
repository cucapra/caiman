use std::{cell::RefCell, collections::HashMap, rc::Rc};

type NodePtr<T, A> = Rc<RefCell<Node<T, A>>>;

/// A component of a type expression
pub trait Kind: Clone + PartialEq + Eq {}

/// A node in a type expression. Nodes are agnostic to the actual type
/// system being used.
#[derive(Clone, PartialEq, Eq)]
enum Node<T, A>
where
    T: Kind,
    A: Kind,
{
    Var {
        set: Option<NodePtr<T, A>>,
        id: usize,
    },
    Term {
        set: Option<NodePtr<T, A>>,
        op: T,
        args: Vec<NodePtr<T, A>>,
    },
    Atom(A),
}

impl<T: Kind, A: Kind> Node<T, A> {
    /// Gets the parent of a node.
    /// # Panics
    /// Panics if the node is an atom.
    fn parent(&mut self) -> &mut Option<NodePtr<T, A>> {
        match self {
            Self::Var { ref mut set, .. } | Self::Term { ref mut set, .. } => set,
            Self::Atom(_) => panic!("Atoms have no parent"),
        }
    }

    /// Checks if a node is a variable.
    const fn is_var(&self) -> bool {
        matches!(self, Self::Var { .. })
    }
}

/// Gets the representative of an equivalence class. This performs the `find`
/// operation in the union-find algorithm, path-compressing the tree.
fn representative<T: Kind, A: Kind>(n: &NodePtr<T, A>) -> NodePtr<T, A> {
    match &mut *n.borrow_mut() {
        Node::Var {
            set: Some(next_id), ..
        } => {
            *next_id = representative(next_id);
            next_id.clone()
        }
        Node::Var { set: None, .. } | Node::Atom(_) => n.clone(),
        Node::Term { set, args, .. } => {
            for a in args {
                *a = representative(a);
            }
            #[allow(clippy::option_if_let_else)]
            if let Some(set) = set {
                *set = representative(set);
                set.clone()
            } else {
                n.clone()
            }
        }
    }
}

/// Merges two equivalence classes. This performs the `union` operation in the
/// union-find algorithm.
/// Requires the nodes to be unionable (return `true` from `can_union`).
/// # Panics
/// Panics if both nodes are atoms (which would result in `false` from `can_union`)
fn union<T: Kind, A: Kind>(a: &NodePtr<T, A>, b: &NodePtr<T, A>) {
    let a = representative(a);
    let b = representative(b);
    if a.borrow().is_var() {
        *a.borrow_mut().parent() = Some(b);
    } else {
        *b.borrow_mut().parent() = Some(a);
    }
}

/// Checks if two nodes can have their equivalence classes merged.
/// # Returns
/// Returns `true` if the two nodes can be merged, `false` otherwise.
fn can_union<T: Kind, A: Kind>(a: &NodePtr<T, A>, b: &NodePtr<T, A>) -> bool {
    let borrow_a = a.borrow();
    let borrow_b = b.borrow();
    match (&*borrow_a, &*borrow_b) {
        (
            Node::Term {
                op: op_a,
                args: args_a,
                ..
            },
            Node::Term {
                op: op_b,
                args: args_b,
                ..
            },
        ) => op_a == op_b && args_a.len() == args_b.len(),
        (Node::Var { .. }, _) | (_, Node::Var { .. }) => true,
        _ => false,
    }
}

/// Unifies two nodes of a type expression.
/// # Returns
/// Returns `true` if the two nodes can be unified, `false` otherwise.
#[must_use]
fn unify<T: Kind, A: Kind>(a: &NodePtr<T, A>, b: &NodePtr<T, A>) -> bool {
    let a = representative(a);
    let b = representative(b);
    if a == b {
        return true;
    }
    if can_union(&a, &b) {
        union(&a, &b);
    }
    let borrow_a = a.borrow();
    let borrow_b = b.borrow();
    match (&*borrow_a, &*borrow_b) {
        (Node::Atom(a), Node::Atom(b)) => a == b,
        (
            Node::Term {
                op: op_a,
                args: a_args,
                ..
            },
            Node::Term {
                op: op_b,
                args: b_args,
                ..
            },
        ) => {
            if op_a != op_b || a_args.len() != b_args.len() {
                return false;
            }
            for (a, b) in a_args.iter().zip(b_args.iter()) {
                if !unify(a, b) {
                    return false;
                }
            }
            true
        }
        (Node::Var { .. }, _) | (_, Node::Var { .. }) => true,
        _ => false,
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Constraint<T: Kind, A: Kind> {
    Atom(A),
    Term(T, Vec<Constraint<T, A>>),
    Var(String),
}

impl<T: Kind, A: Kind> Constraint<T, A> {
    #[allow(dead_code)]
    fn instantiate(self, temps: &HashMap<String, String>) -> Self {
        match self {
            Self::Atom(_) => self,
            Self::Term(t, args) => {
                let new_args = args
                    .into_iter()
                    .map(|c| c.instantiate(temps))
                    .collect::<Vec<_>>();
                Self::Term(t, new_args)
            }
            Self::Var(name) => temps
                .get(&name)
                .map_or_else(|| Self::Var(name), |new_name| Self::Var(new_name.clone())),
        }
    }

    /// Checks if a constraint is a variable.
    pub const fn is_var(&self) -> bool {
        matches!(self, Self::Var(_))
    }
}

/// A polymorphic constraint with universally quantified type variables.
#[allow(dead_code)]
struct Polymorphic<T: Kind, A: Kind> {
    constraint: Constraint<T, A>,
    quantified: Vec<String>,
}

/// A typing environment.
pub struct Env<T: Kind, A: Kind> {
    temp_id: usize,
    nodes: HashMap<String, NodePtr<T, A>>,
    #[allow(dead_code)]
    polymorphic_constraints: HashMap<String, Polymorphic<T, A>>,
}

impl<T: Kind, A: Kind> Env<T, A> {
    /// Creates a fresh type variable.
    fn new_var(&mut self) -> NodePtr<T, A> {
        let id = self.temp_id;
        self.temp_id += 1;
        Rc::new(RefCell::new(Node::Var { set: None, id }))
    }

    /// Creates a fresh type variable for a name. This will overwrite any existing
    /// type variable for that name.
    fn new_type(&mut self, name: &str) {
        let node = self.new_var();
        self.nodes.insert(name.to_string(), node);
    }

    /// Creates a fresh type variable, returning its name to refer to it
    pub fn new_temp_type(&mut self) -> String {
        let name = format!("%{}", self.temp_id);
        let v = self.new_var();
        self.nodes.insert(name.clone(), v);
        name
    }

    /// Creates a fresh type variable if one is not already associated with a given name.
    pub fn new_type_if_absent(&mut self, name: &str) {
        if !self.nodes.contains_key(name) && !self.polymorphic_constraints.contains_key(name) {
            self.new_type(name);
        }
    }

    /// Adds a constraint to the environment. If the contraint contains a polymorphic
    /// type variable, it will be instantiated.
    /// # Errors
    /// Returns `Err` if the constraint cannot be added (unification fails)
    pub fn add_constraint(&mut self, var: &str, constraint: &Constraint<T, A>) -> Result<(), ()> {
        let c = self.contraint_to_node(constraint);
        let var = self.get_or_make_node(var);
        if unify(&var, &c) {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Gets the node for a variable, creating it if it does not exist.
    /// If the variable is a polymorphic constraint, it will be instantiated.
    fn get_or_make_node(&mut self, name: &str) -> NodePtr<T, A> {
        if !self.nodes.contains_key(name) {
            if self.polymorphic_constraints.contains_key(name) {
                let mut tmps = HashMap::new();
                let quantified = self
                    .polymorphic_constraints
                    .get(name)
                    .unwrap()
                    .quantified
                    .clone();
                for q in quantified {
                    tmps.insert(q, self.new_temp_type());
                }
                let p = self.polymorphic_constraints.get(name).unwrap();
                let c = p.constraint.clone().instantiate(&tmps);
                let c = self.contraint_to_node(&c);
                return c;
            }
            self.new_type(name);
        }
        self.nodes.get(name).unwrap().clone()
    }

    /// Converts a constraint to a node.
    fn contraint_to_node(&mut self, constraint: &Constraint<T, A>) -> NodePtr<T, A> {
        match constraint {
            Constraint::Atom(a) => Rc::new(RefCell::new(Node::Atom(a.clone()))),
            Constraint::Term(op, args) => Rc::new(RefCell::new(Node::Term {
                set: None,
                op: op.clone(),
                args: args
                    .iter()
                    .map(|c| self.contraint_to_node(c))
                    .collect::<Vec<_>>(),
            })),
            Constraint::Var(name) => self.get_or_make_node(name),
        }
    }

    /// Converts a node to a constraint.
    fn node_to_constraint(node: &NodePtr<T, A>) -> Constraint<T, A> {
        let node = representative(node);
        let borrow = node.borrow();
        match &*borrow {
            Node::Atom(a) => Constraint::Atom(a.clone()),
            Node::Term { op, args, .. } => Constraint::Term(
                op.clone(),
                args.iter()
                    .map(|a| Self::node_to_constraint(a))
                    .collect::<Vec<_>>(),
            ),
            Node::Var { id, .. } => Constraint::Var(format!("%{id}")),
        }
    }

    /// Creates a new typing environment.
    pub fn new() -> Self {
        Self {
            temp_id: 0,
            nodes: HashMap::new(),
            polymorphic_constraints: HashMap::new(),
        }
    }

    /// Gets the type of a variable.
    pub fn get_type(&self, name: &str) -> Option<Constraint<T, A>> {
        self.nodes.get(name).map(|n| Self::node_to_constraint(n))
    }

    /// Adds a polymorphic constraint to the environment. A polymorphic constraint
    /// is a constraint that is universally quantified over some type variables.
    pub fn new_polymorphic(&mut self, name: &str, quantified: Vec<String>, c: Constraint<T, A>) {
        self.polymorphic_constraints.insert(
            name.to_string(),
            Polymorphic {
                constraint: c,
                quantified,
            },
        );
    }
}

//! ## Unification Algorithm
//! This uses a somewhat standard version of unification using a union-find
//! with path compression. The main difference is that it supports
//! named equivalence classes (class names). These names are
//! essentially the canonical representation of an equivalence class. This'
//! allows us to not only query if two nodes are equivalent and get
//! a representative of the class, but it also allows us to query the
//! *name* of the class if it has one. Essentially, this allows
//! a metavariable to become a secondary canonical representation of a class.
//!
//! Because we use unification for type deduction and quotient
//! deduction, the API is agnostic to the actual type system being used.
//! It does this by working in terms of abstract nodes which are
//! simply metavariables, terms (trees of nodes), or atoms (leaves).
//! Different instantiations of the unification problem must map their
//! constraints onto this simple format.

use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    rc::Rc,
};

type NodePtr<T, A> = Rc<RefCell<Node<T, A>>>;

/// A component of a type expression
pub trait Kind: Clone + PartialEq + Eq + std::fmt::Debug {}

#[derive(Clone, PartialEq, Eq, Debug, Copy)]
pub enum SubtypeConstraint {
    Any,
    /// If the constraint is the lowest in the subtype lattice
    /// and unification which such cannot meet to a lower type.
    /// In other words, if the constraint is contravariant and accepts
    /// supertypes but not subtypes.
    Contravariant,
}

/// A node in a type expression. Nodes are agnostic to the actual type
/// system being used.
#[derive(Clone, PartialEq, Eq)]
enum Node<T, A>
where
    T: Kind,
    A: Kind,
{
    /// A type variable
    Var {
        /// The parent of a node in the union-find algorithm.
        parent: Option<NodePtr<T, A>>,
        id: usize,
        /// The unique id of the equivalence class the variable is a member of.
        class_id: Option<String>,
    },
    /// A term/operator of a type expression
    Term {
        /// The parent of a node in the union-find algorithm.
        parent: Option<NodePtr<T, A>>,
        op: T,
        args: Vec<NodePtr<T, A>>,
        /// The unique id of the equivalence class the term is a member of.
        class_id: Option<String>,
    },
    /// A term representing record subtyping. Fields are named, and
    /// unification results in the greatest lower bound of the two terms.
    DynamicTerm {
        parent: Option<NodePtr<T, A>>,
        op: T,
        args: BTreeMap<String, NodePtr<T, A>>,
        class_id: Option<String>,
        constraint_kind: SubtypeConstraint,
    },
    /// A concrete base type
    Atom(A),
    /// A term that, when unifying with another term, may opt to drop some
    /// unconstrained children.
    DropTerm {
        parent: Option<NodePtr<T, A>>,
        op: T,
        args: Vec<NodePtr<T, A>>,
        /// The unique id of the equivalence class the term is a member of.
        class_id: Option<String>,
    },
}

/// A newtype for `String` to print strings without quotes.
struct FString(String);

impl std::fmt::Debug for FString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for FString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: Kind, A: Kind> std::fmt::Debug for Node<T, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{self:#}")
        } else {
            write!(f, "{self}")
        }
    }
}

impl<T: Kind, A: Kind> std::fmt::Display for Node<T, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Var {
                parent,
                id,
                class_id,
            } => {
                let id = class_id.as_ref().map_or_else(
                    || FString(id.to_string()),
                    |class_id| FString(format!("{id}:{class_id}")),
                );
                let alt = f.alternate();
                let mut t = f.debug_tuple("Var");
                t.field(&id);
                if parent.is_some() {
                    let p = parent.as_ref().unwrap().borrow();
                    if alt {
                        t.field(&FString(format!("parent: {p:#}",)));
                    } else {
                        t.field(&FString(format!("parent: {p}",)));
                    }
                }
                t.finish()
            }
            Self::Term {
                op, args, class_id, ..
            }
            | Self::DropTerm {
                op, args, class_id, ..
            } => {
                let id = class_id.as_ref().map_or_else(
                    || format!("{op:?}"),
                    |class_id| format!("{op:?}:{class_id}"),
                );
                let mut t = f.debug_tuple(&id);
                for a in args {
                    t.field(&a.borrow());
                }
                t.finish()
            }
            Self::DynamicTerm {
                op, args, class_id, ..
            } => {
                let id = class_id.as_ref().map_or_else(
                    || format!("{op:?}"),
                    |class_id| format!("{op:?}:{class_id}"),
                );
                let mut t = f.debug_struct(&id);
                for (k, v) in args {
                    t.field(k, &v.borrow());
                }
                t.finish()
            }
            Self::Atom(a) => write!(f, "{a:#?}"),
        }
    }
}

impl<T: Kind, A: Kind> Node<T, A> {
    /// Gets the parent of a node.
    /// # Panics
    /// Panics if the node is an atom.
    fn parent(&mut self) -> &mut Option<NodePtr<T, A>> {
        match self {
            Self::Var { ref mut parent, .. }
            | Self::Term { ref mut parent, .. }
            | Self::DynamicTerm { ref mut parent, .. }
            | Self::DropTerm { ref mut parent, .. } => parent,
            Self::Atom(_) => panic!("Atoms have no parent"),
        }
    }

    /// Checks if a node is a variable.
    const fn is_var(&self) -> bool {
        matches!(self, Self::Var { .. })
    }

    /// Gets the mutable class of a node. Requires that the node is not an atom.
    fn get_class_mut(&mut self) -> &mut Option<String> {
        match self {
            Self::Var {
                ref mut class_id, ..
            }
            | Self::DropTerm {
                ref mut class_id, ..
            }
            | Self::Term {
                ref mut class_id, ..
            }
            | Self::DynamicTerm {
                ref mut class_id, ..
            } => class_id,
            Self::Atom(_) => panic!("Atoms have no class"),
        }
    }

    /// Gets the class of a node, or `None` if it is not a member of a class.
    const fn get_class(&self) -> Option<&String> {
        match self {
            Self::Var { class_id, .. }
            | Self::Term { class_id, .. }
            | Self::DynamicTerm { class_id, .. }
            | Self::DropTerm { class_id, .. } => class_id.as_ref(),
            Self::Atom(_) => None,
        }
    }
}

/// Deep copy of a node.
/// # Arguments
/// - `ptr`: The node to clone.
/// - `cloned_ptrs`: A map of pointers to cloned pointers. This is used to
///  avoid cloning the same node multiple times.
fn deep_clone<T: Kind, A: Kind>(
    ptr: &NodePtr<T, A>,
    cloned_ptrs: &mut HashMap<*const Node<T, A>, NodePtr<T, A>>,
) -> NodePtr<T, A> {
    let key = ptr.as_ptr().cast_const();
    if cloned_ptrs.contains_key(&key) {
        return cloned_ptrs[&key].clone();
    }
    let borrow = ptr.borrow();
    match &*borrow {
        Node::Atom(a) => {
            let r = Rc::new(RefCell::new(Node::Atom(a.clone())));
            cloned_ptrs.insert(key, r.clone());
            r
        }
        Node::Var {
            id,
            parent,
            class_id,
        } => {
            let r = Rc::new(RefCell::new(Node::Var {
                parent: parent.as_ref().map(|x| deep_clone(x, cloned_ptrs)),
                id: *id,
                class_id: class_id.clone(),
            }));
            cloned_ptrs.insert(key, r.clone());
            r
        }
        Node::Term {
            op,
            args,
            parent,
            class_id,
        } => {
            let r = Rc::new(RefCell::new(Node::Term {
                parent: parent.as_ref().map(|x| deep_clone(x, cloned_ptrs)),
                op: op.clone(),
                args: args.iter().map(|x| deep_clone(x, cloned_ptrs)).collect(),
                class_id: class_id.clone(),
            }));
            cloned_ptrs.insert(key, r.clone());
            r
        }
        Node::DropTerm {
            op,
            args,
            parent,
            class_id,
        } => {
            let r = Rc::new(RefCell::new(Node::DropTerm {
                parent: parent.as_ref().map(|x| deep_clone(x, cloned_ptrs)),
                op: op.clone(),
                args: args.iter().map(|x| deep_clone(x, cloned_ptrs)).collect(),
                class_id: class_id.clone(),
            }));
            cloned_ptrs.insert(key, r.clone());
            r
        }
        Node::DynamicTerm {
            parent,
            op,
            args,
            class_id,
            constraint_kind,
        } => {
            let r = Rc::new(RefCell::new(Node::DynamicTerm {
                parent: parent.as_ref().map(|x| deep_clone(x, cloned_ptrs)),
                op: op.clone(),
                args: args
                    .iter()
                    .map(|(k, v)| (k.clone(), deep_clone(v, cloned_ptrs)))
                    .collect(),
                class_id: class_id.clone(),
                constraint_kind: *constraint_kind,
            }));
            cloned_ptrs.insert(key, r.clone());
            r
        }
    }
}

/// Gets the representative of an equivalence class. This performs the `find`
/// operation in the union-find algorithm, path-compressing the tree.
fn representative<T: Kind, A: Kind>(n: &NodePtr<T, A>) -> NodePtr<T, A> {
    match &mut *n.borrow_mut() {
        Node::Var {
            parent: Some(next_id),
            ..
        } => {
            *next_id = representative(next_id);
            next_id.clone()
        }
        Node::Var { parent: None, .. } | Node::Atom(_) => n.clone(),
        Node::Term { parent, args, .. } | Node::DropTerm { parent, args, .. } => {
            for a in args {
                *a = representative(a);
            }
            #[allow(clippy::option_if_let_else)]
            if let Some(parent) = parent {
                *parent = representative(parent);
                parent.clone()
            } else {
                n.clone()
            }
        }
        Node::DynamicTerm { parent, args, .. } => {
            for a in args.values_mut() {
                *a = representative(a);
            }
            #[allow(clippy::option_if_let_else)]
            if let Some(parent) = parent {
                *parent = representative(parent);
                parent.clone()
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
    if !matches!(
        (&*a.borrow(), &*b.borrow()),
        (Node::Atom(_), _) | (_, Node::Atom(_))
    ) {
        if a.borrow().get_class().is_some() {
            assert!(
                b.borrow().get_class().is_none()
                    || b.borrow().get_class() == a.borrow().get_class(),
                "Cannot unify two unequal classes: {:?} != {:?}",
                a.borrow().get_class().as_ref().unwrap(),
                b.borrow().get_class().as_ref().unwrap()
            );
            *b.borrow_mut().get_class_mut() = a.borrow().get_class().cloned();
        } else {
            *a.borrow_mut().get_class_mut() = b.borrow().get_class().cloned();
        }
    }

    if a.borrow().is_var() {
        *a.borrow_mut().parent() = Some(b);
    } else if !matches!(
        (&*a.borrow(), &*b.borrow()),
        (Node::DynamicTerm { .. }, Node::DynamicTerm { .. })
    ) {
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
        (
            Node::DropTerm {
                op: op_a,
                args: args_a,
                ..
            },
            Node::Term {
                op: op_b,
                args: args_b,
                ..
            },
        ) => op_a == op_b && args_a.len() >= args_b.len(),
        (
            Node::Term {
                op: op_a,
                args: args_a,
                ..
            },
            Node::DropTerm {
                op: op_b,
                args: args_b,
                ..
            },
        ) => op_a == op_b && args_a.len() <= args_b.len(),
        (
            Node::DynamicTerm {
                op: op_a,
                constraint_kind: const_kind_a,
                args: args_a,
                ..
            },
            Node::DynamicTerm {
                op: op_b,
                constraint_kind: const_kind_b,
                args: args_b,
                ..
            },
        ) => {
            op_a == op_b
                && (*const_kind_a != SubtypeConstraint::Contravariant
                    || args_a.len() >= args_b.len())
                && (*const_kind_b != SubtypeConstraint::Contravariant
                    || args_b.len() >= args_a.len())
        }
        (Node::Var { .. }, _) | (_, Node::Var { .. }) => true,
        _ => false,
    }
}

/// Unifies two nodes of a type expression.
/// # Returns
/// Returns `true` if the two nodes can be unified, `false` otherwise.
#[must_use]
#[allow(clippy::too_many_lines)]
fn unify<T: Kind, A: Kind>(a: &NodePtr<T, A>, b: &NodePtr<T, A>) -> bool {
    let a = representative(a);
    let b = representative(b);
    if a == b {
        return true;
    }
    if a.borrow().get_class().is_some()
        && b.borrow().get_class().is_some()
        && a.borrow().get_class() != b.borrow().get_class()
    {
        return false;
    }
    if can_union(&a, &b) {
        union(&a, &b);
    }
    {
        let borrow_a = a.borrow();
        let borrow_b = b.borrow();
        match (&*borrow_a, &*borrow_b) {
            (Node::Atom(a), Node::Atom(b)) => return a == b,
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
                return true;
            }
            (
                Node::Term {
                    op: op_a,
                    args: short_args,
                    ..
                },
                Node::DropTerm {
                    op: op_b,
                    args: long_args,
                    ..
                },
            )
            | (
                Node::DropTerm {
                    op: op_a,
                    args: long_args,
                    ..
                },
                Node::Term {
                    op: op_b,
                    args: short_args,
                    ..
                },
            ) => {
                if op_a != op_b || short_args.len() > long_args.len() {
                    return false;
                }
                let mut diff = long_args.len() - short_args.len();
                for (a, b) in short_args.iter().zip(long_args.iter().filter(|node| {
                    if diff > 0 && node.borrow().is_var() {
                        diff -= 1;
                        false
                    } else {
                        true
                    }
                })) {
                    if !unify(a, b) {
                        return false;
                    }
                }
                return true;
            }
            (
                Node::DynamicTerm {
                    op: op_a,
                    args: args_a,
                    constraint_kind: const_kind_a,
                    ..
                },
                Node::DynamicTerm {
                    op: op_b,
                    args: args_b,
                    constraint_kind: const_kind_b,
                    ..
                },
            ) => {
                if op_a != op_b
                    || (*const_kind_a == SubtypeConstraint::Contravariant
                        && (args_b.len() > args_a.len()
                            || args_b.iter().any(|(k, _)| !args_a.contains_key(k))))
                    || (*const_kind_b == SubtypeConstraint::Contravariant
                        && (args_a.len() > args_b.len()
                            || args_a.iter().any(|(k, _)| !args_b.contains_key(k))))
                {
                    return false;
                }
                for (k, a) in args_a {
                    if let Some(b) = args_b.get(k) {
                        if !unify(a, b) {
                            return false;
                        }
                    }
                }
            }
            (Node::Var { .. }, _) | (_, Node::Var { .. }) => return true,
            _ => return false,
        }
    }
    union_dynamic_terms(&a, &b);
    true
}

/// Meets two dynamic terms by reparenting them. If one term is a subtype of the other,
/// the subtype will become the parent. If neither is a subtype of the other, a new
/// parent will be created which is the greatest lower bound of the two terms.
fn union_dynamic_terms<T: Kind, A: Kind>(a: &NodePtr<T, A>, b: &NodePtr<T, A>) {
    match (&mut *a.borrow_mut(), &mut *b.borrow_mut()) {
        (
            Node::DynamicTerm {
                constraint_kind: SubtypeConstraint::Contravariant,
                ..
            },
            Node::DynamicTerm { parent, .. },
        ) => {
            parent.replace(a.clone());
        }
        (
            Node::DynamicTerm { parent, .. },
            Node::DynamicTerm {
                constraint_kind: SubtypeConstraint::Contravariant,
                ..
            },
        ) => {
            parent.replace(b.clone());
        }
        (
            Node::DynamicTerm { args: a_args, .. },
            Node::DynamicTerm {
                args: b_args,
                parent,
                ..
            },
        ) if b_args.iter().all(|(k, _)| a_args.contains_key(k)) => {
            parent.replace(a.clone());
        }
        (
            Node::DynamicTerm {
                parent,
                args: a_args,
                ..
            },
            Node::DynamicTerm { args: b_args, .. },
        ) if a_args.iter().all(|(k, _)| b_args.contains_key(k)) => {
            parent.replace(b.clone());
        }
        (
            Node::DynamicTerm {
                op,
                class_id,
                args: args_a,
                constraint_kind: const_kind_a,
                parent,
            },
            Node::DynamicTerm {
                args: args_b,
                constraint_kind: const_kind_b,
                parent: parent_b,
                ..
            },
        ) => {
            assert!(parent.is_none());
            assert!(parent_b.is_none());
            let arg_union: BTreeMap<_, _> = args_a
                .iter()
                .chain(args_b.iter())
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            let new_parent = Rc::new(RefCell::new(Node::DynamicTerm {
                parent: None,
                op: op.clone(),
                args: arg_union,
                class_id: class_id.clone(),
                constraint_kind: if *const_kind_a == SubtypeConstraint::Contravariant
                    || *const_kind_b == SubtypeConstraint::Contravariant
                {
                    SubtypeConstraint::Contravariant
                } else {
                    SubtypeConstraint::Any
                },
            }));
            parent.replace(new_parent.clone());
            parent_b.replace(new_parent);
        }
        _ => panic!("Invalid dynamic term union"),
    }
}

/// A constraint for type unification. Constraints are agnostic to the actual
/// type system being used.
///
/// # Parameters
/// - `T`: The type of terms/operators in the type system.
/// - `A`: The type of atoms in the type system.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Constraint<T: Kind, A: Kind> {
    Atom(A),
    Term(T, Vec<Constraint<T, A>>),
    DynamicTerm(T, BTreeMap<String, Constraint<T, A>>, SubtypeConstraint),
    Var(String),
    DropTerm(T, Vec<Constraint<T, A>>),
}

impl<T: Kind, A: Kind> Constraint<T, A> {
    /// Checks if a constraint is a variable.
    pub const fn is_var(&self) -> bool {
        matches!(self, Self::Var(_))
    }

    /// Returns true if the constraints are equal, up to alpha equivalence.
    pub fn alpha_equiv(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Atom(a), Self::Atom(b)) => a == b,
            (Self::Term(op_a, args_a), Self::Term(op_b, args_b)) => {
                op_a == op_b && args_a.len() == args_b.len() && {
                    for (a, b) in args_a.iter().zip(args_b.iter()) {
                        if !a.alpha_equiv(b) {
                            return false;
                        }
                    }
                    true
                }
            }
            (Self::Var(_), Self::Var(_)) => true,
            _ => false,
        }
    }

    /// Returns true if the constraints are equal, modulo unconstrained variables
    pub fn matches(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Var(_), _) | (_, Self::Var(_)) => true,
            (Self::Atom(a), Self::Atom(b)) => a == b,
            (Self::Term(op_a, args_a), Self::Term(op_b, args_b))
                if op_a == op_b && args_a.len() == args_b.len() =>
            {
                args_a.iter().zip(args_b.iter()).all(|(a, b)| a.matches(b))
            }
            (Self::DynamicTerm(..), _) | (_, Self::DynamicTerm(..)) => {
                panic!("Unexpected constraint type in matches")
            }
            (
                Self::DropTerm(op_a, args_long),
                Self::Term(op_b, args_short) | Self::DropTerm(op_b, args_short),
            )
            | (Self::Term(op_a, args_short), Self::DropTerm(op_b, args_long))
                if op_a == op_b && args_short.len() <= args_long.len() =>
            {
                let mut diff = args_long.len() - args_short.len();
                args_long
                    .iter()
                    .filter(|node| {
                        if diff > 0 && node.is_var() {
                            diff -= 1;
                            false
                        } else {
                            true
                        }
                    })
                    .zip(args_short.iter())
                    .all(|(a, b)| a.matches(b))
            }
            _ => false,
        }
    }
}

/// A typing environment.
#[derive(Debug)]
pub struct Env<T: Kind, A: Kind> {
    temp_id: usize,
    nodes: HashMap<String, NodePtr<T, A>>,
}

impl<T: Kind, A: Kind> Clone for Env<T, A> {
    fn clone(&self) -> Self {
        let mut nodes = HashMap::new();
        let mut cloned_ptrs = HashMap::new();
        for (name, node) in &self.nodes {
            nodes.insert(name.clone(), deep_clone(node, &mut cloned_ptrs));
        }
        Self {
            temp_id: self.temp_id,
            nodes,
        }
    }
}

impl<T: Kind, A: Kind> Env<T, A> {
    /// Creates a fresh type variable.
    fn new_var(&mut self) -> NodePtr<T, A> {
        let id = self.temp_id;
        self.temp_id += 1;
        Rc::new(RefCell::new(Node::Var {
            parent: None,
            id,
            class_id: None,
        }))
    }

    /// Creates a fresh type variable that represents a named class if one
    /// does not already exist. If a type variable already exists with the
    /// given name, it will be reused and made into a class node.
    /// Any new nodes will be inserted into the environment.
    fn new_class_type(&mut self, class_name: &str) -> NodePtr<T, A> {
        if self.nodes.contains_key(class_name) {
            self.nodes
                .get_mut(class_name)
                .unwrap()
                .borrow_mut()
                .get_class_mut()
                .replace(class_name.to_string());
            return self.nodes.get(class_name).unwrap().clone();
        }
        let id = self.temp_id;
        self.temp_id += 1;
        let r = Rc::new(RefCell::new(Node::Var {
            parent: None,
            id,
            class_id: Some(class_name.to_string()),
        }));
        self.nodes.insert(class_name.to_string(), r.clone());
        r
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
        if !self.nodes.contains_key(name) {
            self.new_type(name);
        }
    }

    /// Adds a constraint to the environment.
    /// # Arguments
    /// - `var`: The name of the type variable to add the constraint to.
    /// - `constraint`: The constraint to add.
    /// # Errors
    /// Returns `Err` if the constraint cannot be added (unification fails)
    pub fn add_constraint(
        &mut self,
        var: &str,
        constraint: &Constraint<T, A>,
    ) -> Result<(), String> {
        let var = self.get_or_make_node(var);
        self.add_constraint_helper(&var, constraint)
    }

    /// Unifies the node `var` with the constraint.
    fn add_constraint_helper(
        &mut self,
        var: &NodePtr<T, A>,
        constraint: &Constraint<T, A>,
    ) -> Result<(), String> {
        #![allow(clippy::similar_names)]
        let c = self.contraint_to_node(constraint);
        if unify(var, &c) {
            Ok(())
        } else {
            Err(format!(
                "{:#?}\n\t!=\n{:#?}",
                *representative(var).borrow(),
                *c.borrow()
            ))
        }
    }

    /// Adds a constraint to an equivalence class.
    /// # Arguments
    /// - `class_name`: The name of the class to add the constraint to.
    /// - `constraint`: The constraint to add.
    pub fn add_class_constraint(
        &mut self,
        class_name: &str,
        constraint: &Constraint<T, A>,
    ) -> Result<(), String> {
        let class = self.new_class_type(class_name);
        self.add_constraint_helper(&class, constraint)
    }

    /// Gets the node for a variable, creating it if it does not exist.
    /// If the variable is a polymorphic constraint, it will be instantiated.
    fn get_or_make_node(&mut self, name: &str) -> NodePtr<T, A> {
        if !self.nodes.contains_key(name) {
            self.new_type(name);
        }
        self.nodes.get(name).unwrap().clone()
    }

    /// Converts a constraint to a node.
    fn contraint_to_node(&mut self, constraint: &Constraint<T, A>) -> NodePtr<T, A> {
        match constraint {
            Constraint::Atom(a) => Rc::new(RefCell::new(Node::Atom(a.clone()))),
            Constraint::Term(op, args) => Rc::new(RefCell::new(Node::Term {
                parent: None,
                op: op.clone(),
                args: args
                    .iter()
                    .map(|c| self.contraint_to_node(c))
                    .collect::<Vec<_>>(),
                class_id: None,
            })),
            Constraint::DropTerm(op, args) => Rc::new(RefCell::new(Node::DropTerm {
                parent: None,
                op: op.clone(),
                args: args
                    .iter()
                    .map(|c| self.contraint_to_node(c))
                    .collect::<Vec<_>>(),
                class_id: None,
            })),
            Constraint::Var(name) => self.get_or_make_node(name),
            Constraint::DynamicTerm(op, args, constraint_kind) => {
                Rc::new(RefCell::new(Node::DynamicTerm {
                    parent: None,
                    op: op.clone(),
                    args: args
                        .iter()
                        .map(|(k, v)| (k.clone(), self.contraint_to_node(v)))
                        .collect(),
                    class_id: None,
                    constraint_kind: *constraint_kind,
                }))
            }
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
            Node::DropTerm { op, args, .. } => Constraint::DropTerm(
                op.clone(),
                args.iter()
                    .map(|a| Self::node_to_constraint(a))
                    .collect::<Vec<_>>(),
            ),
            Node::Var { id, parent, .. } =>
            {
                #[allow(clippy::option_if_let_else)]
                if let Some(parent) = parent {
                    Self::node_to_constraint(parent)
                } else {
                    Constraint::Var(format!("%{id}"))
                }
            }
            Node::DynamicTerm {
                op,
                args,
                constraint_kind,
                ..
            } => Constraint::DynamicTerm(
                op.clone(),
                args.iter()
                    .map(|(k, v)| (k.clone(), Self::node_to_constraint(v)))
                    .collect(),
                *constraint_kind,
            ),
        }
    }

    /// Creates a new typing environment.
    pub fn new() -> Self {
        Self {
            temp_id: 0,
            nodes: HashMap::new(),
        }
    }

    /// Gets the type of a variable.
    pub fn get_type(&self, name: &str) -> Option<Constraint<T, A>> {
        self.nodes.get(name).map(|n| Self::node_to_constraint(n))
    }

    /// Gets a unique identifier for the equivalence class the type variable
    /// is a member of. Equivalence classes do not have identifiers unless
    /// constraints were added to them via `add_class_constraint`.
    /// # Returns
    /// Returns `None` if the type variable is not a member of an equivalence class
    /// or if the equivalence class does not have an identifier.
    pub fn get_class_id(&self, name: &str) -> Option<String> {
        self.nodes
            .get(name)
            .and_then(|n| representative(n).borrow().get_class().cloned())
    }
}

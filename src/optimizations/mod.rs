#![warn(warnings)]
use crate::dataflow::*;
use crate::ir;
use crate::operations::BinopKind;
use std::convert::TryFrom;
use thiserror::Error;

mod basic_cse;

/// An error returned when parsing an [`Optimization`] using [`from_str`] fails.
///
/// [`from_str`]: std::str::FromStr::from_str
#[derive(Debug, Clone, Copy, Error)]
#[error("unknown optimization")]
pub struct ParseOptError;

/// An error returned when parsing an [`OptLevel`] using [`from_str`] fails.
///
/// [`from_str`]: std::str::FromStr::from_str
#[derive(Debug, Clone, Error)]
pub enum ParseOptLevelError {
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("not a valid optimization level")]
    InvalidOptLevel,
}

/// An error which occurred during an optimization.
#[derive(Debug, Error)]
pub enum OptError {
    #[error("value graph error: {0}")]
    ValueError(#[from] crate::dataflow::Error),
}

macro_rules! declare_opt_levels {
    (
        $(#[$attr:meta])*
        $visibility:vis enum $name:ident {
            $($variant_ident:ident = $variant_discrim:literal,)* $(,)?
        }
    ) => {
        $(#[$attr])*
        $visibility enum $name { $($variant_ident = $variant_discrim),* }
        impl From<$name> for u8 {
            fn from(src: OptLevel) -> Self { src as u8 }
        }
        impl TryFrom<u8> for $name {
            type Error = ParseOptLevelError;
            fn try_from(src: u8) -> Result<Self, Self::Error> {
                match src {
                    $($variant_discrim => Ok(Self::$variant_ident),)*
                    _ => Err(Self::Error::InvalidOptLevel)
                }
            }
        }

    };
}
declare_opt_levels! {
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub enum OptLevel {
        Min = 0,
        Max = 1,
    }
}
impl Default for OptLevel {
    fn default() -> Self {
        Self::Max
    }
}
impl std::str::FromStr for OptLevel {
    type Err = ParseOptLevelError;
    fn from_str(s: &str) -> Result<Self, ParseOptLevelError> {
        Self::try_from(s.parse::<u8>()?)
    }
}
impl std::fmt::Display for OptLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", u8::from(*self))
    }
}

macro_rules! declare_optimizations {
    (
        $(#[$attr:meta])*
        $visibility:vis enum $name:ident for $cfg:ident: &mut $cfg_type:ty {
            $($variant_ident:ident {
                name: $variant_name:literal,
                level: $variant_level:expr,
                apply: $variant_apply:expr $(,)?
            }),* $(,)?
        }
    ) => {
        $(#[$attr])*
        $visibility enum $name { $($variant_ident),* }
        impl $name {
            pub fn valid_names() -> &'static [&'static str] {
                &[ $($variant_name),* ]
            }
            pub fn from_opt_level(opt_level: OptLevel) -> Vec<Self> {
                let mut optimizations = Vec::new();
                $(if $variant_level >= opt_level {
                    optimizations.push(Self::$variant_ident)
                })*
                optimizations
            }
            pub fn name(&self) -> &'static str {
                match self { $(Self::$variant_ident => $variant_name),* }
            }
            pub fn level(&self) -> OptLevel {
                match self { $(Self::$variant_ident => $variant_level),* }
            }
            fn apply(&self, $cfg: &mut $cfg_type) {
                match self { $(Self::$variant_ident => $variant_apply),* }
            }
        }
        impl std::str::FromStr for $name {
            type Err = ParseOptError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($variant_name => Ok(Self::$variant_ident),)*
                    _ => Err(Self::Err {})
                }
            }
        }
    };
}

declare_optimizations! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Optimization for cfg: &mut Optimizer {
        BasicCse {
            name: "basic-cse",
            level: OptLevel::Max,
            apply: cfg.basic_cse = true
        },
        ConstantFold {
            name: "constant-fold",
            level: OptLevel::Max,
            apply: {
                cfg.transforms.push(Box::new(SumFold {}));
                cfg.cleanups.push(Box::new(SumUnfold {}));
            }
        },
    }
}
impl std::fmt::Display for Optimization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

pub struct Optimizer {
    /// The maximum number of optimization passes to run.
    max_passes: usize,
    /// Whether to run basic constant subexpression elimination.
    basic_cse: bool,
    /// The list of subgraph transforms to apply.
    transforms: Vec<Box<dyn SubgraphTransform>>,
    cleanups: Vec<Box<dyn SubgraphTransform>>,
}
impl Optimizer {
    pub const DEFAULT_MAX_PASSES: usize = 16;
    pub fn new<'a>(max_passes: usize, opts: &[Optimization]) -> Self {
        let mut optimizer = Self {
            max_passes,
            basic_cse: false,
            transforms: Vec::new(),
            cleanups: Vec::new(),
        };
        opts.iter().for_each(|opt| opt.apply(&mut optimizer));
        optimizer
    }
    pub fn apply(&self, program: &mut ir::Program) -> Result<(), OptError> {
        for (_, funclet) in program.funclets.iter_mut() {
            let mut graph = Graph::from_ir(&funclet.nodes, &funclet.tail_edge)?;
            for _ in 0..self.max_passes {
                if self.basic_cse {
                    basic_cse::apply(&mut graph)?;
                }
                let mut mutated = false;
                let mut traversal = traversals::DependencyFirst::new(&graph);
                let mut to_clean = Vec::new();
                while let Some(index) = traversal.next(&graph)? {
                    for transform in self.transforms.iter() {
                        mutated |= transform.attempt(&mut graph, index);
                    }
                    to_clean.push(index);
                }
                for index in to_clean.into_iter() {
                    for cleanup in self.cleanups.iter() {
                        mutated |= cleanup.attempt(&mut graph, index);
                    }
                }
                if !mutated {
                    break;
                }
            }
            if self.basic_cse {
                basic_cse::apply(&mut graph)?;
            }
            let (ir_nodes, ir_tail) = graph.into_ir()?;
            funclet.nodes = ir_nodes.into_boxed_slice();
            funclet.tail_edge = ir_tail;
        }
        Ok(())
    }
}
impl Default for Optimizer {
    fn default() -> Self {
        let transforms = Optimization::from_opt_level(OptLevel::Max);
        Self::new(Self::DEFAULT_MAX_PASSES, &transforms)
    }
}

/// Represents an transformation on a subgraph of a dataflow graph.
trait SubgraphTransform {
    /// Attempts to apply the transformation to the subgraph of `graph` induced by `index`
    /// and all of its indirect and direct dependencies. The return code indicates success.
    ///
    /// This **should not** modify any nodes outside of the aforementioned subtree.
    ///
    /// # Mutability
    /// Although [`SubgraphTransform`]s are free to mutate their subgraph, they can't mutate
    /// themselves; if self-mutability is needed, you're probably better off writing a
    /// "freestanding" transformation. This restriction exists for two reasons:
    /// - Since a [`SubgraphTransform`] can't keep track of which nodes its already visited,
    ///   it can't modify nodes outside of its subtree without using out-of-band information
    ///   to obtain their node indices. (That is, it's difficult to break the trait contract
    ///   unless you intentionally "cheat".)
    /// - If a transform *requires* self-mutability, it must maintain some internal state. That
    ///   state could be accidentally invalidated by other transformations running in parallel.
    ///   For example, [`BasicCse`](basic_cse::BasicCse) maintains an internal hashmap from node
    ///   contents to node indices. A transformation interspersed between `BasicCse` applications
    ///   could mutate node contents, thus de-syncing the hashmap from the graph.
    ///   Self-immutability helps avoid these footguns.
    fn attempt(&self, graph: &mut Graph, index: NodeIndex) -> bool;
}

struct SumFold {}
impl SubgraphTransform for SumFold {
    fn attempt(&self, graph: &mut Graph, index: NodeIndex) -> bool {
        let (arg0, arg1) = match graph.node(index) {
            Node::Binop(Binop {
                kind: BinopKind::Add,
                arg0,
                arg1,
            }) => (*arg0, *arg1),
            _ => return false,
        };
        let mut sum = Sum::new();
        sum.add_arg(graph, arg0);
        sum.add_arg(graph, arg1);
        *graph.node_mut(index) = Node::Sum(sum);
        return true;
    }
}
struct SumUnfold {}
impl SubgraphTransform for SumUnfold {
    fn attempt(&self, graph: &mut Graph, index: NodeIndex) -> bool {
        let sum = match graph.node(index) {
            Node::Sum(sum) => sum.clone(),
            _ => return false,
        };
        *graph.node_mut(index) = sum.reduce(graph);
        return true;
    }
}

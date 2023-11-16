use crate::error::Info;

pub type Name = String;

pub type Arg<T> = (String, T);
pub type NamedOutput<T> = (Option<String>, T);

/// A numeric data type
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NumberType {
    I32,
    I64,
}

/// A core data type available in the value and scheduling languages
/// Ex. i64, bool, i32 etc.
///
/// Not all of these types can be used in every function, however I think
/// it will be easier to essentially perform AST-level type checking
/// rather than trying to do it at the parsing level
#[derive(Clone, Debug)]
pub enum DataType {
    Num(NumberType),
    Bool,
    BufferSpace,
    Event,
    Tuple(Vec<DataType>),
    Array(Box<DataType>, Box<SpecExpr>),
    Slice(Box<DataType>),
    UserDefined(String),
}

/// Binary operators in the value and scheduling languages
#[derive(Clone, Debug)]
pub enum Binop {
    Dot,
    Add,
    Mul,
    Sub,
    Div,
    Land,
    Lor,
    Eq,
    Neq,
    Lt,
    Gt,
    Leq,
    Geq,
    Index,
    Range,
    Cons,
    Xor,
    Or,
    And,
    Shl,
    Shr,
    AShr,
    Mod,
}

/// Unary operators in the value and scheduling languages
#[derive(Clone, Debug)]
pub enum Uop {
    Neg,
    LNot,
    Not,
}

/// A literal in the spec languages
#[derive(Clone, Debug)]
pub enum SpecLiteral {
    Int(String),
    Float(String),
    Bool(bool),
    Array(Vec<SpecExpr>),
    Tuple(Vec<SpecExpr>),
}

/// A literal in the spec languages
#[derive(Clone, Debug)]
pub enum SchedLiteral {
    Int(String),
    Float(String),
    Bool(bool),
    Array(Vec<SchedExpr>),
    Tuple(Vec<SchedExpr>),
}

/// An expression in the spec language
pub type SpecExpr = NestedExpr<SpecTerm>;

/// A term in the spec language
/// A term is the bottom levels of the expression tree and consists of
/// atoms (variables or literals) or function calls
#[derive(Clone, Debug)]
pub enum SpecTerm {
    Var {
        info: Info,
        name: Name,
    },
    Lit {
        info: Info,
        lit: SpecLiteral,
    },
    Call {
        info: Info,
        function: Box<SpecExpr>,
        args: Vec<SpecExpr>,
        template: Option<FlaggedType>,
    },
}

/// A nested expression is the top level of an expression tree which is agnostic
/// to the type of expression (spec or scheduling)
#[derive(Clone, Debug)]
pub enum NestedExpr<T> {
    Binop {
        info: Info,
        op: Binop,
        lhs: Box<NestedExpr<T>>,
        rhs: Box<NestedExpr<T>>,
    },
    Uop {
        info: Info,
        op: Uop,
        expr: Box<NestedExpr<T>>,
    },
    Conditional {
        info: Info,
        if_true: Box<NestedExpr<T>>,
        guard: Box<NestedExpr<T>>,
        if_false: Box<NestedExpr<T>>,
    },
    Term(T),
}

/// A statement in a specification function
/// Supports all specifications
#[derive(Clone, Debug)]
pub enum SpecStmt {
    Assign {
        info: Info,
        lhs: Vec<(Name, Option<DataType>)>,
        rhs: SpecExpr,
    },
    Returns(Info, SpecExpr),
}
/// AST-level quotient (once merged, we can use the ir enum)
#[derive(Clone, Debug)]
pub enum Quotient {
    Node,
    None,
    Input,
    Output,
}

/// AST-level flow (once merged, we can use the ir enum)
#[derive(Clone, Debug)]
pub enum Flow {
    Usable,
    Save,
    Dead,
    Need,
}

/// The part of a type annotation referring to a specific variable in a spec
/// Ex: `i64<storage, map_write, align=8> @ [node(val.x)-usable, node(space.y)-usable, none(time.x)-usable]`
/// `val.x` is a spec var
#[derive(Clone, Debug)]
pub struct QuotientReference {
    /// Name of the spec
    pub spec_name: String,
    /// Name of the variable within the spec to refer to
    pub spec_var: Option<String>,
}

/// Frontend level tag
#[derive(Clone, Debug)]
pub struct Tag {
    pub info: Info,
    pub quot: Option<Quotient>,
    pub quot_var: Option<QuotientReference>,
    pub flow: Option<Flow>,
}

/// A flagged type is a base type parameterized by an optional set of WGPU
/// flags and settings
/// Ex. `i64<storage, map_write, alignment_bits=8>`
#[derive(Clone, Debug)]
pub struct FlaggedType {
    pub info: Info,
    pub base: DataType,
    /// WGPU flags, can be empty
    pub flags: Vec<String>,
    /// WGPU settings, (flags that can have values, such as `alignment_bits`)
    /// can be empty
    pub settings: Vec<(String, String)>,
}

/// A full scheduling type:
/// Ex. `i64<storage, map_wrtie, align=8> @ [node(val.x):usable, node(space.y):usable, none(time.x):usable]`
#[derive(Clone, Debug)]
#[allow(unused)]
pub struct FullType {
    pub base: FlaggedType,
    /// Tags, can be empty
    pub tags: Vec<Tag>,
}

/// A function arguments or its encoded statement
/// A function can either be called with a list of arguments or with an encoded
/// statement, but not both
#[derive(Clone, Debug)]
pub enum ArgsOrEnc {
    Args(Vec<SchedExpr>),
    Encode(EncodedStmt),
}

/// A list of expressions or a type
/// Used for template arguments
#[derive(Clone, Debug)]
pub enum TemplateArgs {
    Vals(Vec<SpecExpr>),
    Type(FlaggedType),
}

pub type Tags = Vec<Tag>;

/// A function call in the scheduling language.
/// Can be a procedure (no return value) or a function (has a return value).
/// Can be an encoded statement or have a list of arguments.
#[derive(Clone, Debug)]
#[allow(unused)]
pub struct SchedFuncCall {
    pub target: Box<SchedExpr>,
    pub templates: Option<TemplateArgs>,
    pub args: Box<ArgsOrEnc>,
    pub tag: Option<Tags>,
}

/// A term (bottom level) of a scheduling expression
#[derive(Clone, Debug)]
pub enum SchedTerm {
    Lit {
        info: Info,
        lit: SchedLiteral,
        tag: Option<Tags>,
    },
    Var {
        info: Info,
        name: Name,
        tag: Option<Tags>,
    },
    Call(Info, SchedFuncCall),
    Hole(Info),
}

/// A scheduling expression
pub type SchedExpr = NestedExpr<SchedTerm>;

/// An encoded statement in the scheduling language
/// Ex. `e.encode_copy[x <- y]` the `x <- y` is the encoded statement
#[derive(Clone, Debug)]
pub enum EncodedStmt {
    Move {
        info: Info,
        lhs: Vec<(Name, Option<FlaggedType>)>,
        rhs: SchedExpr,
    },
    Invoke(Info, SchedFuncCall),
}

/// Statements for the scheduling language
#[derive(Clone, Debug)]
pub enum SchedStmt {
    Decl {
        info: Info,
        lhs: Vec<(Name, Option<FullType>)>,
        is_const: bool,
        expr: SchedExpr,
    },
    Assign {
        info: Info,
        lhs: Name,
        rhs: SchedExpr,
    },
    If {
        info: Info,
        guard: SchedExpr,
        true_block: Vec<SchedStmt>,
        false_block: Vec<SchedStmt>,
    },
    Block(Info, Vec<SchedStmt>),
    Return(Info, SchedExpr),
    Hole(Info),
    Call(Info, SchedFuncCall),
}

/// A scheduling function
#[derive(Clone, Debug)]
pub struct SchedulingFunc {
    pub info: Info,
    pub name: String,
    pub input: Vec<Arg<FullType>>,
    pub output: Option<FullType>,
    pub specs: Vec<String>,
    pub statements: Vec<SchedStmt>,
}

/// Input or output binding of an extern function resource
#[derive(Clone, Debug)]
pub enum InputOrOutputVal {
    Input(Name),
    Output(Name),
}

/// A resource used by an extern function
#[derive(Clone, Debug)]
pub struct ExternResource {
    pub binding: usize,
    pub group: usize,
    pub caiman_val: InputOrOutputVal,
}

/// An enum for parsing resource members of an extern function
/// in any order
pub enum ResourceMembers {
    Numeric(String, String),
    Input(Name),
    Output(Name),
}

impl std::fmt::Display for ResourceMembers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Numeric(name, val) => write!(f, "{name}: {val}"),
            Self::Input(val) => {
                write!(f, "input: {val}")
            }
            Self::Output(val) => {
                write!(f, "output: {val}")
            }
        }
    }
}

/// Definition of an extern function
/// Ex:
///
/// ```
/// path : "gpu_external.comp",
/// entry : "main",
/// dimensionality : 3,
/// resource {
///     group : 0,
///     binding : 0,
///     input : %x
/// },
/// resource {
///     group : 0,
///     binding : 1,
///     output : %out
/// }
/// ```
#[derive(Clone, Debug)]
pub struct ExternDef {
    pub path: String,
    pub entry: String,
    pub dimensions: usize,
    pub resources: Vec<ExternResource>,
}

/// An enum for parsing extern members in any order without taking up extra keywords
pub enum ExternDefMembers {
    StrVal(String, String),
    Dimensions(String, String),
    Resource(ExternResource),
}

impl std::fmt::Display for ExternDefMembers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dimensions(name, val) | Self::StrVal(name, val) => write!(f, "{name}: {val}"),
            Self::Resource(_) => Result::Ok(()),
        }
    }
}

/// Function class members. Either an extern function or a value functlet
#[derive(Clone, Debug)]
pub enum ClassMembers {
    ValueFunclet {
        info: Info,
        name: String,
        input: Vec<Arg<DataType>>,
        output: Option<(Option<String>, DataType)>,
        statements: Vec<SpecStmt>,
    },
    Extern {
        info: Info,
        name: String,
        device: String,
        pure: bool,
        input: Vec<(Option<String>, DataType)>,
        output: Option<(Option<String>, DataType)>,
        def: Option<ExternDef>,
    },
}

impl ClassMembers {
    /// Get's the name of the function
    #[must_use]
    pub fn get_name(&self) -> String {
        match self {
            Self::ValueFunclet { name, .. } | Self::Extern { name, .. } => name.clone(),
        }
    }

    /// Get's the source info of the function
    #[must_use]
    pub const fn get_info(&self) -> Info {
        match self {
            Self::ValueFunclet { info, .. } | Self::Extern { info, .. } => *info,
        }
    }

    /// Gets a tuple of the input and output types of the function
    #[must_use]
    pub fn get_type_signature(&self) -> (Vec<DataType>, Vec<DataType>) {
        match self {
            Self::Extern { input, output, .. } => (
                input.iter().map(|(_, typ)| typ.clone()).collect(),
                output.iter().map(|(_, typ)| typ.clone()).collect(),
            ),
            Self::ValueFunclet { input, output, .. } => (
                input.iter().map(|(_, typ)| typ.clone()).collect(),
                output.iter().map(|(_, typ)| typ.clone()).collect(),
            ),
        }
    }
}

/// A top level statement in the source language
#[derive(Clone, Debug)]
pub enum TopLevel {
    FunctionClass {
        info: Info,
        name: String,
        members: Vec<ClassMembers>,
    },
    TimelineFunclet {
        info: Info,
        name: String,
        input: Vec<Arg<DataType>>,
        output: NamedOutput<DataType>,
        statements: Vec<SpecStmt>,
    },
    SpatialFunclet {
        info: Info,
        name: String,
        input: Vec<Arg<DataType>>,
        output: NamedOutput<DataType>,
        statements: Vec<SpecStmt>,
    },
    SchedulingFunc {
        info: Info,
        name: String,
        input: Vec<Arg<FullType>>,
        output: Option<FullType>,
        specs: Vec<String>,
        statements: Vec<SchedStmt>,
    },
    Pipeline {
        info: Info,
        name: String,
        entry: String,
    },
    Typedef {
        info: Info,
        name: String,
        typ: FlaggedType,
    },
    Const {
        info: Info,
        name: String,
        expr: SpecExpr,
    },
}

pub type Program = Vec<TopLevel>;

#![allow(clippy::redundant_field_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::unused_self, clippy::option_option)]
use std::iter;

use lalrpop_util::{ParseError, lexer::Token};

use super::ast::*;
use crate::error::{Info, CustomParsingError};
use crate::custom_parse_error;

const MAJOR_VERSION: &str = "0";
const MINOR_VERSION: &str = "1";
const PATCH_VERSION: &str = "0";

/// A macro for creating tuple-like enum variants that handles the passing along
/// of location information.
///
/// Has the format:
/// ```ignore
/// function_name(<fn_args>) -> <Enum Type>:<Enum Variant>
/// ```
///
/// ### Examples:
/// ```ignore
/// tuple_variant_factory!(sched_returns(e: SchedExpr) -> SchedStmt:SchedStmt::Return);
/// ```
macro_rules! tuple_variant_factory {
    ($f:ident ( $($x:ident : $t:ty),* ) -> $rt:ty:$var:path) => {
        #[must_use]
        pub fn $f (&self, l : usize, $($x : $t,)* r : usize) -> $rt {
            $var(self.info(l, r), $($x,)*)
        }
    }
}

/// A macro for creating struct-like enum variants that handles the passing along
/// of location information
///
/// Has the format:
/// ```ignore
/// function_name(<fn_args>) -> <Enum Type>:<Enum Variant>
/// ```
/// OR
/// ```ignore
/// function_name(<fn_args>) -> <Enum Type>:<Enum Variant> {
///     <field_name>: <field_expr>,
///     ...
/// }
/// ```
///
///
/// ### Examples:
/// ````ignore
/// // Pass along the arguments of the function to the enum variant with
/// // the same types and names:
/// struct_variant_factory!(spec_call(function: Name, args: Vec<SpecExpr>)
///     -> SpecExpr:SpecExpr::Call);
///
///
/// // Pass along the arguments of the function to the enum variant with
/// // different types or names:
/// struct_variant_factory!(spec_binop(op: Binop, lhs: SpecExpr, rhs: SpecExpr)
///     -> SpecExpr:SpecExpr::Binop {
///     op: op,
///     lhs: Box::new(lhs),
///     rhs: Box::new(rhs)
/// });
/// ```
macro_rules! struct_variant_factory {
    ($f:ident ( $($x:ident : $t:ty),* ) -> $rt:ty:$var:path) => {
        #[must_use]
        pub fn $f (&self, l : usize, $($x : $t,)* r : usize) -> $rt {
            $var {
                info: self.info(l, r),
                $($x,)*
            }
        }
    };
    ($f:ident ( $($x:ident : $t:ty),* ) -> $rt:ty:$var:path{$($g:ident:$e:expr),*}) => {
        #[must_use]
        pub fn $f (&self, l : usize, $($x : $t,)* r : usize) -> $rt {
            $var {
                info: self.info(l, r),
                $($g: $e,)*
            }
        }
    };
    ($f:ident<$($templates:ident),*>( $($x:ident : $t:ty),* ) -> $rt:ty:$var:path) => {
        #[must_use]
        pub fn $f<$($templates,)*>(&self, l : usize, $($x : $t,)* r : usize) -> $rt {
            $var {
                info: self.info(l, r),
                $($x,)*
            }
        }
    };
    ($f:ident<$($templates:ident),*>( $($x:ident : $t:ty),* ) -> $rt:ty:$var:path{$($g:ident:$e:expr),*}) => {
        #[must_use]
        pub fn $f<$($templates,)*> (&self, l : usize, $($x : $t,)* r : usize) -> $rt {
            $var {
                info: self.info(l, r),
                $($g: $e,)*
            }
        }
    }
}

/// The `ASTFactory` is responsible for constructing AST nodes for each
/// parser state. The Factory keeps track of the byte offsets of each line
/// so it can convert the byte offsets that lalrpop gives us to line
/// and column numbers.
/// 
/// Each factory function in the `ASTFactory` takes the byte offset of the starting
/// and ending byte offsets and converts them into starting and ending line and column
/// numbers. Macros are used to define these functions to avoid repeating the 
/// same passing along of source location information
pub struct ASTFactory {
    line_ending_byte_offsets: Vec<usize>,
}

/// `LALRpop` parsing error using our custom error type, `CustomParsingError`
type ParserError = ParseError<usize, Token<'static>, CustomParsingError>;

impl ASTFactory {

    /// Creates a new `ASTFactory` from a string of caimain frontend code
    #[must_use]
    pub fn new(_filename: &str, s: &str) -> Self {
        Self {
            line_ending_byte_offsets: s
                .as_bytes()
                .iter()
                // add a newline so this works for the last line
                .chain(iter::once(&b'\n'))
                .enumerate()
                .filter_map(|(idx, b)| if *b == b'\n' { Some(idx) } else { None })
                .collect(),
        }
    }

    /// Returns the line and column number of the given byte offset
    /// # Panics
    /// Panics if the byte offset is greater than the length of the string
    #[must_use]
    pub fn line_and_column(&self, u: usize) -> (usize, usize) {
        if let Some(b) = self.line_ending_byte_offsets.last() {
            assert!(u <= *b, "Byte offset too big: {u}");
        }
        self.line_ending_byte_offsets
            .iter()
            .enumerate()
            .map(|(l, c)| (l + 1, c))
            .fold(
                (1, u), // Case where offset is on line one
                |curr, (l, c)| if u > *c { (l + 1, u - c) } else { curr },
            )
    }

    /// Construct an `Info` struct from a start and end byte offset
    /// 
    /// The `Info` struct contains the line and column number of the start and end
    #[must_use]
    pub fn info(&self, l: usize, r: usize) -> Info {
        Info {
            start_ln_and_col: self.line_and_column(l), 
            end_ln_and_col: self.line_and_column(r),
        }
    }

    /// Constructs an external resource from a list of members
    /// # Errors
    /// Returns an error if the resource is missing a binding or group field
    /// # Panics
    /// Panics if somehow we didn't exit early with an error
    pub fn extern_resource(&self, l: usize, v: Vec<ResourceMembers>, r: usize) 
        -> Result<ExternResource, ParserError> 
    {
            let mut binding = None;
            let mut group = None;
            let mut input = None;
            let mut output = None;
            let src_info = self.info(l, r);
            for member in v {
                match member {
                    ResourceMembers::Input(val) => {
                        input = Some(val.clone());
                    }
                    ResourceMembers::Output(val) => {
                        output = Some(val.clone());
                    }
                    ResourceMembers::Numeric(name, val) if name == "binding" => {
                        binding = Some(val.clone());
                    }
                    ResourceMembers::Numeric(name, val) if name == "group" => group = Some(val.clone()),
                    m @ ResourceMembers::Numeric(..) => return Err(custom_parse_error!(src_info, "Invalid member '{}' in extern definition", m)),
                }
            }
            if binding.is_none() {
                return Err(custom_parse_error!(src_info, "Resource at {} missing field \"binding\"", src_info));
            }
            if group.is_none() {
                return Err(custom_parse_error!(src_info, "Resource at {} missing field \"group\"", src_info));
            }
            if input.is_some() && output.is_some() || input.is_none() && output.is_none() {
                return Err(custom_parse_error!(src_info, "Resource at {} must have exactly one input or output field", src_info));
            }
            Ok(ExternResource {
                binding: binding.unwrap().parse().map_err(|e| custom_parse_error!(src_info, "Resource at {} has invalid binding {}", src_info, e))?,
                group: group.unwrap().parse().map_err(|e| custom_parse_error!(src_info, "Resource at {} has invalid group {}", src_info, e))?,
                caiman_val: match (input, output) {
                    (Some(s), None) => InputOrOutputVal::Input(s),
                    (None, Some(s)) => InputOrOutputVal::Output(s),
                    _ => panic!("Resource at {src_info} must have exactly one input or output"),
                },
            })
    }

    /// Constructs an extern definition from a list of members
    /// # Errors
    /// Returns an error if the definition is missing a path, entry, or dimensions field
    /// # Panics
    /// Panics if somehow we didn't exit early with an error
    pub fn extern_def(&self, l: usize, members: Vec<ExternDefMembers>, r: usize) 
        -> Result<ExternDef, ParserError> 
    {
        let info = self.info(l, r);
        let mut def = ExternDef {
            path: String::new(),
            entry: String::new(),
            dimensions: usize::MAX,
            resources: Vec::new(),
        };
        for mem in members {
            match mem {
                ExternDefMembers::StrVal(key, s) if key == "path" => def.path = s,
                ExternDefMembers::StrVal(key, s) if key == "entry" => def.entry = s,
                ExternDefMembers::Dimensions(key, s) if key == "dimensions" => {
                    def.dimensions = s.parse().unwrap();
                }
                ExternDefMembers::Resource(r) => def.resources.push(r),
                x => {
                    return Err(custom_parse_error!(info, "Extern definition at {} has invalid member {}", info, x));
                }
            }
        }
        if def.path.is_empty() {
            return Err(custom_parse_error!(info, "Extern definition at {} missing field \"path\"", info));
        }
        if def.entry.is_empty() {
            return Err(custom_parse_error!(info, "Extern definition at {} missing field \"path\"", info));
        }
        if def.dimensions == usize::MAX {
            return Err(custom_parse_error!(info, "Extern definition at {} missing field \"dimensions\"", info));
        }
        Ok(def)
    }

    /// Checks that an expression is, (syntactically), a valid constant expression
    /// and returns it if so. Otherwise, returns an error.
    /// # Errors
    /// Returns an error if the expression is not a constant expression
    pub fn const_expr(&self, expr: SpecExpr) -> Result<SpecExpr, ParserError> {
        fn sanitize_expr(expr: &SpecExpr) -> Result<(), ParserError> {
            match expr {
                SpecExpr::Term(t) => {
                    match t {
                        SpecTerm::Lit {lit, ..} => {
                            match lit {
                                SpecLiteral::Int(_) | SpecLiteral::Bool(_) | SpecLiteral::Float(_) => Ok(()),
                                SpecLiteral::Array(a) | SpecLiteral::Tuple(a) => {
                                    for e in a {
                                        sanitize_expr(e)?;
                                    }
                                    Ok(())
                                } 
                            }
                        },
                        SpecTerm::Var { .. } => Ok(()),
                        SpecTerm::Call { info, .. } => Err(custom_parse_error!(*info, 
                            "Non constant expression found in a constant context at {}", info)),
                    }
                },
                SpecExpr::Binop { lhs, rhs, ..} => {
                    sanitize_expr(lhs)?;
                    sanitize_expr(rhs)?;
                    Ok(())
                },
                SpecExpr::Uop { expr, ..} => sanitize_expr(expr),
                SpecExpr::Conditional { if_true, guard, if_false, .. } => {
                    sanitize_expr(if_true)?;
                    sanitize_expr(guard)?;
                    sanitize_expr(if_false)
                }
            }
        }
        sanitize_expr(&expr).map(|_| expr)
    }

    /// Constructs a flagged type from a data type and a list of flags/settings
    /// Flags/settings are optional
    #[must_use]
    pub fn flagged_type(&self, l: usize, t: DataType, flags: Option<Vec<(String, Option<String>)>>, r: usize) -> FlaggedType {
        // are there a limited set of WGPU flags/setting we should check for?
        match flags {
            Some(flags) => FlaggedType {
                info: self.info(l, r),
                base: t,
                flags: flags.iter().filter_map(|(k, v)| 
                    if v.is_none() {
                        Some(k.to_string())
                    } else { None }
                ).collect(),
                settings: flags.iter().filter_map(|(k, v)| {
                    v.as_ref().map(|v| (k.to_string(), v.to_string()))
                }).collect(),
            },
            None => FlaggedType {
                info: self.info(l, r),
                base: t,
                flags: Vec::new(),
                settings: Vec::new(),
            }
        }
    }

    struct_variant_factory!(tag(quot: Option<Quotient>, quot_var: Option<QuotientReference>, flow: Option<Option<Flow>>) -> Tag:Tag {
        quot: quot,
        quot_var: quot_var,
        flow: flow.flatten()
    });

    struct_variant_factory!(flow_tag(quot_var: Option<QuotientReference>, flow: Option<Flow>) -> Tag:Tag {
        quot: None,
        quot_var: quot_var,
        flow: flow
    });

    struct_variant_factory!(import(path: String) -> TopLevel:TopLevel::Import);

    /// Converts a scheduling expression to a specification expression or
    /// returns an error if the expression is invalid in a specification
    /// # Errors
    /// Returns an error if the expression is invalid in a specification
    pub fn sched_to_spec_expr(e: SchedExpr) -> Result<SpecExpr, ParserError> {
        match e {
            SchedExpr::Binop { info, op, lhs, rhs } => {
                let lhs = Self::sched_to_spec_expr(*lhs)?;
                let rhs = Self::sched_to_spec_expr(*rhs)?;
                Ok(SpecExpr::Binop { info, op, lhs: Box::new(lhs), rhs: Box::new(rhs) })
            },
            SchedExpr::Uop { info, op, expr } => {
                let expr = Self::sched_to_spec_expr(*expr)?;
                Ok(SpecExpr::Uop { info, op, expr: Box::new(expr) })
            },
            SchedExpr::Conditional { info, if_true, guard, if_false } => {
                let if_true = Self::sched_to_spec_expr(*if_true)?;
                let guard = Self::sched_to_spec_expr(*guard)?;
                let if_false = Self::sched_to_spec_expr(*if_false)?;
                Ok(SpecExpr::Conditional { info, if_true: Box::new(if_true), guard: Box::new(guard), if_false: Box::new(if_false) })
            },
            SchedExpr::Term(term) => Self::sched_to_spec_term(term).map(SpecExpr::Term),
        }
    }

    /// Converts a scheduling term to a specification term or returns an error
    /// if the term is invalid in a specification
    fn sched_to_spec_term(term: SchedTerm) -> Result<SpecTerm, ParserError> {
        match term {
            SchedTerm::Lit { info, lit, tag } if tag.is_none() => Ok(SpecTerm::Lit { info, lit: Self::sched_to_spec_literal(lit)? }),
            SchedTerm::Var { info, name, tag } if tag.is_none() => Ok(SpecTerm::Var { info, name }),
            SchedTerm::Lit { info, ..} | SchedTerm::Var { info, ..} => Err(custom_parse_error!(info, "A tag cannot occur in this context")),
            SchedTerm::Call(info, SchedFuncCall {
                target,
                templates,
                args,
                tag,
            }) if (templates.is_none() || matches!(templates, Some(TemplateArgs::Type(_)))) && tag.is_none() => {
                let target = Self::sched_to_spec_expr(*target)?;
                match *args {
                   ArgsOrEnc::Encode(_) => Err(custom_parse_error!(info, 
                    "An encoded statement cannot occur in this context")),
                   ArgsOrEnc::Args(args) => {
                        let args = args.into_iter().map(Self::sched_to_spec_expr).collect::<Result<Vec<_>, _>>()?;
                        Ok(SpecTerm::Call { info, function: Box::new(target), args, template: templates.and_then(|t| match t {
                            TemplateArgs::Type(t) => Some(t),
                            TemplateArgs::Vals(_) => None,
                        }) })
                   }
                }
            },
            SchedTerm::Call(info, ..) => Err(custom_parse_error!(info, 
                "Cannot parameterize a function call with non-type template arguments nor specify a tag in this context")),
            SchedTerm::Hole(info) => Err(custom_parse_error!(info, 
                "Holes cannot occur in this context")),
        }
    }

    /// Converts a scheduling literal to a specification literal or returns an
    /// error if the literal is invalid in a specification
    fn sched_to_spec_literal(lit: SchedLiteral) -> Result<SpecLiteral, ParserError> {
        match lit {
            SchedLiteral::Int(i) => Ok(SpecLiteral::Int(i)),
            SchedLiteral::Bool(b) => Ok(SpecLiteral::Bool(b)),
            SchedLiteral::Float(f) => Ok(SpecLiteral::Float(f)),
            SchedLiteral::Array(a) => {
                let a = a.into_iter().map(Self::sched_to_spec_expr).collect::<Result<Vec<_>, _>>()?;
                Ok(SpecLiteral::Array(a))
            },
            SchedLiteral::Tuple(t) => {
                let t = t.into_iter().map(Self::sched_to_spec_expr).collect::<Result<Vec<_>, _>>()?;
                Ok(SpecLiteral::Tuple(t))
            }
        }
    }

    // Nested Exprs

    struct_variant_factory!(binop<T>(lhs: NestedExpr<T>, op: Binop, rhs: NestedExpr<T>) -> NestedExpr<T>:NestedExpr::Binop {
        op: op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs)
    });
    struct_variant_factory!(range<T>(lhs: NestedExpr<T>, rhs: NestedExpr<T>) -> NestedExpr<T>:NestedExpr::Binop { 
        op: Binop::Range,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs)
    });
    struct_variant_factory!(uop<T>(op: Uop, expr: NestedExpr<T>) -> NestedExpr<T>:NestedExpr::Uop {
        op: op,
        expr: Box::new(expr)
    });
    struct_variant_factory!(conditional<T>(if_true: NestedExpr<T>, guard: NestedExpr<T>, 
        if_false: NestedExpr<T>) -> NestedExpr<T>:NestedExpr::Conditional 
    {
        guard: Box::new(guard),
        if_true: Box::new(if_true),
        if_false: Box::new(if_false)
    });

    // Spec Statements

    /// Constructs a declaration in a specification
    /// # Errors
    /// Returns an error if the declaration cannot occur in a specification
    pub fn spec_decl(&self, l: usize, lhs: Vec<(Name, Option<DataType>)>, rhs: SchedExpr, r: usize) -> Result<SpecStmt, ParserError> {
        let rhs = Self::sched_to_spec_expr(rhs)?;
        Ok(SpecStmt::Assign {
            info: self.info(l, r),
            lhs,
            rhs,
        })
    }

    /// Constructs a return in a specification
    /// # Errors
    /// Returns an error if the return cannot occur in a specification
    pub fn spec_returns(&self, l: usize, e: SchedExpr, r: usize) -> Result<SpecStmt, ParserError> {
        let e = Self::sched_to_spec_expr(e)?;
        Ok(SpecStmt::Returns(self.info(l, r), e))
    }

    struct_variant_factory!(spec_lit(lit: SpecLiteral) -> SpecTerm:SpecTerm::Lit);
    struct_variant_factory!(spec_var(name: Name) -> SpecTerm:SpecTerm::Var);

    // scheduling statements

    struct_variant_factory!(sched_in_annotation(tags: Vec<Arg<Tags>>) -> SchedStmt:SchedStmt::InEdgeAnnotation);
    struct_variant_factory!(sched_out_annotation(tags: Vec<Arg<Tags>>) -> SchedStmt:SchedStmt::OutEdgeAnnotation);
    struct_variant_factory!(sched_assign(lhs: Name, tag: Option<Tags>, rhs: SchedExpr) -> SchedStmt:SchedStmt::Assign);
    tuple_variant_factory!(sched_return(e: SchedExpr) -> SchedStmt:SchedStmt::Return);
    tuple_variant_factory!(sched_hole_stmt() -> SchedStmt:SchedStmt::Hole);
    tuple_variant_factory!(sched_call_stmt(call: SchedFuncCall) -> SchedStmt:SchedStmt::Call);
    struct_variant_factory!(sched_if(tags: Option<Tags>, guard: SchedExpr, true_block: Vec<SchedStmt>, 
        false_block: Option<SchedStmt>) -> SchedStmt:SchedStmt::If {
            guard: guard,
            tag: tags,
            true_block: true_block,
            false_block: false_block.map(|x| vec![x]).unwrap_or_default()
        });
    struct_variant_factory!(sched_matched_if(tags: Option<Tags>, guard: SchedExpr, true_block: Vec<SchedStmt>, 
        false_block: SchedStmt) -> SchedStmt:SchedStmt::If {
            guard: guard,
            tag: tags,
            true_block: true_block,
            false_block: vec![false_block]
        });

    tuple_variant_factory!(sched_block(stmts: Vec<SchedStmt>) -> SchedStmt:SchedStmt::Block);

    // scheduling expressions

    struct_variant_factory!(sched_lit(lit: SchedLiteral, tag: Option<Tags>) -> SchedTerm:SchedTerm::Lit);
    struct_variant_factory!(sched_var(name: Name, tag: Option<Tags>) -> SchedTerm:SchedTerm::Var);
    tuple_variant_factory!(sched_hole_expr() -> SchedTerm:SchedTerm::Hole);


    // scheduling function calls:

    /// Constructs a scheduling function call
    #[must_use]
    pub fn sched_fn_call(&self, target: SchedExpr, templates: Option<TemplateArgs>, args: Vec<SchedExpr>, tag: Option<Tags>) 
        -> SchedFuncCall {
        SchedFuncCall {
            target: Box::new(target),
            templates: templates,
            args: Box::new(ArgsOrEnc::Args(args)),
            tag,
        }
    }

    /// Constructs template value arguments for a scheduling function call
    /// # Errors
    /// Returns an error if the templates are not valid in a scheduling context
    pub fn template_args(&self, templates: Vec<SchedExpr>) -> Result<TemplateArgs, ParserError> {
        Ok(TemplateArgs::Vals(templates.into_iter()
            .map(Self::sched_to_spec_expr)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|e| self.const_expr(e))
            .collect::<Result<Vec<_>, _>>()?))
    }

    tuple_variant_factory!(sched_call_expr(call: SchedFuncCall) -> SchedTerm:SchedTerm::Call);

    /// Constructs an encoded statement
    #[must_use]
    pub fn sched_encode(&self, target: SchedExpr, encoding: EncodedStmt, tag: Option<Tags>) 
        -> SchedFuncCall {
            SchedFuncCall {
                target: Box::new(target),
                templates: None,
                args: Box::new(ArgsOrEnc::Encode(encoding)),
                tag,
            }
        }
    
    struct_variant_factory!(sched_let_decl(lhs: Vec<(String, Option<FullType>)>, rhs: SchedExpr) 
        -> SchedStmt:SchedStmt::Decl {
            is_const: true,
            expr: Some(rhs),
            lhs: lhs
        });

    struct_variant_factory!(sched_var_decl(lhs: Vec<(String, Option<FullType>)>, rhs: Option<SchedExpr>) 
        -> SchedStmt:SchedStmt::Decl {
            is_const: false,
            expr: rhs,
            lhs: lhs
        });

    struct_variant_factory!(sched_const_seq(lhs: Vec<(String, Option<FullType>)>, rhs: SchedStmt) 
        -> SchedStmt:SchedStmt::Seq {
            dests: lhs,
            block: Box::new(rhs),
            is_const: true
        });

    struct_variant_factory!(sched_var_seq(lhs: Vec<(String, Option<FullType>)>, rhs: SchedStmt)
        -> SchedStmt:SchedStmt::Seq {
            dests: lhs,
            block: Box::new(rhs),
            is_const: false
        });

    struct_variant_factory!(encoded_stmt(lhs: Vec<(String, Option<FlaggedType>)>, rhs: SchedExpr) -> EncodedStmt:EncodedStmt::Move);
    // TOP-Level:

    struct_variant_factory!(value_funclet(name: String, input: Vec<Arg<DataType>>, 
        output: Option<Vec<NamedOutput<DataType>>>, statements: Vec<SpecStmt>) 
        -> ClassMembers:ClassMembers::ValueFunclet {
            name: name,
            input: input,
            output: output.unwrap_or_default(),
            statements: statements
        });

    struct_variant_factory!(space_funclet(name: String, input: Vec<Arg<DataType>>, 
        output: NamedOutput<DataType>, statements: Vec<SpecStmt>) 
        -> TopLevel:TopLevel::SpatialFunclet);

    struct_variant_factory!(time_funclet(name: String, input: Vec<Arg<DataType>>, 
        output: NamedOutput<DataType>, statements: Vec<SpecStmt>) 
        -> TopLevel:TopLevel::TimelineFunclet);

    struct_variant_factory!(function_class(name: String, members: Vec<ClassMembers>) 
        -> TopLevel:TopLevel::FunctionClass);

    struct_variant_factory!(sched_function(name: String, input: Vec<Arg<FullType>>, 
        output: Option<Vec<FullType>>, specs: Vec<String>, statements: Vec<SchedStmt>) 
        -> TopLevel:TopLevel::SchedulingFunc {
            name: name,
            input: input,
            output: output.unwrap_or_default(),
            specs: specs,
            statements: statements
        });

    struct_variant_factory!(extern_func(device: String, name: String, input: Vec<(Option<String>, DataType)>, 
        output: Option<Vec<NamedOutput<DataType>>>, def: Option<ExternDef>) -> ClassMembers:ClassMembers::Extern {
            device: device,
            def: def,
            name: name,
            input: input,
            output: output.unwrap_or_default(),
            pure: false
        });

    struct_variant_factory!(extern_pure_func(device: String, name: String, input: Vec<(Option<String>, DataType)>, 
        output: Option<Vec<NamedOutput<DataType>>>, def: Option<ExternDef>) -> ClassMembers:ClassMembers::Extern {
            device: device,
            def: def,
            name: name,
            input: input,
            output: output.unwrap_or_default(),
            pure: true
        });
    
    /// Constructs a function class for a single class member (value or external function)
    #[must_use]
    pub fn singleton_function_class(&self, member: ClassMembers) -> TopLevel {
        TopLevel::FunctionClass { info: member.get_info(), name: member.get_name(), members: vec![member] }
    }

    struct_variant_factory!(pipeline(name: String, entry: String) -> TopLevel:TopLevel::Pipeline);

    struct_variant_factory!(type_def(name: Name, typ: FlaggedType) -> TopLevel:TopLevel::Typedef);

    /// Constructs a constant definition from a name and expression. Checks that
    /// the expression is a valid constant expression and returns an error if not
    /// # Errors
    /// Returns an error if the expression is not a constant expression
    pub fn const_def(&self, l: usize, name: Name, expr: SchedExpr, r: usize) -> Result<TopLevel, ParserError> {
        self.const_expr(Self::sched_to_spec_expr(expr)?).map(|expr| TopLevel::Const { info: self.info(l, r), name, expr })
    }

    /// Constructs a program from a list of top level declarations, checking the
    /// version string and returning an error if it is invalid
    /// Constructs a high-level-caiman program
    /// # Errors
    /// Returns an error if the program is not a valid high-level-caiman program
    pub fn program(&self, maj_min: &str, patch: &str, prog: Program) -> Result<Program, ParserError> {
        let split_maj_min: Vec<_> = maj_min.split('.').collect();
        if split_maj_min.len() != 2 {
            return Err(custom_parse_error!(Info {
                start_ln_and_col: (0, 0),
                end_ln_and_col: (0, 0),
            }, "Invalid version string: {}.{}", maj_min, patch));
        }
        let maj = split_maj_min[0];
        let min = split_maj_min[1];
        if (MAJOR_VERSION, MINOR_VERSION, PATCH_VERSION) != (maj, min, patch) {
            return Err(custom_parse_error!(Info {
                start_ln_and_col: (0, 0),
                end_ln_and_col: (0, 0),
            }, "Version mismatch: expected {}.{}.{} but found {}.{}.{}", 
                MAJOR_VERSION, MINOR_VERSION, PATCH_VERSION, maj, min, patch));
        }
        Ok(prog)
    }
}

// auto-generated: "lalrpop 0.19.12"
// sha3: 054dd826e22b8c9533916c1cdc2869c16438d1bba0b52eefecf69f27d07d2440
use super::ast::*;
use super::ast_factory::ASTFactory;
#[allow(unused_extern_crates)]
extern crate lalrpop_util as __lalrpop_util;
#[allow(unused_imports)]
use self::__lalrpop_util::state_machine as __state_machine;
extern crate core;
extern crate alloc;

#[cfg_attr(rustfmt, rustfmt_skip)]
mod __parse__Program {
    #![allow(non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens, clippy::all)]

    use super::super::ast::*;
    use super::super::ast_factory::ASTFactory;
    #[allow(unused_extern_crates)]
    extern crate lalrpop_util as __lalrpop_util;
    #[allow(unused_imports)]
    use self::__lalrpop_util::state_machine as __state_machine;
    extern crate core;
    extern crate alloc;
    use self::__lalrpop_util::lexer::Token;
    #[allow(dead_code)]
    pub(crate) enum __Symbol<'input>
     {
        Variant0(&'input str),
        Variant1(Arg<scheduling::Type>),
        Variant2(alloc::vec::Vec<Arg<scheduling::Type>>),
        Variant3(Arg<timeline::Type>),
        Variant4(alloc::vec::Vec<Arg<timeline::Type>>),
        Variant5(Arg<value::Type>),
        Variant6(alloc::vec::Vec<Arg<value::Type>>),
        Variant7(String),
        Variant8(alloc::vec::Vec<String>),
        Variant9(usize),
        Variant10(core::option::Option<Arg<scheduling::Type>>),
        Variant11(core::option::Option<Arg<timeline::Type>>),
        Variant12(core::option::Option<Arg<value::Type>>),
        Variant13(bool),
        Variant14(Vec<Arg<scheduling::Type>>),
        Variant15(Vec<Arg<timeline::Type>>),
        Variant16(Vec<Arg<value::Type>>),
        Variant17(Vec<String>),
        Variant18(Decl),
        Variant19(alloc::vec::Vec<Decl>),
        Variant20(core::option::Option<String>),
        Variant21(Program),
        Variant22(scheduling::ScheduledExpr),
        Variant23(scheduling::FullSchedulable),
        Variant24(scheduling::Stmt),
        Variant25(alloc::vec::Vec<scheduling::Stmt>),
        Variant26(scheduling::Type),
        Variant27(scheduling::SchedulingFunclet),
        Variant28(alloc::vec::Vec<scheduling::SchedulingFunclet>),
        Variant29((Option<String>, Option<String>)),
        Variant30(core::option::Option<(Option<String>, Option<String>)>),
        Variant31(timeline::Stmt),
        Variant32(alloc::vec::Vec<timeline::Stmt>),
        Variant33(timeline::Type),
        Variant34(value::Expr),
        Variant35((String, value::NumberType)),
        Variant36((Option<String>, value::Type)),
        Variant37(value::Stmt),
        Variant38(alloc::vec::Vec<value::Stmt>),
        Variant39(value::Type),
    }
    const __ACTION: &[i16] = &[
        // State 0
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 51, 0, 0, 0, 3, 0, 0, 4, 0, 0, 5, 0, 6, 0, 0, 0, 0,
        // State 1
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 51, 0, 0, 0, 3, 0, 0, 4, 0, 0, 5, 0, 6, 0, 0, 0, 0,
        // State 2
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 3
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 4
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 5
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 6
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 7
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 8
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 61, 0, 0,
        // State 9
        0, 0, -39, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 10
        0, 0, -43, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 11
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -47, 0, 54,
        // State 12
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 72, 0, 0,
        // State 13
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 14
        0, 0, -41, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 15
        0, 0, -45, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 16
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -49, 0, 54,
        // State 17
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 85, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 18
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 88, 0, 0, 0, 0, 89, 90, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 19
        0, 0, -35, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 20
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 85, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 21
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 88, 0, 0, 0, 0, 89, 90, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 22
        0, 0, -37, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 23
        28, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 24
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 30, 0, 0, 0, 0, 0, 0, 0, 0, 104, 0, 0,
        // State 25
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 33, 0, 0, 0, 0, 0, 0, 0, 106, 0, 0,
        // State 26
        28, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 27
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 28
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 30, 0, 0, 0, 0, 0, 0, 0, 0, 109, 0, 0,
        // State 29
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 30
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 33, 0, 0, 0, 0, 0, 0, 0, 112, 0, 0,
        // State 31
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 32
        0, 35, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 118, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 119, 0, 0, 0, 120, 54,
        // State 33
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 122, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0,
        // State 34
        0, 35, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 118, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 119, 0, 0, 0, 120, 54,
        // State 35
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 42, 0, 43, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0,
        // State 36
        0, 35, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 118, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 119, 0, 0, 0, 120, 54,
        // State 37
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 42, 0, 43, 0, 0, 0, 0, 0, 0, 0, 0, 130, 0, 0,
        // State 38
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 39
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 40
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 42, 0, 43, 0, 0, 0, 0, 0, 0, 0, 0, 134, 0, 0,
        // State 41
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 42
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 43
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 42, 0, 43, 0, 0, 0, 0, 0, 0, 0, 0, 138, 0, 0,
        // State 44
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 45
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 46
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 54,
        // State 47
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 148, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 48
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -60, 0, 0, 0, -60, 0, 0, -60, 0, 0, -60, 0, -60, 0, 0, 0, 0,
        // State 49
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 50
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 51
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -61, 0, 0, 0, -61, 0, 0, -61, 0, 0, -61, 0, -61, 0, 0, 0, 0,
        // State 52
        0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 53
        0, -62, -62, -62, 0, -62, -62, -62, -62, -62, 0, 0, -62, -62, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -62, -62, 0, 0,
        // State 54
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0,
        // State 55
        0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 56
        0, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 57
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0,
        // State 58
        0, 0, 0, 0, 0, 0, 0, 0, 70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 59
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -82, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -82, 0, 0,
        // State 60
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -53, 0, 0, 0, -53, 0, 0, -53, 0, 0, -53, 0, -53, 0, 0, 0, 0,
        // State 61
        0, 0, -38, 75, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 62
        0, 0, 76, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 63
        0, 0, 0, 0, 0, 0, 18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 64
        0, 0, -42, 78, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 65
        0, 0, 79, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 66
        0, 0, 0, 0, 0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 67
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 81, 0, 0,
        // State 68
        0, 0, 0, 82, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -46, 0, 0,
        // State 69
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -57, 0, 0, 0, -57, 0, 0, -57, 0, 0, -57, 0, -57, 0, 0, 0, 0,
        // State 70
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -83, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -83, 0, 0,
        // State 71
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -54, 0, 0, 0, -54, 0, 0, -54, 0, 0, -54, 0, -54, 0, 0, 0, 0,
        // State 72
        0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 73
        0, 0, -40, 83, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 74
        0, 0, -9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -9,
        // State 75
        0, 0, 0, 0, 21, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 76
        0, 0, -44, 86, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 77
        0, 0, -14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -14,
        // State 78
        0, 0, 0, 0, 22, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 79
        0, 0, 0, 91, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -48, 0, 0,
        // State 80
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -52, 0, 0, 0, -52, 0, 0, -52, 0, 0, -52, 0, -52, 0, 0, 0, 0,
        // State 81
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -19, 0, -19,
        // State 82
        0, 0, -10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -10,
        // State 83
        0, 0, -26, -26, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 84
        0, 0, -95, -95, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -95, 0, 0, 0,
        // State 85
        0, 0, -15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -15,
        // State 86
        0, 0, -29, -29, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 87
        0, 0, -110, -110, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -110, 0, 0, 0,
        // State 88
        0, 0, -108, -108, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -108, 0, 0, 0,
        // State 89
        0, 0, -109, -109, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -109, 0, 0, 0,
        // State 90
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -20, 0, -20,
        // State 91
        0, 0, -34, 99, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 92
        0, 0, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 93
        0, 0, 0, 0, 0, 0, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 94
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 25, 0, 0, 0,
        // State 95
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 26, 0, 0, 0,
        // State 96
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -101, 0, 0, 0,
        // State 97
        0, 0, -36, 101, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 98
        0, 0, -4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -4,
        // State 99
        0, 0, 0, 0, 27, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 100
        0, 0, -5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -5,
        // State 101
        0, 0, -23, -23, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 102
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -93, 0, 0, 0, 0, 0, 0, 0, 0, -93, 0, 0,
        // State 103
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -55, 0, 0, 0, -55, 0, 0, -55, 0, 0, -55, 0, -55, 0, 0, 0, 0,
        // State 104
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -106, 0, 0, -106, 0, 0, 0, 0, 0, 0, 0, -106, 0, 0,
        // State 105
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -50, 0, 0, 0, -50, 0, 0, -50, 0, 0, -50, 0, -50, 0, 0, 0, 0,
        // State 106
        0, 0, -75, -75, 0, 0, 0, 0, 0, 0, 0, 0, 0, -75, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -75, 0, 0, 0,
        // State 107
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -94, 0, 0, 0, 0, 0, 0, 0, 0, -94, 0, 0,
        // State 108
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -56, 0, 0, 0, -56, 0, 0, -56, 0, 0, -56, 0, -56, 0, 0, 0, 0,
        // State 109
        0, 0, 0, 0, 0, 0, 0, 0, 123, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 110
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -107, 0, 0, -107, 0, 0, 0, 0, 0, 0, 0, -107, 0, 0,
        // State 111
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -51, 0, 0, 0, -51, 0, 0, -51, 0, 0, -51, 0, -51, 0, 0, 0, 0,
        // State 112
        0, 0, 0, 0, 0, 0, 0, 0, 0, 37, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 113
        0, 0, -97, 0, 0, -97, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 114
        0, 0, -96, 0, 0, -96, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 115
        0, 0, 0, 0, 0, 124, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 116
        0, 0, -98, 0, 0, -98, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 117
        0, 0, -33, 0, 0, -33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 118
        0, 0, -32, 0, 0, -32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 119
        0, 0, -100, 0, 0, -100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 120
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 38, 0, 0, 0,
        // State 121
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 39, 40, 0, 0, 0, 0, 0, 0, 0,
        // State 122
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -90, 0, 0, 0, 0, 0, 0, 0, 0, -90, 0, 0,
        // State 123
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -103, 0, 0, -103, 0, 0, 0, 0, 0, 0, 0, -103, 0, 0,
        // State 124
        0, 0, 129, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 125
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -73, 0, -73, 0, 0, 0, 0, 0, 0, 0, 0, -73, 0, 0,
        // State 126
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -77, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -77, 0, 0,
        // State 127
        0, 0, 0, 0, 0, 137, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 128
        0, 0, -99, 0, 0, -99, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 129
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -76, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -76, 0, 0,
        // State 130
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 139, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -87, 0, 0, 0,
        // State 131
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 140, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -86, 0, 0, 0,
        // State 132
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -74, 0, -74, 0, 0, 0, 0, 0, 0, 0, 0, -74, 0, 0,
        // State 133
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -79, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -79, 0, 0,
        // State 134
        0, 0, 0, 0, 0, 0, 0, 45, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 135
        0, 0, 0, 0, 0, 0, 0, 0, 141, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 136
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -102, 0, 0, -102, 0, 0, 0, 0, 0, 0, 0, -102, 0, 0,
        // State 137
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -78, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -78, 0, 0,
        // State 138
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 46, 0, 0, 0, 0, 0, 0, 0,
        // State 139
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 47, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 140
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -70, 0, -70, 0, 0, 0, 0, 0, 0, 0, 0, -70, 0, 0,
        // State 141
        0, 0, 0, 0, 0, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 142
        0, 0, 0, 0, 0, 0, 0, 0, 146, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 143
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -85, 0, 0, 0,
        // State 144
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -84, 0, 0, 0,
        // State 145
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -69, 0, -69, 0, 0, 0, 0, 0, 0, 0, 0, -69, 0, 0,
        // State 146
        0, 0, 0, 0, 0, 0, 0, 0, -67, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 147
        0, 0, 0, 0, 0, 0, 0, 0, -68, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    fn __action(state: i16, integer: usize) -> i16 {
        __ACTION[(state as usize) * 35 + integer]
    }
    const __EOF_ACTION: &[i16] = &[
        // State 0
        -65,
        // State 1
        -66,
        // State 2
        0,
        // State 3
        0,
        // State 4
        0,
        // State 5
        0,
        // State 6
        0,
        // State 7
        0,
        // State 8
        0,
        // State 9
        0,
        // State 10
        0,
        // State 11
        0,
        // State 12
        0,
        // State 13
        0,
        // State 14
        0,
        // State 15
        0,
        // State 16
        0,
        // State 17
        0,
        // State 18
        0,
        // State 19
        0,
        // State 20
        0,
        // State 21
        0,
        // State 22
        0,
        // State 23
        0,
        // State 24
        0,
        // State 25
        0,
        // State 26
        0,
        // State 27
        0,
        // State 28
        0,
        // State 29
        0,
        // State 30
        0,
        // State 31
        0,
        // State 32
        0,
        // State 33
        0,
        // State 34
        0,
        // State 35
        0,
        // State 36
        0,
        // State 37
        0,
        // State 38
        0,
        // State 39
        0,
        // State 40
        0,
        // State 41
        0,
        // State 42
        0,
        // State 43
        0,
        // State 44
        0,
        // State 45
        0,
        // State 46
        0,
        // State 47
        0,
        // State 48
        -60,
        // State 49
        -111,
        // State 50
        0,
        // State 51
        -61,
        // State 52
        0,
        // State 53
        0,
        // State 54
        0,
        // State 55
        0,
        // State 56
        0,
        // State 57
        0,
        // State 58
        0,
        // State 59
        0,
        // State 60
        -53,
        // State 61
        0,
        // State 62
        0,
        // State 63
        0,
        // State 64
        0,
        // State 65
        0,
        // State 66
        0,
        // State 67
        0,
        // State 68
        0,
        // State 69
        -57,
        // State 70
        0,
        // State 71
        -54,
        // State 72
        0,
        // State 73
        0,
        // State 74
        0,
        // State 75
        0,
        // State 76
        0,
        // State 77
        0,
        // State 78
        0,
        // State 79
        0,
        // State 80
        -52,
        // State 81
        0,
        // State 82
        0,
        // State 83
        0,
        // State 84
        0,
        // State 85
        0,
        // State 86
        0,
        // State 87
        0,
        // State 88
        0,
        // State 89
        0,
        // State 90
        0,
        // State 91
        0,
        // State 92
        0,
        // State 93
        0,
        // State 94
        0,
        // State 95
        0,
        // State 96
        0,
        // State 97
        0,
        // State 98
        0,
        // State 99
        0,
        // State 100
        0,
        // State 101
        0,
        // State 102
        0,
        // State 103
        -55,
        // State 104
        0,
        // State 105
        -50,
        // State 106
        0,
        // State 107
        0,
        // State 108
        -56,
        // State 109
        0,
        // State 110
        0,
        // State 111
        -51,
        // State 112
        0,
        // State 113
        0,
        // State 114
        0,
        // State 115
        0,
        // State 116
        0,
        // State 117
        0,
        // State 118
        0,
        // State 119
        0,
        // State 120
        0,
        // State 121
        0,
        // State 122
        0,
        // State 123
        0,
        // State 124
        0,
        // State 125
        0,
        // State 126
        0,
        // State 127
        0,
        // State 128
        0,
        // State 129
        0,
        // State 130
        0,
        // State 131
        0,
        // State 132
        0,
        // State 133
        0,
        // State 134
        0,
        // State 135
        0,
        // State 136
        0,
        // State 137
        0,
        // State 138
        0,
        // State 139
        0,
        // State 140
        0,
        // State 141
        0,
        // State 142
        0,
        // State 143
        0,
        // State 144
        0,
        // State 145
        0,
        // State 146
        0,
        // State 147
        0,
    ];
    fn __goto(state: i16, nt: usize) -> i16 {
        match nt {
            2 => 22,
            5 => 14,
            8 => 15,
            11 => 16,
            14 => match state {
                22 => 97,
                _ => 91,
            },
            16 => match state {
                14 => 73,
                _ => 61,
            },
            18 => match state {
                15 => 76,
                _ => 64,
            },
            20 => 113,
            21 => 92,
            22 => 62,
            23 => 65,
            24 => 67,
            25 => match state {
                1 => 51,
                _ => 48,
            },
            27 => 1,
            28 => match state {
                2 => 52,
                3 => 54,
                4 => 55,
                5 => 56,
                6 => 57,
                7 => 58,
                9 | 14 => 63,
                10 | 15 => 66,
                11 => 68,
                13 => 72,
                16 => 79,
                19 | 22 => 93,
                27 => 106,
                29 => 109,
                31 => 112,
                38 => 130,
                39 => 131,
                41 => 134,
                42 => 135,
                44 => 141,
                45 => 143,
                46 => 144,
                _ => 114,
            },
            30 => 49,
            31 => 142,
            32 => 146,
            33 => match state {
                40 | 43 => 132,
                _ => 125,
            },
            35 => match state {
                37 => 43,
                _ => 40,
            },
            36 => match state {
                23 => 101,
                _ => 33,
            },
            37 => match state {
                12 => 70,
                _ => 59,
            },
            39 => 12,
            40 => 120,
            42 => match state {
                28 => 107,
                _ => 102,
            },
            44 => 28,
            45 => match state {
                20 => 94,
                _ => 83,
            },
            46 => match state {
                34 => 124,
                36 => 127,
                _ => 115,
            },
            47 => 116,
            48 => 95,
            49 => match state {
                30 => 110,
                _ => 104,
            },
            51 => 30,
            52 => match state {
                21 => 96,
                _ => 86,
            },
            _ => 0,
        }
    }
    fn __expected_tokens(__state: i16) -> alloc::vec::Vec<alloc::string::String> {
        const __TERMINAL: &[&str] = &[
            r###""&""###,
            r###""(""###,
            r###"")""###,
            r###"",""###,
            r###""->""###,
            r###"".""###,
            r###"":""###,
            r###"":=""###,
            r###"";""###,
            r###""=""###,
            r###""Event""###,
            r###""Prim""###,
            r###""and""###,
            r###""at""###,
            r###""bool""###,
            r###""class""###,
            r###""false""###,
            r###""fn""###,
            r###""function""###,
            r###""i32""###,
            r###""i64""###,
            r###""let""###,
            r###""pipeline""###,
            r###""return""###,
            r###""returns""###,
            r###""schedule""###,
            r###""space""###,
            r###""time""###,
            r###""timeline""###,
            r###""true""###,
            r###""value""###,
            r###""{""###,
            r###""}""###,
            r###"r#"[-]?[0-9]+([.][0-9]+)?"#"###,
            r###"r#"[a-zA-Z][a-zA-Z0-9_]*"#"###,
        ];
        __TERMINAL.iter().enumerate().filter_map(|(index, terminal)| {
            let next_state = __action(__state, index);
            if next_state == 0 {
                None
            } else {
                Some(alloc::string::ToString::to_string(terminal))
            }
        }).collect()
    }
    pub(crate) struct __StateMachine<'input, '__1>
    where 
    {
        astf: &'__1 ASTFactory,
        input: &'input str,
        __phantom: core::marker::PhantomData<(&'input ())>,
    }
    impl<'input, '__1> __state_machine::ParserDefinition for __StateMachine<'input, '__1>
    where 
    {
        type Location = usize;
        type Error = &'static str;
        type Token = Token<'input>;
        type TokenIndex = usize;
        type Symbol = __Symbol<'input>;
        type Success = Program;
        type StateIndex = i16;
        type Action = i16;
        type ReduceIndex = i16;
        type NonterminalIndex = usize;

        #[inline]
        fn start_location(&self) -> Self::Location {
              Default::default()
        }

        #[inline]
        fn start_state(&self) -> Self::StateIndex {
              0
        }

        #[inline]
        fn token_to_index(&self, token: &Self::Token) -> Option<usize> {
            __token_to_integer(token, core::marker::PhantomData::<(&())>)
        }

        #[inline]
        fn action(&self, state: i16, integer: usize) -> i16 {
            __action(state, integer)
        }

        #[inline]
        fn error_action(&self, state: i16) -> i16 {
            __action(state, 35 - 1)
        }

        #[inline]
        fn eof_action(&self, state: i16) -> i16 {
            __EOF_ACTION[state as usize]
        }

        #[inline]
        fn goto(&self, state: i16, nt: usize) -> i16 {
            __goto(state, nt)
        }

        fn token_to_symbol(&self, token_index: usize, token: Self::Token) -> Self::Symbol {
            __token_to_symbol(token_index, token, core::marker::PhantomData::<(&())>)
        }

        fn expected_tokens(&self, state: i16) -> alloc::vec::Vec<alloc::string::String> {
            __expected_tokens(state)
        }

        #[inline]
        fn uses_error_recovery(&self) -> bool {
            false
        }

        #[inline]
        fn error_recovery_symbol(
            &self,
            recovery: __state_machine::ErrorRecovery<Self>,
        ) -> Self::Symbol {
            panic!("error recovery not enabled for this grammar")
        }

        fn reduce(
            &mut self,
            action: i16,
            start_location: Option<&Self::Location>,
            states: &mut alloc::vec::Vec<i16>,
            symbols: &mut alloc::vec::Vec<__state_machine::SymbolTriple<Self>>,
        ) -> Option<__state_machine::ParseResult<Self>> {
            __reduce(
                self.astf,
                self.input,
                action,
                start_location,
                states,
                symbols,
                core::marker::PhantomData::<(&())>,
            )
        }

        fn simulate_reduce(&self, action: i16) -> __state_machine::SimulatedReduce<Self> {
            panic!("error recovery not enabled for this grammar")
        }
    }
    fn __token_to_integer<
        'input,
    >(
        __token: &Token<'input>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> Option<usize>
    {
        match *__token {
            Token(0, _) if true => Some(0),
            Token(1, _) if true => Some(1),
            Token(2, _) if true => Some(2),
            Token(3, _) if true => Some(3),
            Token(4, _) if true => Some(4),
            Token(5, _) if true => Some(5),
            Token(6, _) if true => Some(6),
            Token(7, _) if true => Some(7),
            Token(8, _) if true => Some(8),
            Token(9, _) if true => Some(9),
            Token(18, _) if true => Some(10),
            Token(19, _) if true => Some(11),
            Token(20, _) if true => Some(12),
            Token(21, _) if true => Some(13),
            Token(22, _) if true => Some(14),
            Token(23, _) if true => Some(15),
            Token(10, _) if true => Some(16),
            Token(24, _) if true => Some(17),
            Token(25, _) if true => Some(18),
            Token(26, _) if true => Some(19),
            Token(27, _) if true => Some(20),
            Token(28, _) if true => Some(21),
            Token(29, _) if true => Some(22),
            Token(30, _) if true => Some(23),
            Token(31, _) if true => Some(24),
            Token(32, _) if true => Some(25),
            Token(33, _) if true => Some(26),
            Token(34, _) if true => Some(27),
            Token(35, _) if true => Some(28),
            Token(11, _) if true => Some(29),
            Token(36, _) if true => Some(30),
            Token(12, _) if true => Some(31),
            Token(13, _) if true => Some(32),
            Token(15, _) if true => Some(33),
            Token(16, _) if true => Some(34),
            _ => None,
        }
    }
    fn __token_to_symbol<
        'input,
    >(
        __token_index: usize,
        __token: Token<'input>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> __Symbol<'input>
    {
        match __token_index {
            0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 30 | 31 | 32 | 33 | 34 => match __token {
                Token(0, __tok0) | Token(1, __tok0) | Token(2, __tok0) | Token(3, __tok0) | Token(4, __tok0) | Token(5, __tok0) | Token(6, __tok0) | Token(7, __tok0) | Token(8, __tok0) | Token(9, __tok0) | Token(18, __tok0) | Token(19, __tok0) | Token(20, __tok0) | Token(21, __tok0) | Token(22, __tok0) | Token(23, __tok0) | Token(10, __tok0) | Token(24, __tok0) | Token(25, __tok0) | Token(26, __tok0) | Token(27, __tok0) | Token(28, __tok0) | Token(29, __tok0) | Token(30, __tok0) | Token(31, __tok0) | Token(32, __tok0) | Token(33, __tok0) | Token(34, __tok0) | Token(35, __tok0) | Token(11, __tok0) | Token(36, __tok0) | Token(12, __tok0) | Token(13, __tok0) | Token(15, __tok0) | Token(16, __tok0) if true => __Symbol::Variant0(__tok0),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
    pub struct ProgramParser {
        builder: __lalrpop_util::lexer::MatcherBuilder,
        _priv: (),
    }

    impl ProgramParser {
        pub fn new() -> ProgramParser {
            let __builder = super::__intern_token::new_builder();
            ProgramParser {
                builder: __builder,
                _priv: (),
            }
        }

        #[allow(dead_code)]
        pub fn parse<
            'input,
        >(
            &self,
            astf: &ASTFactory,
            input: &'input str,
        ) -> Result<Program, __lalrpop_util::ParseError<usize, Token<'input>, &'static str>>
        {
            let mut __tokens = self.builder.matcher(input);
            __state_machine::Parser::drive(
                __StateMachine {
                    astf,
                    input,
                    __phantom: core::marker::PhantomData::<(&())>,
                },
                __tokens,
            )
        }
    }
    pub(crate) fn __reduce<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __action: i16,
        __lookahead_start: Option<&usize>,
        __states: &mut alloc::vec::Vec<i16>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> Option<Result<Program,__lalrpop_util::ParseError<usize, Token<'input>, &'static str>>>
    {
        let (__pop_states, __nonterminal) = match __action {
            0 => {
                __reduce0(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            1 => {
                __reduce1(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            2 => {
                __reduce2(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            3 => {
                __reduce3(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            4 => {
                __reduce4(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            5 => {
                __reduce5(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            6 => {
                __reduce6(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            7 => {
                __reduce7(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            8 => {
                __reduce8(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            9 => {
                __reduce9(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            10 => {
                __reduce10(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            11 => {
                __reduce11(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            12 => {
                __reduce12(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            13 => {
                __reduce13(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            14 => {
                __reduce14(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            15 => {
                __reduce15(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            16 => {
                __reduce16(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            17 => {
                __reduce17(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            18 => {
                __reduce18(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            19 => {
                __reduce19(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            20 => {
                __reduce20(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            21 => {
                __reduce21(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            22 => {
                __reduce22(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            23 => {
                __reduce23(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            24 => {
                __reduce24(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            25 => {
                __reduce25(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            26 => {
                __reduce26(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            27 => {
                __reduce27(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            28 => {
                __reduce28(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            29 => {
                __reduce29(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            30 => {
                __reduce30(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            31 => {
                __reduce31(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            32 => {
                __reduce32(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            33 => {
                __reduce33(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            34 => {
                __reduce34(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            35 => {
                __reduce35(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            36 => {
                __reduce36(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            37 => {
                __reduce37(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            38 => {
                __reduce38(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            39 => {
                __reduce39(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            40 => {
                __reduce40(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            41 => {
                __reduce41(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            42 => {
                __reduce42(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            43 => {
                __reduce43(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            44 => {
                __reduce44(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            45 => {
                __reduce45(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            46 => {
                __reduce46(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            47 => {
                __reduce47(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            48 => {
                __reduce48(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            49 => {
                __reduce49(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            50 => {
                __reduce50(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            51 => {
                __reduce51(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            52 => {
                __reduce52(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            53 => {
                __reduce53(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            54 => {
                __reduce54(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            55 => {
                __reduce55(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            56 => {
                __reduce56(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            57 => {
                __reduce57(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            58 => {
                __reduce58(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            59 => {
                __reduce59(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            60 => {
                __reduce60(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            61 => {
                __reduce61(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            62 => {
                __reduce62(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            63 => {
                __reduce63(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            64 => {
                __reduce64(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            65 => {
                __reduce65(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            66 => {
                __reduce66(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            67 => {
                __reduce67(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            68 => {
                __reduce68(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            69 => {
                __reduce69(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            70 => {
                __reduce70(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            71 => {
                __reduce71(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            72 => {
                __reduce72(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            73 => {
                __reduce73(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            74 => {
                __reduce74(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            75 => {
                __reduce75(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            76 => {
                __reduce76(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            77 => {
                __reduce77(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            78 => {
                __reduce78(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            79 => {
                __reduce79(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            80 => {
                __reduce80(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            81 => {
                __reduce81(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            82 => {
                __reduce82(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            83 => {
                __reduce83(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            84 => {
                __reduce84(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            85 => {
                __reduce85(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            86 => {
                __reduce86(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            87 => {
                __reduce87(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            88 => {
                __reduce88(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            89 => {
                __reduce89(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            90 => {
                __reduce90(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            91 => {
                __reduce91(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            92 => {
                __reduce92(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            93 => {
                __reduce93(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            94 => {
                __reduce94(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            95 => {
                __reduce95(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            96 => {
                __reduce96(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            97 => {
                __reduce97(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            98 => {
                __reduce98(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            99 => {
                __reduce99(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            100 => {
                __reduce100(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            101 => {
                __reduce101(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            102 => {
                __reduce102(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            103 => {
                __reduce103(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            104 => {
                __reduce104(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            105 => {
                __reduce105(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            106 => {
                __reduce106(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            107 => {
                __reduce107(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            108 => {
                __reduce108(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            109 => {
                __reduce109(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            110 => {
                // __Program = Program => ActionFn(0);
                let __sym0 = __pop_Variant21(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action0::<>(astf, input, __sym0);
                return Some(Ok(__nt));
            }
            _ => panic!("invalid action code {}", __action)
        };
        let __states_len = __states.len();
        __states.truncate(__states_len - __pop_states);
        let __state = *__states.last().unwrap();
        let __next_state = __goto(__state, __nonterminal);
        __states.push(__next_state);
        None
    }
    #[inline(never)]
    fn __symbol_type_mismatch() -> ! {
        panic!("symbol type mismatch")
    }
    fn __pop_Variant29<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, (Option<String>, Option<String>), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant29(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant36<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, (Option<String>, value::Type), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant36(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant35<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, (String, value::NumberType), usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant35(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant1<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Arg<scheduling::Type>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant1(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant3<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Arg<timeline::Type>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant3(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant5<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Arg<value::Type>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant5(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant18<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Decl, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant18(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant21<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Program, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant21(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant7<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, String, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant7(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant14<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Vec<Arg<scheduling::Type>>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant14(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant15<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Vec<Arg<timeline::Type>>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant15(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant16<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Vec<Arg<value::Type>>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant16(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant17<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Vec<String>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant17(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant2<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, alloc::vec::Vec<Arg<scheduling::Type>>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant2(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant4<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, alloc::vec::Vec<Arg<timeline::Type>>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant4(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant6<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, alloc::vec::Vec<Arg<value::Type>>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant6(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant19<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, alloc::vec::Vec<Decl>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant19(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant8<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, alloc::vec::Vec<String>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant8(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant28<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, alloc::vec::Vec<scheduling::SchedulingFunclet>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant28(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant25<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, alloc::vec::Vec<scheduling::Stmt>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant25(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant32<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, alloc::vec::Vec<timeline::Stmt>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant32(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant38<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, alloc::vec::Vec<value::Stmt>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant38(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant13<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, bool, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant13(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant30<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, core::option::Option<(Option<String>, Option<String>)>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant30(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant10<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, core::option::Option<Arg<scheduling::Type>>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant10(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant11<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, core::option::Option<Arg<timeline::Type>>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant11(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant12<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, core::option::Option<Arg<value::Type>>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant12(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant20<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, core::option::Option<String>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant20(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant23<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, scheduling::FullSchedulable, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant23(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant22<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, scheduling::ScheduledExpr, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant22(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant27<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, scheduling::SchedulingFunclet, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant27(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant24<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, scheduling::Stmt, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant24(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant26<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, scheduling::Type, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant26(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant31<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, timeline::Stmt, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant31(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant33<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, timeline::Type, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant33(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant9<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, usize, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant9(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant34<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, value::Expr, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant34(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant37<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, value::Stmt, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant37(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant39<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, value::Type, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant39(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant0<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, &'input str, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant0(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    pub(crate) fn __reduce0<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (<Arg<SchType>> ",") = Arg<SchType>, "," => ActionFn(81);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action81::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (2, 0)
    }
    pub(crate) fn __reduce1<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (<Arg<SchType>> ",")* =  => ActionFn(79);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action79::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (0, 1)
    }
    pub(crate) fn __reduce2<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (<Arg<SchType>> ",")* = (<Arg<SchType>> ",")+ => ActionFn(80);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action80::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 1)
    }
    pub(crate) fn __reduce3<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (<Arg<SchType>> ",")+ = Arg<SchType>, "," => ActionFn(92);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action92::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (2, 2)
    }
    pub(crate) fn __reduce4<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (<Arg<SchType>> ",")+ = (<Arg<SchType>> ",")+, Arg<SchType>, "," => ActionFn(93);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action93::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (3, 2)
    }
    pub(crate) fn __reduce5<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (<Arg<TimeType>> ",") = Arg<TimeType>, "," => ActionFn(74);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action74::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (2, 3)
    }
    pub(crate) fn __reduce6<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (<Arg<TimeType>> ",")* =  => ActionFn(72);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action72::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (0, 4)
    }
    pub(crate) fn __reduce7<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (<Arg<TimeType>> ",")* = (<Arg<TimeType>> ",")+ => ActionFn(73);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action73::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (1, 4)
    }
    pub(crate) fn __reduce8<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (<Arg<TimeType>> ",")+ = Arg<TimeType>, "," => ActionFn(96);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action96::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (2, 5)
    }
    pub(crate) fn __reduce9<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (<Arg<TimeType>> ",")+ = (<Arg<TimeType>> ",")+, Arg<TimeType>, "," => ActionFn(97);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant3(__symbols);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action97::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (3, 5)
    }
    pub(crate) fn __reduce10<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (<Arg<ValueType>> ",") = Arg<ValueType>, "," => ActionFn(60);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action60::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (2, 6)
    }
    pub(crate) fn __reduce11<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (<Arg<ValueType>> ",")* =  => ActionFn(58);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action58::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (0, 7)
    }
    pub(crate) fn __reduce12<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (<Arg<ValueType>> ",")* = (<Arg<ValueType>> ",")+ => ActionFn(59);
        let __sym0 = __pop_Variant6(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action59::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (1, 7)
    }
    pub(crate) fn __reduce13<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (<Arg<ValueType>> ",")+ = Arg<ValueType>, "," => ActionFn(100);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action100::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (2, 8)
    }
    pub(crate) fn __reduce14<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (<Arg<ValueType>> ",")+ = (<Arg<ValueType>> ",")+, Arg<ValueType>, "," => ActionFn(101);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant5(__symbols);
        let __sym0 = __pop_Variant6(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action101::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (3, 8)
    }
    pub(crate) fn __reduce15<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (<Id> ",") = Id, "," => ActionFn(67);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant7(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action67::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (2, 9)
    }
    pub(crate) fn __reduce16<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (<Id> ",")* =  => ActionFn(65);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action65::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (0, 10)
    }
    pub(crate) fn __reduce17<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (<Id> ",")* = (<Id> ",")+ => ActionFn(66);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action66::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (1, 10)
    }
    pub(crate) fn __reduce18<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (<Id> ",")+ = Id, "," => ActionFn(104);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant7(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action104::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (2, 11)
    }
    pub(crate) fn __reduce19<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // (<Id> ",")+ = (<Id> ",")+, Id, "," => ActionFn(105);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action105::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (3, 11)
    }
    pub(crate) fn __reduce20<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // @L =  => ActionFn(51);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action51::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (0, 12)
    }
    pub(crate) fn __reduce21<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // @R =  => ActionFn(46);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action46::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (0, 13)
    }
    pub(crate) fn __reduce22<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Arg<SchType> = Id, ":", SchType => ActionFn(38);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant26(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant7(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action38::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (3, 14)
    }
    pub(crate) fn __reduce23<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Arg<SchType>? = Arg<SchType> => ActionFn(77);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action77::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (1, 15)
    }
    pub(crate) fn __reduce24<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Arg<SchType>? =  => ActionFn(78);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action78::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (0, 15)
    }
    pub(crate) fn __reduce25<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Arg<TimeType> = Id, ":", TimeType => ActionFn(42);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant33(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant7(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action42::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (3, 16)
    }
    pub(crate) fn __reduce26<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Arg<TimeType>? = Arg<TimeType> => ActionFn(70);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action70::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant11(__nt), __end));
        (1, 17)
    }
    pub(crate) fn __reduce27<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Arg<TimeType>? =  => ActionFn(71);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action71::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant11(__nt), __end));
        (0, 17)
    }
    pub(crate) fn __reduce28<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Arg<ValueType> = Id, ":", ValueType => ActionFn(50);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant39(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant7(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action50::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (3, 18)
    }
    pub(crate) fn __reduce29<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Arg<ValueType>? = Arg<ValueType> => ActionFn(56);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action56::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant12(__nt), __end));
        (1, 19)
    }
    pub(crate) fn __reduce30<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Arg<ValueType>? =  => ActionFn(57);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action57::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant12(__nt), __end));
        (0, 19)
    }
    pub(crate) fn __reduce31<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Bool = "true" => ActionFn(17);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action17::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant13(__nt), __end));
        (1, 20)
    }
    pub(crate) fn __reduce32<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Bool = "false" => ActionFn(18);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action18::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant13(__nt), __end));
        (1, 20)
    }
    pub(crate) fn __reduce33<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // CommaList<Arg<SchType>> = Arg<SchType> => ActionFn(138);
        let __sym0 = __pop_Variant1(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action138::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (1, 21)
    }
    pub(crate) fn __reduce34<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // CommaList<Arg<SchType>> =  => ActionFn(139);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action139::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (0, 21)
    }
    pub(crate) fn __reduce35<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // CommaList<Arg<SchType>> = (<Arg<SchType>> ",")+, Arg<SchType> => ActionFn(140);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action140::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (2, 21)
    }
    pub(crate) fn __reduce36<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // CommaList<Arg<SchType>> = (<Arg<SchType>> ",")+ => ActionFn(141);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action141::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (1, 21)
    }
    pub(crate) fn __reduce37<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // CommaList<Arg<TimeType>> = Arg<TimeType> => ActionFn(142);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action142::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (1, 22)
    }
    pub(crate) fn __reduce38<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // CommaList<Arg<TimeType>> =  => ActionFn(143);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action143::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (0, 22)
    }
    pub(crate) fn __reduce39<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // CommaList<Arg<TimeType>> = (<Arg<TimeType>> ",")+, Arg<TimeType> => ActionFn(144);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant3(__symbols);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action144::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (2, 22)
    }
    pub(crate) fn __reduce40<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // CommaList<Arg<TimeType>> = (<Arg<TimeType>> ",")+ => ActionFn(145);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action145::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (1, 22)
    }
    pub(crate) fn __reduce41<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // CommaList<Arg<ValueType>> = Arg<ValueType> => ActionFn(146);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action146::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant16(__nt), __end));
        (1, 23)
    }
    pub(crate) fn __reduce42<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // CommaList<Arg<ValueType>> =  => ActionFn(147);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action147::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant16(__nt), __end));
        (0, 23)
    }
    pub(crate) fn __reduce43<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // CommaList<Arg<ValueType>> = (<Arg<ValueType>> ",")+, Arg<ValueType> => ActionFn(148);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant5(__symbols);
        let __sym0 = __pop_Variant6(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action148::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant16(__nt), __end));
        (2, 23)
    }
    pub(crate) fn __reduce44<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // CommaList<Arg<ValueType>> = (<Arg<ValueType>> ",")+ => ActionFn(149);
        let __sym0 = __pop_Variant6(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action149::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant16(__nt), __end));
        (1, 23)
    }
    pub(crate) fn __reduce45<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // CommaList<Id> = Id => ActionFn(152);
        let __sym0 = __pop_Variant7(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action152::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant17(__nt), __end));
        (1, 24)
    }
    pub(crate) fn __reduce46<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // CommaList<Id> =  => ActionFn(153);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action153::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant17(__nt), __end));
        (0, 24)
    }
    pub(crate) fn __reduce47<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // CommaList<Id> = (<Id> ",")+, Id => ActionFn(154);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action154::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant17(__nt), __end));
        (2, 24)
    }
    pub(crate) fn __reduce48<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // CommaList<Id> = (<Id> ",")+ => ActionFn(155);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action155::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant17(__nt), __end));
        (1, 24)
    }
    pub(crate) fn __reduce49<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Decl = "value", Id, "(", CommaList<Arg<ValueType>>, ")", "->", ValueOutput, "{", "}" => ActionFn(166);
        assert!(__symbols.len() >= 9);
        let __sym8 = __pop_Variant0(__symbols);
        let __sym7 = __pop_Variant0(__symbols);
        let __sym6 = __pop_Variant36(__symbols);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant16(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym8.2.clone();
        let __nt = super::__action166::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8);
        __symbols.push((__start, __Symbol::Variant18(__nt), __end));
        (9, 25)
    }
    pub(crate) fn __reduce50<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Decl = "value", Id, "(", CommaList<Arg<ValueType>>, ")", "->", ValueOutput, "{", ValueStmt+, "}" => ActionFn(167);
        assert!(__symbols.len() >= 10);
        let __sym9 = __pop_Variant0(__symbols);
        let __sym8 = __pop_Variant38(__symbols);
        let __sym7 = __pop_Variant0(__symbols);
        let __sym6 = __pop_Variant36(__symbols);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant16(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym9.2.clone();
        let __nt = super::__action167::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9);
        __symbols.push((__start, __Symbol::Variant18(__nt), __end));
        (10, 25)
    }
    pub(crate) fn __reduce51<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Decl = "function", "class", Id, "{", CommaList<Id>, "}" => ActionFn(124);
        assert!(__symbols.len() >= 6);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant17(__symbols);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant7(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym5.2.clone();
        let __nt = super::__action124::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5);
        __symbols.push((__start, __Symbol::Variant18(__nt), __end));
        (6, 25)
    }
    pub(crate) fn __reduce52<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Decl = "schedule", Id, "{", "}" => ActionFn(158);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action158::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant18(__nt), __end));
        (4, 25)
    }
    pub(crate) fn __reduce53<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Decl = "schedule", Id, "{", SchedulingFunclet+, "}" => ActionFn(159);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant28(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action159::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant18(__nt), __end));
        (5, 25)
    }
    pub(crate) fn __reduce54<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Decl = "timeline", Id, "(", CommaList<Arg<TimeType>>, ")", "->", TimeType, "{", "}" => ActionFn(164);
        assert!(__symbols.len() >= 9);
        let __sym8 = __pop_Variant0(__symbols);
        let __sym7 = __pop_Variant0(__symbols);
        let __sym6 = __pop_Variant33(__symbols);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant15(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym8.2.clone();
        let __nt = super::__action164::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8);
        __symbols.push((__start, __Symbol::Variant18(__nt), __end));
        (9, 25)
    }
    pub(crate) fn __reduce55<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Decl = "timeline", Id, "(", CommaList<Arg<TimeType>>, ")", "->", TimeType, "{", TimeStmt+, "}" => ActionFn(165);
        assert!(__symbols.len() >= 10);
        let __sym9 = __pop_Variant0(__symbols);
        let __sym8 = __pop_Variant32(__symbols);
        let __sym7 = __pop_Variant0(__symbols);
        let __sym6 = __pop_Variant33(__symbols);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant15(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym9.2.clone();
        let __nt = super::__action165::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9);
        __symbols.push((__start, __Symbol::Variant18(__nt), __end));
        (10, 25)
    }
    pub(crate) fn __reduce56<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Decl = "pipeline", Id, "=", Id, ";" => ActionFn(127);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant7(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action127::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant18(__nt), __end));
        (5, 25)
    }
    pub(crate) fn __reduce57<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Decl* =  => ActionFn(52);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action52::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant19(__nt), __end));
        (0, 26)
    }
    pub(crate) fn __reduce58<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Decl* = Decl+ => ActionFn(53);
        let __sym0 = __pop_Variant19(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action53::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant19(__nt), __end));
        (1, 26)
    }
    pub(crate) fn __reduce59<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Decl+ = Decl => ActionFn(54);
        let __sym0 = __pop_Variant18(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action54::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant19(__nt), __end));
        (1, 27)
    }
    pub(crate) fn __reduce60<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Decl+ = Decl+, Decl => ActionFn(55);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant18(__symbols);
        let __sym0 = __pop_Variant19(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action55::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant19(__nt), __end));
        (2, 27)
    }
    pub(crate) fn __reduce61<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Id = r#"[a-zA-Z][a-zA-Z0-9_]*"# => ActionFn(32);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action32::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 28)
    }
    pub(crate) fn __reduce62<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Id? = Id => ActionFn(63);
        let __sym0 = __pop_Variant7(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action63::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant20(__nt), __end));
        (1, 29)
    }
    pub(crate) fn __reduce63<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Id? =  => ActionFn(64);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action64::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant20(__nt), __end));
        (0, 29)
    }
    pub(crate) fn __reduce64<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Program =  => ActionFn(150);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action150::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant21(__nt), __end));
        (0, 30)
    }
    pub(crate) fn __reduce65<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // Program = Decl+ => ActionFn(151);
        let __sym0 = __pop_Variant19(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action151::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant21(__nt), __end));
        (1, 30)
    }
    pub(crate) fn __reduce66<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // SchExpr = Id, ".", SchFull => ActionFn(128);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant23(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant7(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action128::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant22(__nt), __end));
        (3, 31)
    }
    pub(crate) fn __reduce67<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // SchFull = "Prim" => ActionFn(29);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action29::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant23(__nt), __end));
        (1, 32)
    }
    pub(crate) fn __reduce68<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // SchStmt = "let", Id, ":=", SchExpr, ";" => ActionFn(129);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant22(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action129::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant24(__nt), __end));
        (5, 33)
    }
    pub(crate) fn __reduce69<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // SchStmt = "return", Id, ";" => ActionFn(130);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action130::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant24(__nt), __end));
        (3, 33)
    }
    pub(crate) fn __reduce70<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // SchStmt* =  => ActionFn(33);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action33::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant25(__nt), __end));
        (0, 34)
    }
    pub(crate) fn __reduce71<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // SchStmt* = SchStmt+ => ActionFn(34);
        let __sym0 = __pop_Variant25(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action34::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant25(__nt), __end));
        (1, 34)
    }
    pub(crate) fn __reduce72<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // SchStmt+ = SchStmt => ActionFn(82);
        let __sym0 = __pop_Variant24(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action82::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant25(__nt), __end));
        (1, 35)
    }
    pub(crate) fn __reduce73<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // SchStmt+ = SchStmt+, SchStmt => ActionFn(83);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant24(__symbols);
        let __sym0 = __pop_Variant25(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action83::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant25(__nt), __end));
        (2, 35)
    }
    pub(crate) fn __reduce74<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // SchType = "&", Id => ActionFn(25);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action25::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant26(__nt), __end));
        (2, 36)
    }
    pub(crate) fn __reduce75<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // SchedulingFunclet = "fn", Id, "(", CommaList<Arg<SchType>>, ")", "->", SchType, TimeSpaceTags, "{", "}" => ActionFn(160);
        assert!(__symbols.len() >= 10);
        let __sym9 = __pop_Variant0(__symbols);
        let __sym8 = __pop_Variant0(__symbols);
        let __sym7 = __pop_Variant29(__symbols);
        let __sym6 = __pop_Variant26(__symbols);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant14(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym9.2.clone();
        let __nt = super::__action160::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9);
        __symbols.push((__start, __Symbol::Variant27(__nt), __end));
        (10, 37)
    }
    pub(crate) fn __reduce76<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // SchedulingFunclet = "fn", Id, "(", CommaList<Arg<SchType>>, ")", "->", SchType, "{", "}" => ActionFn(161);
        assert!(__symbols.len() >= 9);
        let __sym8 = __pop_Variant0(__symbols);
        let __sym7 = __pop_Variant0(__symbols);
        let __sym6 = __pop_Variant26(__symbols);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant14(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym8.2.clone();
        let __nt = super::__action161::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8);
        __symbols.push((__start, __Symbol::Variant27(__nt), __end));
        (9, 37)
    }
    pub(crate) fn __reduce77<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // SchedulingFunclet = "fn", Id, "(", CommaList<Arg<SchType>>, ")", "->", SchType, TimeSpaceTags, "{", SchStmt+, "}" => ActionFn(162);
        assert!(__symbols.len() >= 11);
        let __sym10 = __pop_Variant0(__symbols);
        let __sym9 = __pop_Variant25(__symbols);
        let __sym8 = __pop_Variant0(__symbols);
        let __sym7 = __pop_Variant29(__symbols);
        let __sym6 = __pop_Variant26(__symbols);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant14(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym10.2.clone();
        let __nt = super::__action162::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9, __sym10);
        __symbols.push((__start, __Symbol::Variant27(__nt), __end));
        (11, 37)
    }
    pub(crate) fn __reduce78<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // SchedulingFunclet = "fn", Id, "(", CommaList<Arg<SchType>>, ")", "->", SchType, "{", SchStmt+, "}" => ActionFn(163);
        assert!(__symbols.len() >= 10);
        let __sym9 = __pop_Variant0(__symbols);
        let __sym8 = __pop_Variant25(__symbols);
        let __sym7 = __pop_Variant0(__symbols);
        let __sym6 = __pop_Variant26(__symbols);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant14(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym9.2.clone();
        let __nt = super::__action163::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9);
        __symbols.push((__start, __Symbol::Variant27(__nt), __end));
        (10, 37)
    }
    pub(crate) fn __reduce79<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // SchedulingFunclet* =  => ActionFn(43);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action43::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant28(__nt), __end));
        (0, 38)
    }
    pub(crate) fn __reduce80<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // SchedulingFunclet* = SchedulingFunclet+ => ActionFn(44);
        let __sym0 = __pop_Variant28(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action44::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant28(__nt), __end));
        (1, 38)
    }
    pub(crate) fn __reduce81<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // SchedulingFunclet+ = SchedulingFunclet => ActionFn(68);
        let __sym0 = __pop_Variant27(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action68::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant28(__nt), __end));
        (1, 39)
    }
    pub(crate) fn __reduce82<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // SchedulingFunclet+ = SchedulingFunclet+, SchedulingFunclet => ActionFn(69);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant27(__symbols);
        let __sym0 = __pop_Variant28(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action69::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant28(__nt), __end));
        (2, 39)
    }
    pub(crate) fn __reduce83<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // TimeSpaceTags = "at", "time", Id, "and", "space", Id => ActionFn(21);
        assert!(__symbols.len() >= 6);
        let __sym5 = __pop_Variant7(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant7(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym5.2.clone();
        let __nt = super::__action21::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5);
        __symbols.push((__start, __Symbol::Variant29(__nt), __end));
        (6, 40)
    }
    pub(crate) fn __reduce84<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // TimeSpaceTags = "at", "space", Id, "and", "time", Id => ActionFn(22);
        assert!(__symbols.len() >= 6);
        let __sym5 = __pop_Variant7(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant7(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym5.2.clone();
        let __nt = super::__action22::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5);
        __symbols.push((__start, __Symbol::Variant29(__nt), __end));
        (6, 40)
    }
    pub(crate) fn __reduce85<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // TimeSpaceTags = "at", "time", Id => ActionFn(23);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant7(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action23::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant29(__nt), __end));
        (3, 40)
    }
    pub(crate) fn __reduce86<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // TimeSpaceTags = "at", "space", Id => ActionFn(24);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant7(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action24::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant29(__nt), __end));
        (3, 40)
    }
    pub(crate) fn __reduce87<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // TimeSpaceTags? = TimeSpaceTags => ActionFn(35);
        let __sym0 = __pop_Variant29(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action35::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant30(__nt), __end));
        (1, 41)
    }
    pub(crate) fn __reduce88<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // TimeSpaceTags? =  => ActionFn(36);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action36::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant30(__nt), __end));
        (0, 41)
    }
    pub(crate) fn __reduce89<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // TimeStmt = "return", Id, ";" => ActionFn(132);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action132::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant31(__nt), __end));
        (3, 42)
    }
    pub(crate) fn __reduce90<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // TimeStmt* =  => ActionFn(39);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action39::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant32(__nt), __end));
        (0, 43)
    }
    pub(crate) fn __reduce91<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // TimeStmt* = TimeStmt+ => ActionFn(40);
        let __sym0 = __pop_Variant32(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action40::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant32(__nt), __end));
        (1, 43)
    }
    pub(crate) fn __reduce92<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // TimeStmt+ = TimeStmt => ActionFn(75);
        let __sym0 = __pop_Variant31(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action75::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant32(__nt), __end));
        (1, 44)
    }
    pub(crate) fn __reduce93<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // TimeStmt+ = TimeStmt+, TimeStmt => ActionFn(76);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant31(__symbols);
        let __sym0 = __pop_Variant32(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action76::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant32(__nt), __end));
        (2, 44)
    }
    pub(crate) fn __reduce94<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // TimeType = "Event" => ActionFn(30);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action30::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant33(__nt), __end));
        (1, 45)
    }
    pub(crate) fn __reduce95<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ValueExpr = Id => ActionFn(133);
        let __sym0 = __pop_Variant7(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action133::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant34(__nt), __end));
        (1, 46)
    }
    pub(crate) fn __reduce96<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ValueExpr = Bool => ActionFn(134);
        let __sym0 = __pop_Variant13(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action134::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant34(__nt), __end));
        (1, 46)
    }
    pub(crate) fn __reduce97<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ValueExpr = ValueNumber => ActionFn(135);
        let __sym0 = __pop_Variant35(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action135::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant34(__nt), __end));
        (1, 46)
    }
    pub(crate) fn __reduce98<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ValueExpr = "(", ValueExpr, ")" => ActionFn(16);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant34(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action16::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant34(__nt), __end));
        (3, 46)
    }
    pub(crate) fn __reduce99<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ValueNumber = r#"[-]?[0-9]+([.][0-9]+)?"# => ActionFn(19);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action19::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant35(__nt), __end));
        (1, 47)
    }
    pub(crate) fn __reduce100<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ValueOutput = ValueType => ActionFn(10);
        let __sym0 = __pop_Variant39(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action10::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant36(__nt), __end));
        (1, 48)
    }
    pub(crate) fn __reduce101<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ValueStmt = "let", Id, "=", ValueExpr, "." => ActionFn(136);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant34(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action136::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant37(__nt), __end));
        (5, 49)
    }
    pub(crate) fn __reduce102<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ValueStmt = "returns", ValueExpr, "." => ActionFn(137);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant34(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action137::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant37(__nt), __end));
        (3, 49)
    }
    pub(crate) fn __reduce103<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ValueStmt* =  => ActionFn(47);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action47::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant38(__nt), __end));
        (0, 50)
    }
    pub(crate) fn __reduce104<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ValueStmt* = ValueStmt+ => ActionFn(48);
        let __sym0 = __pop_Variant38(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action48::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant38(__nt), __end));
        (1, 50)
    }
    pub(crate) fn __reduce105<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ValueStmt+ = ValueStmt => ActionFn(61);
        let __sym0 = __pop_Variant37(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action61::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant38(__nt), __end));
        (1, 51)
    }
    pub(crate) fn __reduce106<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ValueStmt+ = ValueStmt+, ValueStmt => ActionFn(62);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant37(__symbols);
        let __sym0 = __pop_Variant38(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action62::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant38(__nt), __end));
        (2, 51)
    }
    pub(crate) fn __reduce107<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ValueType = "i32" => ActionFn(7);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action7::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant39(__nt), __end));
        (1, 52)
    }
    pub(crate) fn __reduce108<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ValueType = "i64" => ActionFn(8);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action8::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant39(__nt), __end));
        (1, 52)
    }
    pub(crate) fn __reduce109<
        'input,
    >(
        astf: &ASTFactory,
        input: &'input str,
        __lookahead_start: Option<&usize>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> (usize, usize)
    {
        // ValueType = "bool" => ActionFn(9);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action9::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant39(__nt), __end));
        (1, 52)
    }
}
pub use self::__parse__Program::ProgramParser;
#[cfg_attr(rustfmt, rustfmt_skip)]
mod __intern_token {
    #![allow(unused_imports)]
    use super::super::ast::*;
    use super::super::ast_factory::ASTFactory;
    #[allow(unused_extern_crates)]
    extern crate lalrpop_util as __lalrpop_util;
    #[allow(unused_imports)]
    use self::__lalrpop_util::state_machine as __state_machine;
    extern crate core;
    extern crate alloc;
    pub fn new_builder() -> __lalrpop_util::lexer::MatcherBuilder {
        let __strs: &[(&str, bool)] = &[
            ("^(\\&)", false),
            ("^(\\()", false),
            ("^(\\))", false),
            ("^(,)", false),
            ("^(\\->)", false),
            ("^(\\.)", false),
            ("^(:)", false),
            ("^(:=)", false),
            ("^(;)", false),
            ("^(=)", false),
            ("^(false)", false),
            ("^(true)", false),
            ("^(\\{)", false),
            ("^(\\})", false),
            ("^(//[\0-\t\u{b}-\u{c}\u{e}-\u{10ffff}]*[\n\r]*)", true),
            ("^([\\-]?[0-9]+([\\.][0-9]+)?)", false),
            ("^([A-Za-z][0-9A-Z_a-z]*)", false),
            ("^([\t-\r \u{85}\u{a0}\u{1680}\u{2000}-\u{200a}\u{2028}-\u{2029}\u{202f}\u{205f}\u{3000}]*)", true),
            ("^(Event)", false),
            ("^(Prim)", false),
            ("^(and)", false),
            ("^(at)", false),
            ("^(bool)", false),
            ("^(class)", false),
            ("^(fn)", false),
            ("^(function)", false),
            ("^(i32)", false),
            ("^(i64)", false),
            ("^(let)", false),
            ("^(pipeline)", false),
            ("^(return)", false),
            ("^(returns)", false),
            ("^(schedule)", false),
            ("^(space)", false),
            ("^(time)", false),
            ("^(timeline)", false),
            ("^(value)", false),
        ];
        __lalrpop_util::lexer::MatcherBuilder::new(__strs.iter().copied()).unwrap()
    }
}
pub(crate) use self::__lalrpop_util::lexer::Token;

#[allow(unused_variables)]
fn __action0<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, Program, usize),
) -> Program
{
    __0
}

#[allow(unused_variables)]
fn __action1<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, alloc::vec::Vec<Decl>, usize),
) -> Program
{
    __0
}

#[allow(unused_variables)]
fn __action2<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __1, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, Vec<Arg<value::Type>>, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __3, _): (usize, (Option<String>, value::Type), usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __4, _): (usize, alloc::vec::Vec<value::Stmt>, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __5, _): (usize, usize, usize),
) -> Decl
{
    astf.value_funclet(__0, __1, __2, __3, __4, __5)
}

#[allow(unused_variables)]
fn __action3<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __1, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, Vec<String>, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __3, _): (usize, usize, usize),
) -> Decl
{
    astf.function_class(__0, __1, __2, __3)
}

#[allow(unused_variables)]
fn __action4<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __1, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, alloc::vec::Vec<scheduling::SchedulingFunclet>, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __3, _): (usize, usize, usize),
) -> Decl
{
    astf.schedule_block(__0, __1, __2, __3)
}

#[allow(unused_variables)]
fn __action5<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __1, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, Vec<Arg<timeline::Type>>, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __3, _): (usize, timeline::Type, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __4, _): (usize, alloc::vec::Vec<timeline::Stmt>, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __5, _): (usize, usize, usize),
) -> Decl
{
    astf.timeline_funclet(__0, __1, __2, __3, __4, __5)
}

#[allow(unused_variables)]
fn __action6<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __1, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __3, _): (usize, usize, usize),
) -> Decl
{
    astf.pipeline(__0, __1, __2, __3)
}

#[allow(unused_variables)]
fn __action7<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> value::Type
{
    value::Type::Num(value::NumberType::I32)
}

#[allow(unused_variables)]
fn __action8<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> value::Type
{
    value::Type::Num(value::NumberType::I64)
}

#[allow(unused_variables)]
fn __action9<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> value::Type
{
    value::Type::Bool
}

#[allow(unused_variables)]
fn __action10<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, t, _): (usize, value::Type, usize),
) -> (Option<String>, value::Type)
{
    (None, t)
}

#[allow(unused_variables)]
fn __action11<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __1, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, value::Expr, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __3, _): (usize, usize, usize),
) -> value::Stmt
{
    astf.value_let(__0, __1, __2, __3)
}

#[allow(unused_variables)]
fn __action12<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __1, _): (usize, value::Expr, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, usize, usize),
) -> value::Stmt
{
    astf.value_returns(__0, __1, __2)
}

#[allow(unused_variables)]
fn __action13<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, __1, _): (usize, String, usize),
    (_, __2, _): (usize, usize, usize),
) -> value::Expr
{
    astf.value_var(__0, __1, __2)
}

#[allow(unused_variables)]
fn __action14<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, __1, _): (usize, bool, usize),
    (_, __2, _): (usize, usize, usize),
) -> value::Expr
{
    astf.value_bool(__0, __1, __2)
}

#[allow(unused_variables)]
fn __action15<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, __1, _): (usize, (String, value::NumberType), usize),
    (_, __2, _): (usize, usize, usize),
) -> value::Expr
{
    astf.value_number(__0, __1, __2)
}

#[allow(unused_variables)]
fn __action16<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, _, _): (usize, &'input str, usize),
    (_, __0, _): (usize, value::Expr, usize),
    (_, _, _): (usize, &'input str, usize),
) -> value::Expr
{
    __0
}

#[allow(unused_variables)]
fn __action17<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> bool
{
    true
}

#[allow(unused_variables)]
fn __action18<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> bool
{
    false
}

#[allow(unused_variables)]
fn __action19<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, n, _): (usize, &'input str, usize),
) -> (String, value::NumberType)
{
    (String::from(n), value::NumberType::I64)
}

#[allow(unused_variables)]
fn __action20<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __1, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, Vec<Arg<scheduling::Type>>, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __3, _): (usize, scheduling::Type, usize),
    (_, __4, _): (usize, core::option::Option<(Option<String>, Option<String>)>, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __5, _): (usize, alloc::vec::Vec<scheduling::Stmt>, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __6, _): (usize, usize, usize),
) -> scheduling::SchedulingFunclet
{
    astf.scheduling_funclet(__0, __1, __2, __3, __4, __5, __6)
}

#[allow(unused_variables)]
fn __action21<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, _, _): (usize, &'input str, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, t, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, s, _): (usize, String, usize),
) -> (Option<String>, Option<String>)
{
    (Some(t), Some(s))
}

#[allow(unused_variables)]
fn __action22<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, _, _): (usize, &'input str, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, s, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, t, _): (usize, String, usize),
) -> (Option<String>, Option<String>)
{
    (Some(t), Some(s))
}

#[allow(unused_variables)]
fn __action23<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, _, _): (usize, &'input str, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, t, _): (usize, String, usize),
) -> (Option<String>, Option<String>)
{
    (Some(t), None)
}

#[allow(unused_variables)]
fn __action24<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, _, _): (usize, &'input str, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, s, _): (usize, String, usize),
) -> (Option<String>, Option<String>)
{
    (None, Some(s))
}

#[allow(unused_variables)]
fn __action25<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, _, _): (usize, &'input str, usize),
    (_, __0, _): (usize, String, usize),
) -> scheduling::Type
{
    scheduling::Type::Slot(__0)
}

#[allow(unused_variables)]
fn __action26<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __1, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, scheduling::ScheduledExpr, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __3, _): (usize, usize, usize),
) -> scheduling::Stmt
{
    astf.sch_let(__0, __1, __2, __3)
}

#[allow(unused_variables)]
fn __action27<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __1, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, usize, usize),
) -> scheduling::Stmt
{
    astf.sch_return(__0, __1, __2)
}

#[allow(unused_variables)]
fn __action28<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, __1, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, scheduling::FullSchedulable, usize),
    (_, __3, _): (usize, usize, usize),
) -> scheduling::ScheduledExpr
{
    astf.sch_expr(__0, __1, __2, __3)
}

#[allow(unused_variables)]
fn __action29<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> scheduling::FullSchedulable
{
    scheduling::FullSchedulable::Primitive
}

#[allow(unused_variables)]
fn __action30<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> timeline::Type
{
    timeline::Type::Event
}

#[allow(unused_variables)]
fn __action31<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __1, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, usize, usize),
) -> timeline::Stmt
{
    astf.time_return(__0, __1, __2)
}

#[allow(unused_variables)]
fn __action32<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> String
{
    String::from(__0)
}

#[allow(unused_variables)]
fn __action33<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> alloc::vec::Vec<scheduling::Stmt>
{
    alloc::vec![]
}

#[allow(unused_variables)]
fn __action34<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<scheduling::Stmt>, usize),
) -> alloc::vec::Vec<scheduling::Stmt>
{
    v
}

#[allow(unused_variables)]
fn __action35<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, (Option<String>, Option<String>), usize),
) -> core::option::Option<(Option<String>, Option<String>)>
{
    Some(__0)
}

#[allow(unused_variables)]
fn __action36<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> core::option::Option<(Option<String>, Option<String>)>
{
    None
}

#[allow(unused_variables)]
fn __action37<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, mut v, _): (usize, alloc::vec::Vec<Arg<scheduling::Type>>, usize),
    (_, e, _): (usize, core::option::Option<Arg<scheduling::Type>>, usize),
) -> Vec<Arg<scheduling::Type>>
{
    match e { 
        None => v, 
        Some(e) => {
            v.push(e);
            v
        },
    }
}

#[allow(unused_variables)]
fn __action38<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, i, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, t, _): (usize, scheduling::Type, usize),
) -> Arg<scheduling::Type>
{
    (i, t)
}

#[allow(unused_variables)]
fn __action39<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> alloc::vec::Vec<timeline::Stmt>
{
    alloc::vec![]
}

#[allow(unused_variables)]
fn __action40<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<timeline::Stmt>, usize),
) -> alloc::vec::Vec<timeline::Stmt>
{
    v
}

#[allow(unused_variables)]
fn __action41<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, mut v, _): (usize, alloc::vec::Vec<Arg<timeline::Type>>, usize),
    (_, e, _): (usize, core::option::Option<Arg<timeline::Type>>, usize),
) -> Vec<Arg<timeline::Type>>
{
    match e { 
        None => v, 
        Some(e) => {
            v.push(e);
            v
        },
    }
}

#[allow(unused_variables)]
fn __action42<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, i, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, t, _): (usize, timeline::Type, usize),
) -> Arg<timeline::Type>
{
    (i, t)
}

#[allow(unused_variables)]
fn __action43<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> alloc::vec::Vec<scheduling::SchedulingFunclet>
{
    alloc::vec![]
}

#[allow(unused_variables)]
fn __action44<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<scheduling::SchedulingFunclet>, usize),
) -> alloc::vec::Vec<scheduling::SchedulingFunclet>
{
    v
}

#[allow(unused_variables)]
fn __action45<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, mut v, _): (usize, alloc::vec::Vec<String>, usize),
    (_, e, _): (usize, core::option::Option<String>, usize),
) -> Vec<String>
{
    match e { 
        None => v, 
        Some(e) => {
            v.push(e);
            v
        },
    }
}

#[allow(unused_variables)]
fn __action46<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    __lookbehind.clone()
}

#[allow(unused_variables)]
fn __action47<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> alloc::vec::Vec<value::Stmt>
{
    alloc::vec![]
}

#[allow(unused_variables)]
fn __action48<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<value::Stmt>, usize),
) -> alloc::vec::Vec<value::Stmt>
{
    v
}

#[allow(unused_variables)]
fn __action49<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, mut v, _): (usize, alloc::vec::Vec<Arg<value::Type>>, usize),
    (_, e, _): (usize, core::option::Option<Arg<value::Type>>, usize),
) -> Vec<Arg<value::Type>>
{
    match e { 
        None => v, 
        Some(e) => {
            v.push(e);
            v
        },
    }
}

#[allow(unused_variables)]
fn __action50<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, i, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, t, _): (usize, value::Type, usize),
) -> Arg<value::Type>
{
    (i, t)
}

#[allow(unused_variables)]
fn __action51<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> usize
{
    __lookahead.clone()
}

#[allow(unused_variables)]
fn __action52<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> alloc::vec::Vec<Decl>
{
    alloc::vec![]
}

#[allow(unused_variables)]
fn __action53<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<Decl>, usize),
) -> alloc::vec::Vec<Decl>
{
    v
}

#[allow(unused_variables)]
fn __action54<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, Decl, usize),
) -> alloc::vec::Vec<Decl>
{
    alloc::vec![__0]
}

#[allow(unused_variables)]
fn __action55<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<Decl>, usize),
    (_, e, _): (usize, Decl, usize),
) -> alloc::vec::Vec<Decl>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
fn __action56<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, Arg<value::Type>, usize),
) -> core::option::Option<Arg<value::Type>>
{
    Some(__0)
}

#[allow(unused_variables)]
fn __action57<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> core::option::Option<Arg<value::Type>>
{
    None
}

#[allow(unused_variables)]
fn __action58<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> alloc::vec::Vec<Arg<value::Type>>
{
    alloc::vec![]
}

#[allow(unused_variables)]
fn __action59<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<Arg<value::Type>>, usize),
) -> alloc::vec::Vec<Arg<value::Type>>
{
    v
}

#[allow(unused_variables)]
fn __action60<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, Arg<value::Type>, usize),
    (_, _, _): (usize, &'input str, usize),
) -> Arg<value::Type>
{
    __0
}

#[allow(unused_variables)]
fn __action61<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, value::Stmt, usize),
) -> alloc::vec::Vec<value::Stmt>
{
    alloc::vec![__0]
}

#[allow(unused_variables)]
fn __action62<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<value::Stmt>, usize),
    (_, e, _): (usize, value::Stmt, usize),
) -> alloc::vec::Vec<value::Stmt>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
fn __action63<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, String, usize),
) -> core::option::Option<String>
{
    Some(__0)
}

#[allow(unused_variables)]
fn __action64<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> core::option::Option<String>
{
    None
}

#[allow(unused_variables)]
fn __action65<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> alloc::vec::Vec<String>
{
    alloc::vec![]
}

#[allow(unused_variables)]
fn __action66<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<String>, usize),
) -> alloc::vec::Vec<String>
{
    v
}

#[allow(unused_variables)]
fn __action67<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
) -> String
{
    __0
}

#[allow(unused_variables)]
fn __action68<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, scheduling::SchedulingFunclet, usize),
) -> alloc::vec::Vec<scheduling::SchedulingFunclet>
{
    alloc::vec![__0]
}

#[allow(unused_variables)]
fn __action69<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<scheduling::SchedulingFunclet>, usize),
    (_, e, _): (usize, scheduling::SchedulingFunclet, usize),
) -> alloc::vec::Vec<scheduling::SchedulingFunclet>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
fn __action70<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, Arg<timeline::Type>, usize),
) -> core::option::Option<Arg<timeline::Type>>
{
    Some(__0)
}

#[allow(unused_variables)]
fn __action71<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> core::option::Option<Arg<timeline::Type>>
{
    None
}

#[allow(unused_variables)]
fn __action72<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> alloc::vec::Vec<Arg<timeline::Type>>
{
    alloc::vec![]
}

#[allow(unused_variables)]
fn __action73<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<Arg<timeline::Type>>, usize),
) -> alloc::vec::Vec<Arg<timeline::Type>>
{
    v
}

#[allow(unused_variables)]
fn __action74<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, Arg<timeline::Type>, usize),
    (_, _, _): (usize, &'input str, usize),
) -> Arg<timeline::Type>
{
    __0
}

#[allow(unused_variables)]
fn __action75<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, timeline::Stmt, usize),
) -> alloc::vec::Vec<timeline::Stmt>
{
    alloc::vec![__0]
}

#[allow(unused_variables)]
fn __action76<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<timeline::Stmt>, usize),
    (_, e, _): (usize, timeline::Stmt, usize),
) -> alloc::vec::Vec<timeline::Stmt>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
fn __action77<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, Arg<scheduling::Type>, usize),
) -> core::option::Option<Arg<scheduling::Type>>
{
    Some(__0)
}

#[allow(unused_variables)]
fn __action78<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> core::option::Option<Arg<scheduling::Type>>
{
    None
}

#[allow(unused_variables)]
fn __action79<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> alloc::vec::Vec<Arg<scheduling::Type>>
{
    alloc::vec![]
}

#[allow(unused_variables)]
fn __action80<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<Arg<scheduling::Type>>, usize),
) -> alloc::vec::Vec<Arg<scheduling::Type>>
{
    v
}

#[allow(unused_variables)]
fn __action81<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, Arg<scheduling::Type>, usize),
    (_, _, _): (usize, &'input str, usize),
) -> Arg<scheduling::Type>
{
    __0
}

#[allow(unused_variables)]
fn __action82<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, scheduling::Stmt, usize),
) -> alloc::vec::Vec<scheduling::Stmt>
{
    alloc::vec![__0]
}

#[allow(unused_variables)]
fn __action83<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<scheduling::Stmt>, usize),
    (_, e, _): (usize, scheduling::Stmt, usize),
) -> alloc::vec::Vec<scheduling::Stmt>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
fn __action84<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, Arg<scheduling::Type>, usize),
) -> alloc::vec::Vec<Arg<scheduling::Type>>
{
    alloc::vec![__0]
}

#[allow(unused_variables)]
fn __action85<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<Arg<scheduling::Type>>, usize),
    (_, e, _): (usize, Arg<scheduling::Type>, usize),
) -> alloc::vec::Vec<Arg<scheduling::Type>>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
fn __action86<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, Arg<timeline::Type>, usize),
) -> alloc::vec::Vec<Arg<timeline::Type>>
{
    alloc::vec![__0]
}

#[allow(unused_variables)]
fn __action87<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<Arg<timeline::Type>>, usize),
    (_, e, _): (usize, Arg<timeline::Type>, usize),
) -> alloc::vec::Vec<Arg<timeline::Type>>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
fn __action88<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, String, usize),
) -> alloc::vec::Vec<String>
{
    alloc::vec![__0]
}

#[allow(unused_variables)]
fn __action89<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<String>, usize),
    (_, e, _): (usize, String, usize),
) -> alloc::vec::Vec<String>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
fn __action90<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, Arg<value::Type>, usize),
) -> alloc::vec::Vec<Arg<value::Type>>
{
    alloc::vec![__0]
}

#[allow(unused_variables)]
fn __action91<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<Arg<value::Type>>, usize),
    (_, e, _): (usize, Arg<value::Type>, usize),
) -> alloc::vec::Vec<Arg<value::Type>>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
fn __action92<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, Arg<scheduling::Type>, usize),
    __1: (usize, &'input str, usize),
) -> alloc::vec::Vec<Arg<scheduling::Type>>
{
    let __start0 = __0.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action81(
        astf,
        input,
        __0,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action84(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action93<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<Arg<scheduling::Type>>, usize),
    __1: (usize, Arg<scheduling::Type>, usize),
    __2: (usize, &'input str, usize),
) -> alloc::vec::Vec<Arg<scheduling::Type>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action81(
        astf,
        input,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action85(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action94<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, core::option::Option<Arg<scheduling::Type>>, usize),
) -> Vec<Arg<scheduling::Type>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action79(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action37(
        astf,
        input,
        __temp0,
        __0,
    )
}

#[allow(unused_variables)]
fn __action95<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<Arg<scheduling::Type>>, usize),
    __1: (usize, core::option::Option<Arg<scheduling::Type>>, usize),
) -> Vec<Arg<scheduling::Type>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action80(
        astf,
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action37(
        astf,
        input,
        __temp0,
        __1,
    )
}

#[allow(unused_variables)]
fn __action96<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, Arg<timeline::Type>, usize),
    __1: (usize, &'input str, usize),
) -> alloc::vec::Vec<Arg<timeline::Type>>
{
    let __start0 = __0.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action74(
        astf,
        input,
        __0,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action86(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action97<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<Arg<timeline::Type>>, usize),
    __1: (usize, Arg<timeline::Type>, usize),
    __2: (usize, &'input str, usize),
) -> alloc::vec::Vec<Arg<timeline::Type>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action74(
        astf,
        input,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action87(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action98<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, core::option::Option<Arg<timeline::Type>>, usize),
) -> Vec<Arg<timeline::Type>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action72(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action41(
        astf,
        input,
        __temp0,
        __0,
    )
}

#[allow(unused_variables)]
fn __action99<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<Arg<timeline::Type>>, usize),
    __1: (usize, core::option::Option<Arg<timeline::Type>>, usize),
) -> Vec<Arg<timeline::Type>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action73(
        astf,
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action41(
        astf,
        input,
        __temp0,
        __1,
    )
}

#[allow(unused_variables)]
fn __action100<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, Arg<value::Type>, usize),
    __1: (usize, &'input str, usize),
) -> alloc::vec::Vec<Arg<value::Type>>
{
    let __start0 = __0.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action60(
        astf,
        input,
        __0,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action90(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action101<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<Arg<value::Type>>, usize),
    __1: (usize, Arg<value::Type>, usize),
    __2: (usize, &'input str, usize),
) -> alloc::vec::Vec<Arg<value::Type>>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action60(
        astf,
        input,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action91(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action102<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, core::option::Option<Arg<value::Type>>, usize),
) -> Vec<Arg<value::Type>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action58(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action49(
        astf,
        input,
        __temp0,
        __0,
    )
}

#[allow(unused_variables)]
fn __action103<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<Arg<value::Type>>, usize),
    __1: (usize, core::option::Option<Arg<value::Type>>, usize),
) -> Vec<Arg<value::Type>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action59(
        astf,
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action49(
        astf,
        input,
        __temp0,
        __1,
    )
}

#[allow(unused_variables)]
fn __action104<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
    __1: (usize, &'input str, usize),
) -> alloc::vec::Vec<String>
{
    let __start0 = __0.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action67(
        astf,
        input,
        __0,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action88(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action105<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<String>, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
) -> alloc::vec::Vec<String>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action67(
        astf,
        input,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action89(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action106<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, core::option::Option<String>, usize),
) -> Vec<String>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action65(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action45(
        astf,
        input,
        __temp0,
        __0,
    )
}

#[allow(unused_variables)]
fn __action107<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<String>, usize),
    __1: (usize, core::option::Option<String>, usize),
) -> Vec<String>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action66(
        astf,
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action45(
        astf,
        input,
        __temp0,
        __1,
    )
}

#[allow(unused_variables)]
fn __action108<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<Arg<value::Type>>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, (Option<String>, value::Type), usize),
    __7: (usize, &'input str, usize),
    __8: (usize, alloc::vec::Vec<value::Stmt>, usize),
    __9: (usize, &'input str, usize),
    __10: (usize, usize, usize),
) -> Decl
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action51(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action2(
        astf,
        input,
        __temp0,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __6,
        __7,
        __8,
        __9,
        __10,
    )
}

#[allow(unused_variables)]
fn __action109<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, &'input str, usize),
    __2: (usize, String, usize),
    __3: (usize, &'input str, usize),
    __4: (usize, Vec<String>, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, usize, usize),
) -> Decl
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action51(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action3(
        astf,
        input,
        __temp0,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __6,
    )
}

#[allow(unused_variables)]
fn __action110<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, alloc::vec::Vec<scheduling::SchedulingFunclet>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, usize, usize),
) -> Decl
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action51(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action4(
        astf,
        input,
        __temp0,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
    )
}

#[allow(unused_variables)]
fn __action111<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<Arg<timeline::Type>>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, timeline::Type, usize),
    __7: (usize, &'input str, usize),
    __8: (usize, alloc::vec::Vec<timeline::Stmt>, usize),
    __9: (usize, &'input str, usize),
    __10: (usize, usize, usize),
) -> Decl
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action51(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action5(
        astf,
        input,
        __temp0,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __6,
        __7,
        __8,
        __9,
        __10,
    )
}

#[allow(unused_variables)]
fn __action112<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, String, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, usize, usize),
) -> Decl
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action51(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action6(
        astf,
        input,
        __temp0,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
    )
}

#[allow(unused_variables)]
fn __action113<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
    __1: (usize, &'input str, usize),
    __2: (usize, scheduling::FullSchedulable, usize),
    __3: (usize, usize, usize),
) -> scheduling::ScheduledExpr
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action51(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action28(
        astf,
        input,
        __temp0,
        __0,
        __1,
        __2,
        __3,
    )
}

#[allow(unused_variables)]
fn __action114<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, scheduling::ScheduledExpr, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, usize, usize),
) -> scheduling::Stmt
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action51(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action26(
        astf,
        input,
        __temp0,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
    )
}

#[allow(unused_variables)]
fn __action115<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, usize, usize),
) -> scheduling::Stmt
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action51(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action27(
        astf,
        input,
        __temp0,
        __0,
        __1,
        __2,
        __3,
    )
}

#[allow(unused_variables)]
fn __action116<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<Arg<scheduling::Type>>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, scheduling::Type, usize),
    __7: (usize, core::option::Option<(Option<String>, Option<String>)>, usize),
    __8: (usize, &'input str, usize),
    __9: (usize, alloc::vec::Vec<scheduling::Stmt>, usize),
    __10: (usize, &'input str, usize),
    __11: (usize, usize, usize),
) -> scheduling::SchedulingFunclet
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action51(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action20(
        astf,
        input,
        __temp0,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __6,
        __7,
        __8,
        __9,
        __10,
        __11,
    )
}

#[allow(unused_variables)]
fn __action117<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, usize, usize),
) -> timeline::Stmt
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action51(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action31(
        astf,
        input,
        __temp0,
        __0,
        __1,
        __2,
        __3,
    )
}

#[allow(unused_variables)]
fn __action118<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
    __1: (usize, usize, usize),
) -> value::Expr
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action51(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action13(
        astf,
        input,
        __temp0,
        __0,
        __1,
    )
}

#[allow(unused_variables)]
fn __action119<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, bool, usize),
    __1: (usize, usize, usize),
) -> value::Expr
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action51(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action14(
        astf,
        input,
        __temp0,
        __0,
        __1,
    )
}

#[allow(unused_variables)]
fn __action120<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, (String, value::NumberType), usize),
    __1: (usize, usize, usize),
) -> value::Expr
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action51(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action15(
        astf,
        input,
        __temp0,
        __0,
        __1,
    )
}

#[allow(unused_variables)]
fn __action121<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, value::Expr, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, usize, usize),
) -> value::Stmt
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action51(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action11(
        astf,
        input,
        __temp0,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
    )
}

#[allow(unused_variables)]
fn __action122<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, value::Expr, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, usize, usize),
) -> value::Stmt
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action51(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action12(
        astf,
        input,
        __temp0,
        __0,
        __1,
        __2,
        __3,
    )
}

#[allow(unused_variables)]
fn __action123<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<Arg<value::Type>>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, (Option<String>, value::Type), usize),
    __7: (usize, &'input str, usize),
    __8: (usize, alloc::vec::Vec<value::Stmt>, usize),
    __9: (usize, &'input str, usize),
) -> Decl
{
    let __start0 = __9.2.clone();
    let __end0 = __9.2.clone();
    let __temp0 = __action46(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action108(
        astf,
        input,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __6,
        __7,
        __8,
        __9,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action124<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, &'input str, usize),
    __2: (usize, String, usize),
    __3: (usize, &'input str, usize),
    __4: (usize, Vec<String>, usize),
    __5: (usize, &'input str, usize),
) -> Decl
{
    let __start0 = __5.2.clone();
    let __end0 = __5.2.clone();
    let __temp0 = __action46(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action109(
        astf,
        input,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action125<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, alloc::vec::Vec<scheduling::SchedulingFunclet>, usize),
    __4: (usize, &'input str, usize),
) -> Decl
{
    let __start0 = __4.2.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action46(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action110(
        astf,
        input,
        __0,
        __1,
        __2,
        __3,
        __4,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action126<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<Arg<timeline::Type>>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, timeline::Type, usize),
    __7: (usize, &'input str, usize),
    __8: (usize, alloc::vec::Vec<timeline::Stmt>, usize),
    __9: (usize, &'input str, usize),
) -> Decl
{
    let __start0 = __9.2.clone();
    let __end0 = __9.2.clone();
    let __temp0 = __action46(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action111(
        astf,
        input,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __6,
        __7,
        __8,
        __9,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action127<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, String, usize),
    __4: (usize, &'input str, usize),
) -> Decl
{
    let __start0 = __4.2.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action46(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action112(
        astf,
        input,
        __0,
        __1,
        __2,
        __3,
        __4,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action128<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
    __1: (usize, &'input str, usize),
    __2: (usize, scheduling::FullSchedulable, usize),
) -> scheduling::ScheduledExpr
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action46(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action113(
        astf,
        input,
        __0,
        __1,
        __2,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action129<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, scheduling::ScheduledExpr, usize),
    __4: (usize, &'input str, usize),
) -> scheduling::Stmt
{
    let __start0 = __4.2.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action46(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action114(
        astf,
        input,
        __0,
        __1,
        __2,
        __3,
        __4,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action130<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
) -> scheduling::Stmt
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action46(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action115(
        astf,
        input,
        __0,
        __1,
        __2,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action131<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<Arg<scheduling::Type>>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, scheduling::Type, usize),
    __7: (usize, core::option::Option<(Option<String>, Option<String>)>, usize),
    __8: (usize, &'input str, usize),
    __9: (usize, alloc::vec::Vec<scheduling::Stmt>, usize),
    __10: (usize, &'input str, usize),
) -> scheduling::SchedulingFunclet
{
    let __start0 = __10.2.clone();
    let __end0 = __10.2.clone();
    let __temp0 = __action46(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action116(
        astf,
        input,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __6,
        __7,
        __8,
        __9,
        __10,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action132<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
) -> timeline::Stmt
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action46(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action117(
        astf,
        input,
        __0,
        __1,
        __2,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action133<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
) -> value::Expr
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action46(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action118(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action134<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, bool, usize),
) -> value::Expr
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action46(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action119(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action135<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, (String, value::NumberType), usize),
) -> value::Expr
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action46(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action120(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action136<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, value::Expr, usize),
    __4: (usize, &'input str, usize),
) -> value::Stmt
{
    let __start0 = __4.2.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action46(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action121(
        astf,
        input,
        __0,
        __1,
        __2,
        __3,
        __4,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action137<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, value::Expr, usize),
    __2: (usize, &'input str, usize),
) -> value::Stmt
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action46(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action122(
        astf,
        input,
        __0,
        __1,
        __2,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action138<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, Arg<scheduling::Type>, usize),
) -> Vec<Arg<scheduling::Type>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action77(
        astf,
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action94(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action139<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Vec<Arg<scheduling::Type>>
{
    let __start0 = __lookbehind.clone();
    let __end0 = __lookahead.clone();
    let __temp0 = __action78(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action94(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action140<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<Arg<scheduling::Type>>, usize),
    __1: (usize, Arg<scheduling::Type>, usize),
) -> Vec<Arg<scheduling::Type>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action77(
        astf,
        input,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action95(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action141<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<Arg<scheduling::Type>>, usize),
) -> Vec<Arg<scheduling::Type>>
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action78(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action95(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action142<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, Arg<timeline::Type>, usize),
) -> Vec<Arg<timeline::Type>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action70(
        astf,
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action98(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action143<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Vec<Arg<timeline::Type>>
{
    let __start0 = __lookbehind.clone();
    let __end0 = __lookahead.clone();
    let __temp0 = __action71(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action98(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action144<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<Arg<timeline::Type>>, usize),
    __1: (usize, Arg<timeline::Type>, usize),
) -> Vec<Arg<timeline::Type>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action70(
        astf,
        input,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action99(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action145<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<Arg<timeline::Type>>, usize),
) -> Vec<Arg<timeline::Type>>
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action71(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action99(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action146<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, Arg<value::Type>, usize),
) -> Vec<Arg<value::Type>>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action56(
        astf,
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action102(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action147<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Vec<Arg<value::Type>>
{
    let __start0 = __lookbehind.clone();
    let __end0 = __lookahead.clone();
    let __temp0 = __action57(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action102(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action148<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<Arg<value::Type>>, usize),
    __1: (usize, Arg<value::Type>, usize),
) -> Vec<Arg<value::Type>>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action56(
        astf,
        input,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action103(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action149<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<Arg<value::Type>>, usize),
) -> Vec<Arg<value::Type>>
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action57(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action103(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action150<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Program
{
    let __start0 = __lookbehind.clone();
    let __end0 = __lookahead.clone();
    let __temp0 = __action52(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action1(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action151<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<Decl>, usize),
) -> Program
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action53(
        astf,
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action1(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action152<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
) -> Vec<String>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action63(
        astf,
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action106(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action153<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Vec<String>
{
    let __start0 = __lookbehind.clone();
    let __end0 = __lookahead.clone();
    let __temp0 = __action64(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action106(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action154<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<String>, usize),
    __1: (usize, String, usize),
) -> Vec<String>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action63(
        astf,
        input,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action107(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action155<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<String>, usize),
) -> Vec<String>
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action64(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action107(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action156<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<Arg<scheduling::Type>>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, scheduling::Type, usize),
    __7: (usize, core::option::Option<(Option<String>, Option<String>)>, usize),
    __8: (usize, &'input str, usize),
    __9: (usize, &'input str, usize),
) -> scheduling::SchedulingFunclet
{
    let __start0 = __8.2.clone();
    let __end0 = __9.0.clone();
    let __temp0 = __action33(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action131(
        astf,
        input,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __6,
        __7,
        __8,
        __temp0,
        __9,
    )
}

#[allow(unused_variables)]
fn __action157<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<Arg<scheduling::Type>>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, scheduling::Type, usize),
    __7: (usize, core::option::Option<(Option<String>, Option<String>)>, usize),
    __8: (usize, &'input str, usize),
    __9: (usize, alloc::vec::Vec<scheduling::Stmt>, usize),
    __10: (usize, &'input str, usize),
) -> scheduling::SchedulingFunclet
{
    let __start0 = __9.0.clone();
    let __end0 = __9.2.clone();
    let __temp0 = __action34(
        astf,
        input,
        __9,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action131(
        astf,
        input,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __6,
        __7,
        __8,
        __temp0,
        __10,
    )
}

#[allow(unused_variables)]
fn __action158<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, &'input str, usize),
) -> Decl
{
    let __start0 = __2.2.clone();
    let __end0 = __3.0.clone();
    let __temp0 = __action43(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action125(
        astf,
        input,
        __0,
        __1,
        __2,
        __temp0,
        __3,
    )
}

#[allow(unused_variables)]
fn __action159<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, alloc::vec::Vec<scheduling::SchedulingFunclet>, usize),
    __4: (usize, &'input str, usize),
) -> Decl
{
    let __start0 = __3.0.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action44(
        astf,
        input,
        __3,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action125(
        astf,
        input,
        __0,
        __1,
        __2,
        __temp0,
        __4,
    )
}

#[allow(unused_variables)]
fn __action160<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<Arg<scheduling::Type>>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, scheduling::Type, usize),
    __7: (usize, (Option<String>, Option<String>), usize),
    __8: (usize, &'input str, usize),
    __9: (usize, &'input str, usize),
) -> scheduling::SchedulingFunclet
{
    let __start0 = __7.0.clone();
    let __end0 = __7.2.clone();
    let __temp0 = __action35(
        astf,
        input,
        __7,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action156(
        astf,
        input,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __6,
        __temp0,
        __8,
        __9,
    )
}

#[allow(unused_variables)]
fn __action161<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<Arg<scheduling::Type>>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, scheduling::Type, usize),
    __7: (usize, &'input str, usize),
    __8: (usize, &'input str, usize),
) -> scheduling::SchedulingFunclet
{
    let __start0 = __6.2.clone();
    let __end0 = __7.0.clone();
    let __temp0 = __action36(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action156(
        astf,
        input,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __6,
        __temp0,
        __7,
        __8,
    )
}

#[allow(unused_variables)]
fn __action162<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<Arg<scheduling::Type>>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, scheduling::Type, usize),
    __7: (usize, (Option<String>, Option<String>), usize),
    __8: (usize, &'input str, usize),
    __9: (usize, alloc::vec::Vec<scheduling::Stmt>, usize),
    __10: (usize, &'input str, usize),
) -> scheduling::SchedulingFunclet
{
    let __start0 = __7.0.clone();
    let __end0 = __7.2.clone();
    let __temp0 = __action35(
        astf,
        input,
        __7,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action157(
        astf,
        input,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __6,
        __temp0,
        __8,
        __9,
        __10,
    )
}

#[allow(unused_variables)]
fn __action163<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<Arg<scheduling::Type>>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, scheduling::Type, usize),
    __7: (usize, &'input str, usize),
    __8: (usize, alloc::vec::Vec<scheduling::Stmt>, usize),
    __9: (usize, &'input str, usize),
) -> scheduling::SchedulingFunclet
{
    let __start0 = __6.2.clone();
    let __end0 = __7.0.clone();
    let __temp0 = __action36(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action157(
        astf,
        input,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __6,
        __temp0,
        __7,
        __8,
        __9,
    )
}

#[allow(unused_variables)]
fn __action164<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<Arg<timeline::Type>>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, timeline::Type, usize),
    __7: (usize, &'input str, usize),
    __8: (usize, &'input str, usize),
) -> Decl
{
    let __start0 = __7.2.clone();
    let __end0 = __8.0.clone();
    let __temp0 = __action39(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action126(
        astf,
        input,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __6,
        __7,
        __temp0,
        __8,
    )
}

#[allow(unused_variables)]
fn __action165<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<Arg<timeline::Type>>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, timeline::Type, usize),
    __7: (usize, &'input str, usize),
    __8: (usize, alloc::vec::Vec<timeline::Stmt>, usize),
    __9: (usize, &'input str, usize),
) -> Decl
{
    let __start0 = __8.0.clone();
    let __end0 = __8.2.clone();
    let __temp0 = __action40(
        astf,
        input,
        __8,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action126(
        astf,
        input,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __6,
        __7,
        __temp0,
        __9,
    )
}

#[allow(unused_variables)]
fn __action166<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<Arg<value::Type>>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, (Option<String>, value::Type), usize),
    __7: (usize, &'input str, usize),
    __8: (usize, &'input str, usize),
) -> Decl
{
    let __start0 = __7.2.clone();
    let __end0 = __8.0.clone();
    let __temp0 = __action47(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action123(
        astf,
        input,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __6,
        __7,
        __temp0,
        __8,
    )
}

#[allow(unused_variables)]
fn __action167<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<Arg<value::Type>>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, (Option<String>, value::Type), usize),
    __7: (usize, &'input str, usize),
    __8: (usize, alloc::vec::Vec<value::Stmt>, usize),
    __9: (usize, &'input str, usize),
) -> Decl
{
    let __start0 = __8.0.clone();
    let __end0 = __8.2.clone();
    let __temp0 = __action48(
        astf,
        input,
        __8,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action123(
        astf,
        input,
        __0,
        __1,
        __2,
        __3,
        __4,
        __5,
        __6,
        __7,
        __temp0,
        __9,
    )
}

pub trait __ToTriple<'input, >
{
    fn to_triple(value: Self) -> Result<(usize,Token<'input>,usize), __lalrpop_util::ParseError<usize, Token<'input>, &'static str>>;
}

impl<'input, > __ToTriple<'input, > for (usize, Token<'input>, usize)
{
    fn to_triple(value: Self) -> Result<(usize,Token<'input>,usize), __lalrpop_util::ParseError<usize, Token<'input>, &'static str>> {
        Ok(value)
    }
}
impl<'input, > __ToTriple<'input, > for Result<(usize, Token<'input>, usize), &'static str>
{
    fn to_triple(value: Self) -> Result<(usize,Token<'input>,usize), __lalrpop_util::ParseError<usize, Token<'input>, &'static str>> {
        match value {
            Ok(v) => Ok(v),
            Err(error) => Err(__lalrpop_util::ParseError::User { error }),
        }
    }
}

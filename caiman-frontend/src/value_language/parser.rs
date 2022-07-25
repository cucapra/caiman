// auto-generated: "lalrpop 0.19.8"
// sha3: c2bd2469d5d9de4debbf1212939e73245e19a352427bd7ac9dbccbf1fb43833c
use crate::value_language::ast::*;
use crate::value_language::ast_factory::ASTFactory;
#[allow(unused_extern_crates)]
extern crate lalrpop_util as __lalrpop_util;
#[allow(unused_imports)]
use self::__lalrpop_util::state_machine as __state_machine;
extern crate core;
extern crate alloc;

#[cfg_attr(rustfmt, rustfmt_skip)]
mod __parse__Program {
    #![allow(non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens, clippy::all)]

    use crate::value_language::ast::*;
    use crate::value_language::ast_factory::ASTFactory;
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
        Variant1(core::option::Option<&'input str>),
        Variant2(ParsedExpr),
        Variant3(alloc::vec::Vec<ParsedExpr>),
        Variant4(String),
        Variant5(alloc::vec::Vec<String>),
        Variant6(usize),
        Variant7(Binop),
        Variant8(bool),
        Variant9(Vec<ParsedExpr>),
        Variant10(Vec<String>),
        Variant11(core::option::Option<ParsedExpr>),
        Variant12(core::option::Option<String>),
        Variant13(ParsedProgram),
        Variant14(ParsedStmt),
        Variant15(alloc::vec::Vec<ParsedStmt>),
        Variant16(Unop),
    }
    const __ACTION: &[i8] = &[
        // State 0
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 4, 0, 5, 0, 44, 6, 0, 7, 0, 0, 0, 0, 45,
        // State 1
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 4, 0, 5, 0, 44, 6, 0, 7, 0, 0, 0, 0, 45,
        // State 2
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 45,
        // State 3
        55, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 0, 57, 0, 0, 0, 0, 58, 0, 17, 0, 0, 59, 45,
        // State 4
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 60, 0, 0, 0, 0, 0, 0, 0, 0, -51,
        // State 5
        55, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 0, 57, 0, 0, 0, 0, 58, 0, 17, 0, 0, 59, 45,
        // State 6
        55, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 0, 57, 0, 0, 0, 0, 58, 0, 17, 0, 0, 59, 45,
        // State 7
        55, 0, 16, -36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 0, 57, 0, 0, 0, 0, 58, 0, 17, 0, 0, 59, 45,
        // State 8
        55, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 0, 57, 0, 0, 0, 0, 58, 0, 17, 0, 0, 59, 45,
        // State 9
        0, -28, 0, -28, 0, 0, -28, 0, 0, 0, -28, 0, 66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -28, -28, -28, 0, 0,
        // State 10
        0, -22, 0, -22, 0, 67, -22, 68, 0, 0, -22, 0, -22, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -22, -22, -22, 0, 0,
        // State 11
        0, -24, 0, -24, 69, -24, -24, -24, 70, 0, -24, 0, -24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -24, -24, -24, 0, 0,
        // State 12
        0, 71, 0, -30, 0, 0, -30, 0, 0, 0, -30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -30, -30, -30, 0, 0,
        // State 13
        0, 0, 0, -31, 0, 0, -31, 0, 0, 0, -31, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -31, 72, -31, 0, 0,
        // State 14
        55, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 0, 57, 0, 0, 0, 0, 58, 0, 17, 0, 0, 59, 45,
        // State 15
        55, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 0, 57, 0, 0, 0, 0, 58, 0, 17, 0, 0, 59, 45,
        // State 16
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 45,
        // State 17
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 45,
        // State 18
        55, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 0, 57, 0, 0, 0, 0, 58, 0, 17, 0, 0, 59, 45,
        // State 19
        55, 0, 16, -38, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 0, 57, 0, 0, 0, 0, 58, 0, 17, 0, 0, 59, 45,
        // State 20
        0, 0, 0, -40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 45,
        // State 21
        55, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 0, 57, 0, 0, 0, 0, 58, 0, 17, 0, 0, 59, 45,
        // State 22
        55, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 0, 57, 0, 0, 0, 0, 58, 0, 17, 0, 0, 59, 45,
        // State 23
        55, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 0, 57, 0, 0, 0, 0, 58, 0, 17, 0, 0, 59, 45,
        // State 24
        55, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 0, 57, 0, 0, 0, 0, 58, 0, 17, 0, 0, 59, 45,
        // State 25
        55, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 0, 57, 0, 0, 0, 0, 58, 0, 17, 0, 0, 59, 45,
        // State 26
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 4, 0, 5, 0, 44, 6, 0, 7, 0, 0, 87, 0, 45,
        // State 27
        55, 0, 16, -36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 0, 57, 0, 0, 0, 0, 58, 0, 17, 0, 0, 59, 45,
        // State 28
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 4, 0, 5, 0, 44, 6, 0, 7, 0, 0, 92, 0, 45,
        // State 29
        0, 0, 0, -42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 45,
        // State 30
        0, -21, 0, -21, 0, 67, -21, 68, 0, 0, -21, 0, -21, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -21, -21, -21, 0, 0,
        // State 31
        0, -23, 0, -23, 69, -23, -23, -23, 70, 0, -23, 0, -23, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -23, -23, -23, 0, 0,
        // State 32
        0, -27, 0, -27, 0, 0, -27, 0, 0, 0, -27, 0, 66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -27, -27, -27, 0, 0,
        // State 33
        0, 71, 0, -29, 0, 0, -29, 0, 0, 0, -29, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -29, -29, -29, 0, 0,
        // State 34
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 4, 0, 5, 0, 44, 6, 0, 7, 0, 0, 98, 0, 45,
        // State 35
        55, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 0, 57, 0, 0, 0, 0, 58, 0, 17, 0, 0, 59, 45,
        // State 36
        55, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 0, 0, 57, 0, 0, 0, 0, 58, 0, 17, 0, 0, 59, 45,
        // State 37
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 4, 0, 5, 0, 44, 6, 0, 7, 0, 0, 103, 0, 45,
        // State 38
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 4, 0, 5, 0, 44, 6, 0, 7, 0, 0, 107, 0, 45,
        // State 39
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 4, 0, 5, 0, 44, 6, 0, 7, 0, 0, 108, 0, 45,
        // State 40
        0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 41
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 42
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -67, -67, 0, -67, 0, -67, -67, 0, -67, 0, 0, -67, 0, -67,
        // State 43
        0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 44
        0, -47, -47, -47, -47, -47, -47, -47, -47, -47, -47, -47, -47, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -47, -47, -47, 0, 0,
        // State 45
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -68, -68, 0, -68, 0, -68, -68, 0, -68, 0, 0, -68, 0, -68,
        // State 46
        0, 0, 21, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 47
        0, 0, 0, -44, 0, 0, -44, 0, 0, 0, -44, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -44, 0, -44, 0, 0,
        // State 48
        0, -71, 0, -71, -71, -71, -71, -71, -71, 0, -71, 0, -71, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -71, -71, -71, 0, 0,
        // State 49
        0, -69, 0, -69, -69, -69, -69, -69, -69, 0, -69, 0, -69, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -69, -69, -69, 0, 0,
        // State 50
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 27, 0, 0, 0, 0,
        // State 51
        0, -70, 28, -70, -70, -70, -70, -70, -70, 0, -70, 0, -70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -70, -70, -70, 0, 0,
        // State 52
        0, -26, 0, -26, -26, -26, -26, -26, -26, 0, -26, 0, -26, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -26, -26, -26, 0, 0,
        // State 53
        0, -75, 0, -75, -75, -75, -75, -75, -75, 0, -75, 0, -75, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -75, -75, -75, 0, 0,
        // State 54
        -78, 0, -78, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -78, 0, 0, -78, 0, 0, 0, 0, -78, 0, -78, 0, 0, -78, -78,
        // State 55
        0, -34, 0, -34, -34, -34, -34, -34, -34, 0, -34, 0, -34, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -34, -34, -34, 0, 0,
        // State 56
        0, 0, 75, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 57
        0, -33, 0, -33, -33, -33, -33, -33, -33, 0, -33, 0, -33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -33, -33, -33, 0, 0,
        // State 58
        0, -43, 0, -43, -43, -43, -43, -43, -43, 0, -43, 0, -43, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -43, -43, -43, 0, 0,
        // State 59
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -50,
        // State 60
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 79, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 61
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 29, 0, 0, 0, 0,
        // State 62
        0, 0, 0, 81, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 63
        0, 0, 0, -35, 0, 0, 82, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 64
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 83, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 65
        -15, 0, -15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -15, 0, 0, -15, 0, 0, 0, 0, -15, 0, -15, 0, 0, -15, -15,
        // State 66
        -16, 0, -16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -16, 0, 0, -16, 0, 0, 0, 0, -16, 0, -16, 0, 0, -16, -16,
        // State 67
        -17, 0, -17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -17, 0, 0, -17, 0, 0, 0, 0, -17, 0, -17, 0, 0, -17, -17,
        // State 68
        -18, 0, -18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -18, 0, 0, -18, 0, 0, 0, 0, -18, 0, -18, 0, 0, -18, -18,
        // State 69
        -19, 0, -19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -19, 0, 0, -19, 0, 0, 0, 0, -19, 0, -19, 0, 0, -19, -19,
        // State 70
        -20, 0, -20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -20, 0, 0, -20, 0, 0, 0, 0, -20, 0, -20, 0, 0, -20, -20,
        // State 71
        -32, 0, -32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -32, 0, 0, -32, 0, 0, 0, 0, -32, 0, -32, 0, 0, -32, -32,
        // State 72
        0, -77, 0, -77, -77, -77, -77, -77, -77, 0, -77, 0, -77, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -77, -77, -77, 0, 0,
        // State 73
        0, 0, 0, 89, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 74
        0, 0, 0, 90, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 75
        0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 76
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 37, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 77
        0, 0, 0, 91, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 78
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -62, -62, 0, -62, 0, -62, -62, 0, -62, 0, 0, -62, 0, -62,
        // State 79
        0, 0, 0, -37, 0, 0, 93, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 80
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 94, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 81
        -6, 0, -6, -6, 0, 0, 0, 0, 0, 0, 0, 0, 0, -6, 0, 0, -6, 0, 0, 0, 0, -6, 0, -6, 0, 0, -6, -6,
        // State 82
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -60, -60, 0, -60, 0, -60, -60, 0, -60, 0, 0, -60, 0, -60,
        // State 83
        0, 0, 0, 96, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 84
        0, 0, 0, -39, 0, 0, 97, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 85
        0, -25, 0, -25, -25, -25, -25, -25, -25, 0, -25, 0, -25, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -25, -25, -25, 0, 0,
        // State 86
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -54, -54, 0, -54, 0, -54, -54, 0, -54, 0, 0, -54, 0, -54,
        // State 87
        0, 0, 0, 99, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 88
        0, -76, 0, -76, -76, -76, -76, -76, -76, 0, -76, 0, -76, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -76, -76, -76, 0, 0,
        // State 89
        0, -72, 0, -72, -72, -72, -72, -72, -72, 0, -72, 0, -72, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -72, -72, -72, 0, 0,
        // State 90
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 102, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 91
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -56, -56, 0, -56, 0, -56, -56, 0, -56, 0, 0, -56, 0, -56,
        // State 92
        -7, 0, -7, -7, 0, 0, 0, 0, 0, 0, 0, 0, 0, -7, 0, 0, -7, 0, 0, 0, 0, -7, 0, -7, 0, 0, -7, -7,
        // State 93
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -61, -61, 0, -61, 0, -61, -61, 0, -61, 0, 0, -61, 0, -61,
        // State 94
        0, 0, 0, -41, 0, 0, 104, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 95
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 39, 0, 0, 0, 0,
        // State 96
        0, 0, 0, -11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -11,
        // State 97
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -55, -55, 0, -55, 0, -55, -55, 0, -55, 0, 0, -55, 0, -55,
        // State 98
        0, -73, 0, -73, -73, -73, -73, -73, -73, 0, -73, 0, -73, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -73, -73, -73, 0, 0,
        // State 99
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 105, 0, 0,
        // State 100
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 106, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 101
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -58, -58, 0, -58, 0, -58, -58, 0, -58, 0, 0, -58, 0, -58,
        // State 102
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -57, -57, 0, -57, 0, -57, -57, 0, -57, 0, 0, -57, 0, -57,
        // State 103
        0, 0, 0, -12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -12,
        // State 104
        0, -74, 0, -74, -74, -74, -74, -74, -74, 0, -74, 0, -74, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -74, -74, -74, 0, 0,
        // State 105
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -59, -59, 0, -59, 0, -59, -59, 0, -59, 0, 0, -59, 0, -59,
        // State 106
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -63, -63, 0, -63, 0, -63, -63, 0, -63, 0, 0, -63, 0, -63,
        // State 107
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -64, -64, 0, -64, 0, -64, -64, 0, -64, 0, 0, -64, 0, -64,
    ];
    fn __action(state: i8, integer: usize) -> i8 {
        __ACTION[(state as usize) * 28 + integer]
    }
    const __EOF_ACTION: &[i8] = &[
        // State 0
        -52,
        // State 1
        -53,
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
        -79,
        // State 42
        -67,
        // State 43
        0,
        // State 44
        0,
        // State 45
        -68,
        // State 46
        0,
        // State 47
        0,
        // State 48
        0,
        // State 49
        0,
        // State 50
        0,
        // State 51
        0,
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
        0,
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
        0,
        // State 70
        0,
        // State 71
        0,
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
        -62,
        // State 79
        0,
        // State 80
        0,
        // State 81
        0,
        // State 82
        -60,
        // State 83
        0,
        // State 84
        0,
        // State 85
        0,
        // State 86
        -54,
        // State 87
        0,
        // State 88
        0,
        // State 89
        0,
        // State 90
        0,
        // State 91
        -56,
        // State 92
        0,
        // State 93
        -61,
        // State 94
        0,
        // State 95
        0,
        // State 96
        0,
        // State 97
        -55,
        // State 98
        0,
        // State 99
        0,
        // State 100
        0,
        // State 101
        -58,
        // State 102
        -57,
        // State 103
        0,
        // State 104
        0,
        // State 105
        -59,
        // State 106
        -63,
        // State 107
        -64,
    ];
    fn __goto(state: i8, nt: usize) -> i8 {
        match nt {
            3 => 19,
            6 => 29,
            9 => 21,
            10 => 22,
            11 => 23,
            12 => 24,
            13 => match state {
                24 => 32,
                _ => 9,
            },
            14 => match state {
                21 => 30,
                _ => 10,
            },
            15 => match state {
                22 => 31,
                _ => 11,
            },
            16 => match state {
                25 => 33,
                _ => 12,
            },
            17 => 13,
            18 => 47,
            19 => 25,
            20 => 48,
            21 => match state {
                27 => 87,
                _ => 62,
            },
            22 => 83,
            23 => 49,
            24 => match state {
                3 => 50,
                5 => 60,
                6 => 61,
                8 => 64,
                15 => 73,
                18 => 77,
                19 => 79,
                35 => 99,
                36 => 100,
                _ => 63,
            },
            26 => match state {
                0..=1 | 26 | 28 | 34 | 37..=39 => 40,
                2 => 46,
                16 => 75,
                17 => 76,
                20 => 84,
                29 => 94,
                _ => 51,
            },
            28 => 17,
            29 => 41,
            30 => match state {
                1 | 34 | 37 | 39 => 45,
                _ => 42,
            },
            32 => match state {
                26 => 34,
                28 => 37,
                38 => 39,
                _ => 1,
            },
            33 => match state {
                14 => 72,
                23 => 85,
                _ => 52,
            },
            34 => 53,
            35 => 14,
            _ => 0,
        }
    }
    fn __expected_tokens(__state: i8) -> alloc::vec::Vec<alloc::string::String> {
        const __TERMINAL: &[&str] = &[
            r###""!""###,
            r###""&&""###,
            r###""(""###,
            r###"")""###,
            r###""*""###,
            r###""+""###,
            r###"",""###,
            r###""-""###,
            r###""/""###,
            r###"":""###,
            r###"";""###,
            r###""=""###,
            r###""==""###,
            r###""false""###,
            r###""fn""###,
            r###""if""###,
            r###""input""###,
            r###""let""###,
            r###""mut""###,
            r###""print""###,
            r###""return""###,
            r###""true""###,
            r###""while""###,
            r###""{""###,
            r###""||""###,
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
        type Success = ParsedProgram;
        type StateIndex = i8;
        type Action = i8;
        type ReduceIndex = i8;
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
        fn action(&self, state: i8, integer: usize) -> i8 {
            __action(state, integer)
        }

        #[inline]
        fn error_action(&self, state: i8) -> i8 {
            __action(state, 28 - 1)
        }

        #[inline]
        fn eof_action(&self, state: i8) -> i8 {
            __EOF_ACTION[state as usize]
        }

        #[inline]
        fn goto(&self, state: i8, nt: usize) -> i8 {
            __goto(state, nt)
        }

        fn token_to_symbol(&self, token_index: usize, token: Self::Token) -> Self::Symbol {
            __token_to_symbol(token_index, token, core::marker::PhantomData::<(&())>)
        }

        fn expected_tokens(&self, state: i8) -> alloc::vec::Vec<alloc::string::String> {
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
            action: i8,
            start_location: Option<&Self::Location>,
            states: &mut alloc::vec::Vec<i8>,
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

        fn simulate_reduce(&self, action: i8) -> __state_machine::SimulatedReduce<Self> {
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
            Token(4, _) if true => Some(0),
            Token(5, _) if true => Some(1),
            Token(6, _) if true => Some(2),
            Token(7, _) if true => Some(3),
            Token(8, _) if true => Some(4),
            Token(9, _) if true => Some(5),
            Token(10, _) if true => Some(6),
            Token(11, _) if true => Some(7),
            Token(12, _) if true => Some(8),
            Token(13, _) if true => Some(9),
            Token(14, _) if true => Some(10),
            Token(15, _) if true => Some(11),
            Token(16, _) if true => Some(12),
            Token(17, _) if true => Some(13),
            Token(18, _) if true => Some(14),
            Token(19, _) if true => Some(15),
            Token(20, _) if true => Some(16),
            Token(21, _) if true => Some(17),
            Token(22, _) if true => Some(18),
            Token(23, _) if true => Some(19),
            Token(24, _) if true => Some(20),
            Token(25, _) if true => Some(21),
            Token(26, _) if true => Some(22),
            Token(27, _) if true => Some(23),
            Token(28, _) if true => Some(24),
            Token(29, _) if true => Some(25),
            Token(1, _) if true => Some(26),
            Token(2, _) if true => Some(27),
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
            0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 => match __token {
                Token(4, __tok0) | Token(5, __tok0) | Token(6, __tok0) | Token(7, __tok0) | Token(8, __tok0) | Token(9, __tok0) | Token(10, __tok0) | Token(11, __tok0) | Token(12, __tok0) | Token(13, __tok0) | Token(14, __tok0) | Token(15, __tok0) | Token(16, __tok0) | Token(17, __tok0) | Token(18, __tok0) | Token(19, __tok0) | Token(20, __tok0) | Token(21, __tok0) | Token(22, __tok0) | Token(23, __tok0) | Token(24, __tok0) | Token(25, __tok0) | Token(26, __tok0) | Token(27, __tok0) | Token(28, __tok0) | Token(29, __tok0) | Token(1, __tok0) | Token(2, __tok0) if true => __Symbol::Variant0(__tok0),
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
        ) -> Result<ParsedProgram, __lalrpop_util::ParseError<usize, Token<'input>, &'static str>>
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
        __action: i8,
        __lookahead_start: Option<&usize>,
        __states: &mut alloc::vec::Vec<i8>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> Option<Result<ParsedProgram,__lalrpop_util::ParseError<usize, Token<'input>, &'static str>>>
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
                // __Program = Program => ActionFn(0);
                let __sym0 = __pop_Variant13(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action0::<>(astf, input, __sym0);
                return Some(Ok(__nt));
            }
            79 => {
                __reduce79(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
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
    fn __pop_Variant7<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Binop, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant7(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant2<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, ParsedExpr, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant2(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant13<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, ParsedProgram, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant13(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant14<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, ParsedStmt, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant14(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant4<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, String, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant4(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant16<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Unop, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant16(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant9<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Vec<ParsedExpr>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant9(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant10<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Vec<String>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant10(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant3<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, alloc::vec::Vec<ParsedExpr>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant3(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant15<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, alloc::vec::Vec<ParsedStmt>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant15(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant5<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, alloc::vec::Vec<String>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant5(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant8<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, bool, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant8(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant11<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, core::option::Option<ParsedExpr>, usize)
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
    ) -> (usize, core::option::Option<String>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant12(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant1<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, core::option::Option<&'input str>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant1(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant6<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, usize, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant6(__v), __r)) => (__l, __v, __r),
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
        // "mut"? = "mut" => ActionFn(45);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action45::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (1, 0)
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
        // "mut"? =  => ActionFn(46);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action46::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (0, 0)
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
        // (<Expr> ",") = Expr, "," => ActionFn(59);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action59::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (2, 1)
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
        // (<Expr> ",")* =  => ActionFn(57);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action57::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (0, 2)
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
        // (<Expr> ",")* = (<Expr> ",")+ => ActionFn(58);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action58::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (1, 2)
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
        // (<Expr> ",")+ = Expr, "," => ActionFn(71);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action71::<>(astf, input, __sym0, __sym1);
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
        // (<Expr> ",")+ = (<Expr> ",")+, Expr, "," => ActionFn(72);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action72::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (3, 3)
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
        // (<Id> ",") = Id, "," => ActionFn(64);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action64::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (2, 4)
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
        // (<Id> ",")* =  => ActionFn(62);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action62::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (0, 5)
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
        // (<Id> ",")* = (<Id> ",")+ => ActionFn(63);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action63::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (1, 5)
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
        // (<Id> ",")+ = Id, "," => ActionFn(75);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action75::<>(astf, input, __sym0, __sym1);
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
        // (<Id> ",")+ = (<Id> ",")+, Id, "," => ActionFn(76);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant4(__symbols);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action76::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (3, 6)
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
        // @L =  => ActionFn(50);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action50::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (0, 7)
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
        // @R =  => ActionFn(49);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action49::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (0, 8)
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
        // Binop1 = "==" => ActionFn(37);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action37::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 9)
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
        // Binop2 = "+" => ActionFn(38);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action38::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 10)
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
        // Binop2 = "-" => ActionFn(39);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action39::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 10)
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
        // Binop3 = "*" => ActionFn(40);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action40::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 11)
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
        // Binop3 = "/" => ActionFn(41);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action41::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 11)
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
        // BinopAnd = "&&" => ActionFn(35);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action35::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 12)
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
        // BinopE1 = BinopE1, Binop1, BinopE2 => ActionFn(99);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action99::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (3, 13)
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
        // BinopE1 = BinopE2 => ActionFn(30);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action30::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 13)
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
        // BinopE2 = BinopE2, Binop2, BinopE3 => ActionFn(100);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action100::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
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
        // BinopE2 = BinopE3 => ActionFn(32);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action32::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 14)
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
        // BinopE3 = BinopE3, Binop3, TermExpr => ActionFn(101);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action101::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (3, 15)
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
        // BinopE3 = TermExpr => ActionFn(34);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action34::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 15)
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
        // BinopEAnd = BinopEAnd, BinopAnd, BinopE1 => ActionFn(102);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action102::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (3, 16)
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
        // BinopEAnd = BinopE1 => ActionFn(28);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action28::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 16)
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
        // BinopEOr = BinopEOr, BinopOr, BinopEAnd => ActionFn(103);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action103::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (3, 17)
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
        // BinopEOr = BinopEAnd => ActionFn(26);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action26::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 17)
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
        // BinopExpr = BinopEOr => ActionFn(24);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action24::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 18)
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
        // BinopOr = "||" => ActionFn(36);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action36::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 19)
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
        // Bool = "true" => ActionFn(20);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action20::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
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
        // Bool = "false" => ActionFn(21);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action21::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (1, 20)
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
        // CommaList<Expr> = Expr => ActionFn(119);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action119::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (1, 21)
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
        // CommaList<Expr> =  => ActionFn(120);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action120::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (0, 21)
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
        // CommaList<Expr> = (<Expr> ",")+, Expr => ActionFn(121);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action121::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (2, 21)
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
        // CommaList<Expr> = (<Expr> ",")+ => ActionFn(122);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action122::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (1, 21)
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
        // CommaList<Id> = Id => ActionFn(123);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action123::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (1, 22)
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
        // CommaList<Id> =  => ActionFn(124);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action124::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (0, 22)
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
        // CommaList<Id> = (<Id> ",")+, Id => ActionFn(125);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant4(__symbols);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action125::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (2, 22)
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
        // CommaList<Id> = (<Id> ",")+ => ActionFn(126);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action126::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (1, 22)
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
        // Constant = r#"[-]?[0-9]+([.][0-9]+)?"# => ActionFn(44);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action44::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (1, 23)
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
        // Expr = BinopExpr => ActionFn(11);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action11::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 24)
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
        // Expr? = Expr => ActionFn(55);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action55::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant11(__nt), __end));
        (1, 25)
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
        // Expr? =  => ActionFn(56);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action56::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant11(__nt), __end));
        (0, 25)
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
        // Id = r#"[a-zA-Z][a-zA-Z0-9_]*"# => ActionFn(43);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action43::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (1, 26)
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
        // Id? = Id => ActionFn(60);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action60::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant12(__nt), __end));
        (1, 27)
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
        // Id? =  => ActionFn(61);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action61::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant12(__nt), __end));
        (0, 27)
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
        // Mut = "mut" => ActionFn(69);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action69::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (1, 28)
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
        // Mut =  => ActionFn(70);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action70::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (0, 28)
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
        // Program =  => ActionFn(127);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action127::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant13(__nt), __end));
        (0, 29)
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
        // Program = Stmt+ => ActionFn(128);
        let __sym0 = __pop_Variant15(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action128::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant13(__nt), __end));
        (1, 29)
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
        // Stmt = "if", Expr, "{", "}" => ActionFn(129);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action129::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (4, 30)
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
        // Stmt = "if", Expr, "{", Stmt+, "}" => ActionFn(130);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant15(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action130::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (5, 30)
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
        // Stmt = "while", Expr, "{", "}" => ActionFn(131);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action131::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (4, 30)
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
        // Stmt = "while", Expr, "{", Stmt+, "}" => ActionFn(132);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant15(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action132::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (5, 30)
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
        // Stmt = "print", "(", Expr, ")", ";" => ActionFn(106);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action106::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (5, 30)
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
        // Stmt = "let", Mut, Id, "=", Expr, ";" => ActionFn(107);
        assert!(__symbols.len() >= 6);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant2(__symbols);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant4(__symbols);
        let __sym1 = __pop_Variant8(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym5.2.clone();
        let __nt = super::__action107::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (6, 30)
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
        // Stmt = Id, "=", Expr, ";" => ActionFn(108);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action108::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (4, 30)
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
        // Stmt = Id, "(", CommaList<Expr>, ")", ";" => ActionFn(109);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant9(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action109::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (5, 30)
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
        // Stmt = "return", Expr, ";" => ActionFn(110);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action110::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (3, 30)
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
        // Stmt = "fn", Id, "(", CommaList<Id>, ")", "{", "}" => ActionFn(133);
        assert!(__symbols.len() >= 7);
        let __sym6 = __pop_Variant0(__symbols);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant10(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant4(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym6.2.clone();
        let __nt = super::__action133::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (7, 30)
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
        // Stmt = "fn", Id, "(", CommaList<Id>, ")", "{", Stmt+, "}" => ActionFn(134);
        assert!(__symbols.len() >= 8);
        let __sym7 = __pop_Variant0(__symbols);
        let __sym6 = __pop_Variant15(__symbols);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant10(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant4(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym7.2.clone();
        let __nt = super::__action134::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (8, 30)
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
        // Stmt* =  => ActionFn(51);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action51::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (0, 31)
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
        // Stmt* = Stmt+ => ActionFn(52);
        let __sym0 = __pop_Variant15(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action52::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (1, 31)
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
        // Stmt+ = Stmt => ActionFn(53);
        let __sym0 = __pop_Variant14(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action53::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (1, 32)
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
        // Stmt+ = Stmt+, Stmt => ActionFn(54);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant14(__symbols);
        let __sym0 = __pop_Variant15(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action54::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (2, 32)
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
        // TermExpr = Constant => ActionFn(112);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action112::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 33)
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
        // TermExpr = Id => ActionFn(113);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action113::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 33)
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
        // TermExpr = Bool => ActionFn(114);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action114::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 33)
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
        // TermExpr = "input", "(", ")" => ActionFn(115);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action115::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (3, 33)
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
        // TermExpr = Id, "(", CommaList<Expr>, ")" => ActionFn(116);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant9(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action116::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (4, 33)
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
        // TermExpr = "{", Id, ":", Expr, "}" => ActionFn(117);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant2(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant4(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action117::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (5, 33)
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
        // TermExpr = UnopExpr => ActionFn(18);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action18::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 33)
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
        // TermExpr = "(", Expr, ")" => ActionFn(19);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action19::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (3, 33)
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
        // UnopExpr = UnopNot, TermExpr => ActionFn(118);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant16(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action118::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (2, 34)
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
        // UnopNot = "!" => ActionFn(23);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action23::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant16(__nt), __end));
        (1, 35)
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
        // __Stmt = Stmt => ActionFn(1);
        let __sym0 = __pop_Variant14(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action1::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (1, 37)
    }
}
pub use self::__parse__Program::ProgramParser;

#[cfg_attr(rustfmt, rustfmt_skip)]
mod __parse__Stmt {
    #![allow(non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens, clippy::all)]

    use crate::value_language::ast::*;
    use crate::value_language::ast_factory::ASTFactory;
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
        Variant1(core::option::Option<&'input str>),
        Variant2(ParsedExpr),
        Variant3(alloc::vec::Vec<ParsedExpr>),
        Variant4(String),
        Variant5(alloc::vec::Vec<String>),
        Variant6(usize),
        Variant7(Binop),
        Variant8(bool),
        Variant9(Vec<ParsedExpr>),
        Variant10(Vec<String>),
        Variant11(core::option::Option<ParsedExpr>),
        Variant12(core::option::Option<String>),
        Variant13(ParsedProgram),
        Variant14(ParsedStmt),
        Variant15(alloc::vec::Vec<ParsedStmt>),
        Variant16(Unop),
    }
    const __ACTION: &[i8] = &[
        // State 0
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 0, 4, 0, 42, 5, 0, 6, 0, 0, 0, 0, 43,
        // State 1
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 43,
        // State 2
        52, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 0, 0, 54, 0, 0, 0, 0, 55, 0, 16, 0, 0, 56, 43,
        // State 3
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 57, 0, 0, 0, 0, 0, 0, 0, 0, -51,
        // State 4
        52, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 0, 0, 54, 0, 0, 0, 0, 55, 0, 16, 0, 0, 56, 43,
        // State 5
        52, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 0, 0, 54, 0, 0, 0, 0, 55, 0, 16, 0, 0, 56, 43,
        // State 6
        52, 0, 15, -36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 0, 0, 54, 0, 0, 0, 0, 55, 0, 16, 0, 0, 56, 43,
        // State 7
        52, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 0, 0, 54, 0, 0, 0, 0, 55, 0, 16, 0, 0, 56, 43,
        // State 8
        0, -28, 0, -28, 0, 0, -28, 0, 0, 0, -28, 0, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -28, -28, -28, 0, 0,
        // State 9
        0, -22, 0, -22, 0, 64, -22, 65, 0, 0, -22, 0, -22, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -22, -22, -22, 0, 0,
        // State 10
        0, -24, 0, -24, 66, -24, -24, -24, 67, 0, -24, 0, -24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -24, -24, -24, 0, 0,
        // State 11
        0, 68, 0, -30, 0, 0, -30, 0, 0, 0, -30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -30, -30, -30, 0, 0,
        // State 12
        0, 0, 0, -31, 0, 0, -31, 0, 0, 0, -31, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -31, 69, -31, 0, 0,
        // State 13
        52, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 0, 0, 54, 0, 0, 0, 0, 55, 0, 16, 0, 0, 56, 43,
        // State 14
        52, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 0, 0, 54, 0, 0, 0, 0, 55, 0, 16, 0, 0, 56, 43,
        // State 15
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 43,
        // State 16
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 43,
        // State 17
        52, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 0, 0, 54, 0, 0, 0, 0, 55, 0, 16, 0, 0, 56, 43,
        // State 18
        52, 0, 15, -38, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 0, 0, 54, 0, 0, 0, 0, 55, 0, 16, 0, 0, 56, 43,
        // State 19
        0, 0, 0, -40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 43,
        // State 20
        52, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 0, 0, 54, 0, 0, 0, 0, 55, 0, 16, 0, 0, 56, 43,
        // State 21
        52, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 0, 0, 54, 0, 0, 0, 0, 55, 0, 16, 0, 0, 56, 43,
        // State 22
        52, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 0, 0, 54, 0, 0, 0, 0, 55, 0, 16, 0, 0, 56, 43,
        // State 23
        52, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 0, 0, 54, 0, 0, 0, 0, 55, 0, 16, 0, 0, 56, 43,
        // State 24
        52, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 0, 0, 54, 0, 0, 0, 0, 55, 0, 16, 0, 0, 56, 43,
        // State 25
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 0, 4, 0, 42, 5, 0, 6, 0, 0, 85, 0, 43,
        // State 26
        52, 0, 15, -36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 0, 0, 54, 0, 0, 0, 0, 55, 0, 16, 0, 0, 56, 43,
        // State 27
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 0, 4, 0, 42, 5, 0, 6, 0, 0, 90, 0, 43,
        // State 28
        0, 0, 0, -42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 43,
        // State 29
        0, -21, 0, -21, 0, 64, -21, 65, 0, 0, -21, 0, -21, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -21, -21, -21, 0, 0,
        // State 30
        0, -23, 0, -23, 66, -23, -23, -23, 67, 0, -23, 0, -23, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -23, -23, -23, 0, 0,
        // State 31
        0, -27, 0, -27, 0, 0, -27, 0, 0, 0, -27, 0, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -27, -27, -27, 0, 0,
        // State 32
        0, 68, 0, -29, 0, 0, -29, 0, 0, 0, -29, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -29, -29, -29, 0, 0,
        // State 33
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 0, 4, 0, 42, 5, 0, 6, 0, 0, 97, 0, 43,
        // State 34
        52, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 0, 0, 54, 0, 0, 0, 0, 55, 0, 16, 0, 0, 56, 43,
        // State 35
        52, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 0, 0, 54, 0, 0, 0, 0, 55, 0, 16, 0, 0, 56, 43,
        // State 36
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 0, 4, 0, 42, 5, 0, 6, 0, 0, 102, 0, 43,
        // State 37
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 0, 4, 0, 42, 5, 0, 6, 0, 0, 106, 0, 43,
        // State 38
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 0, 4, 0, 42, 5, 0, 6, 0, 0, 107, 0, 43,
        // State 39
        0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 40
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 41
        0, 0, 18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 42
        0, -47, -47, -47, -47, -47, -47, -47, -47, -47, -47, -47, -47, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -47, -47, -47, 0, 0,
        // State 43
        0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 44
        0, 0, 0, -44, 0, 0, -44, 0, 0, 0, -44, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -44, 0, -44, 0, 0,
        // State 45
        0, -71, 0, -71, -71, -71, -71, -71, -71, 0, -71, 0, -71, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -71, -71, -71, 0, 0,
        // State 46
        0, -69, 0, -69, -69, -69, -69, -69, -69, 0, -69, 0, -69, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -69, -69, -69, 0, 0,
        // State 47
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 26, 0, 0, 0, 0,
        // State 48
        0, -70, 27, -70, -70, -70, -70, -70, -70, 0, -70, 0, -70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -70, -70, -70, 0, 0,
        // State 49
        0, -26, 0, -26, -26, -26, -26, -26, -26, 0, -26, 0, -26, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -26, -26, -26, 0, 0,
        // State 50
        0, -75, 0, -75, -75, -75, -75, -75, -75, 0, -75, 0, -75, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -75, -75, -75, 0, 0,
        // State 51
        -78, 0, -78, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -78, 0, 0, -78, 0, 0, 0, 0, -78, 0, -78, 0, 0, -78, -78,
        // State 52
        0, -34, 0, -34, -34, -34, -34, -34, -34, 0, -34, 0, -34, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -34, -34, -34, 0, 0,
        // State 53
        0, 0, 72, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 54
        0, -33, 0, -33, -33, -33, -33, -33, -33, 0, -33, 0, -33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -33, -33, -33, 0, 0,
        // State 55
        0, -43, 0, -43, -43, -43, -43, -43, -43, 0, -43, 0, -43, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -43, -43, -43, 0, 0,
        // State 56
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -50,
        // State 57
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 76, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 58
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 28, 0, 0, 0, 0,
        // State 59
        0, 0, 0, 78, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 60
        0, 0, 0, -35, 0, 0, 79, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 61
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 62
        -15, 0, -15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -15, 0, 0, -15, 0, 0, 0, 0, -15, 0, -15, 0, 0, -15, -15,
        // State 63
        -16, 0, -16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -16, 0, 0, -16, 0, 0, 0, 0, -16, 0, -16, 0, 0, -16, -16,
        // State 64
        -17, 0, -17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -17, 0, 0, -17, 0, 0, 0, 0, -17, 0, -17, 0, 0, -17, -17,
        // State 65
        -18, 0, -18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -18, 0, 0, -18, 0, 0, 0, 0, -18, 0, -18, 0, 0, -18, -18,
        // State 66
        -19, 0, -19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -19, 0, 0, -19, 0, 0, 0, 0, -19, 0, -19, 0, 0, -19, -19,
        // State 67
        -20, 0, -20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -20, 0, 0, -20, 0, 0, 0, 0, -20, 0, -20, 0, 0, -20, -20,
        // State 68
        -32, 0, -32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -32, 0, 0, -32, 0, 0, 0, 0, -32, 0, -32, 0, 0, -32, -32,
        // State 69
        0, -77, 0, -77, -77, -77, -77, -77, -77, 0, -77, 0, -77, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -77, -77, -77, 0, 0,
        // State 70
        0, 0, 0, 87, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 71
        0, 0, 0, 88, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 72
        0, 0, 0, 0, 0, 0, 0, 0, 0, 35, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 73
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 74
        0, 0, 0, 89, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 75
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -62, -62, 0, -62, 0, -62, -62, 0, -62, 0, 0, -62, 0, -62,
        // State 76
        0, 0, 0, -37, 0, 0, 91, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 77
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 92, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 78
        -6, 0, -6, -6, 0, 0, 0, 0, 0, 0, 0, 0, 0, -6, 0, 0, -6, 0, 0, 0, 0, -6, 0, -6, 0, 0, -6, -6,
        // State 79
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -60, -60, 0, -60, 0, -60, -60, 0, -60, 0, 0, -60, 0, -60,
        // State 80
        0, 0, 0, 94, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 81
        0, 0, 0, -39, 0, 0, 95, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 82
        0, -25, 0, -25, -25, -25, -25, -25, -25, 0, -25, 0, -25, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -25, -25, -25, 0, 0,
        // State 83
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -67, -67, 0, -67, 0, -67, -67, 0, -67, 0, 0, -67, 0, -67,
        // State 84
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -54, -54, 0, -54, 0, -54, -54, 0, -54, 0, 0, -54, 0, -54,
        // State 85
        0, 0, 0, 98, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 86
        0, -76, 0, -76, -76, -76, -76, -76, -76, 0, -76, 0, -76, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -76, -76, -76, 0, 0,
        // State 87
        0, -72, 0, -72, -72, -72, -72, -72, -72, 0, -72, 0, -72, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -72, -72, -72, 0, 0,
        // State 88
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 101, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 89
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -56, -56, 0, -56, 0, -56, -56, 0, -56, 0, 0, -56, 0, -56,
        // State 90
        -7, 0, -7, -7, 0, 0, 0, 0, 0, 0, 0, 0, 0, -7, 0, 0, -7, 0, 0, 0, 0, -7, 0, -7, 0, 0, -7, -7,
        // State 91
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -61, -61, 0, -61, 0, -61, -61, 0, -61, 0, 0, -61, 0, -61,
        // State 92
        0, 0, 0, -41, 0, 0, 103, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 93
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 38, 0, 0, 0, 0,
        // State 94
        0, 0, 0, -11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -11,
        // State 95
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -68, -68, 0, -68, 0, -68, -68, 0, -68, 0, 0, -68, 0, -68,
        // State 96
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -55, -55, 0, -55, 0, -55, -55, 0, -55, 0, 0, -55, 0, -55,
        // State 97
        0, -73, 0, -73, -73, -73, -73, -73, -73, 0, -73, 0, -73, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -73, -73, -73, 0, 0,
        // State 98
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 104, 0, 0,
        // State 99
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 105, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 100
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -58, -58, 0, -58, 0, -58, -58, 0, -58, 0, 0, -58, 0, -58,
        // State 101
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -57, -57, 0, -57, 0, -57, -57, 0, -57, 0, 0, -57, 0, -57,
        // State 102
        0, 0, 0, -12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -12,
        // State 103
        0, -74, 0, -74, -74, -74, -74, -74, -74, 0, -74, 0, -74, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -74, -74, -74, 0, 0,
        // State 104
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -59, -59, 0, -59, 0, -59, -59, 0, -59, 0, 0, -59, 0, -59,
        // State 105
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -63, -63, 0, -63, 0, -63, -63, 0, -63, 0, 0, -63, 0, -63,
        // State 106
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -64, -64, 0, -64, 0, -64, -64, 0, -64, 0, 0, -64, 0, -64,
    ];
    fn __action(state: i8, integer: usize) -> i8 {
        __ACTION[(state as usize) * 28 + integer]
    }
    const __EOF_ACTION: &[i8] = &[
        // State 0
        0,
        // State 1
        0,
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
        -80,
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
        0,
        // State 49
        0,
        // State 50
        0,
        // State 51
        0,
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
        0,
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
        0,
        // State 70
        0,
        // State 71
        0,
        // State 72
        0,
        // State 73
        0,
        // State 74
        0,
        // State 75
        -62,
        // State 76
        0,
        // State 77
        0,
        // State 78
        0,
        // State 79
        -60,
        // State 80
        0,
        // State 81
        0,
        // State 82
        0,
        // State 83
        0,
        // State 84
        -54,
        // State 85
        0,
        // State 86
        0,
        // State 87
        0,
        // State 88
        0,
        // State 89
        -56,
        // State 90
        0,
        // State 91
        -61,
        // State 92
        0,
        // State 93
        0,
        // State 94
        0,
        // State 95
        0,
        // State 96
        -55,
        // State 97
        0,
        // State 98
        0,
        // State 99
        0,
        // State 100
        -58,
        // State 101
        -57,
        // State 102
        0,
        // State 103
        0,
        // State 104
        -59,
        // State 105
        -63,
        // State 106
        -64,
    ];
    fn __goto(state: i8, nt: usize) -> i8 {
        match nt {
            3 => 18,
            6 => 28,
            9 => 20,
            10 => 21,
            11 => 22,
            12 => 23,
            13 => match state {
                23 => 31,
                _ => 8,
            },
            14 => match state {
                20 => 29,
                _ => 9,
            },
            15 => match state {
                21 => 30,
                _ => 10,
            },
            16 => match state {
                24 => 32,
                _ => 11,
            },
            17 => 12,
            18 => 44,
            19 => 24,
            20 => 45,
            21 => match state {
                26 => 85,
                _ => 59,
            },
            22 => 80,
            23 => 46,
            24 => match state {
                2 => 47,
                4 => 57,
                5 => 58,
                7 => 61,
                14 => 70,
                17 => 74,
                18 => 76,
                34 => 98,
                35 => 99,
                _ => 60,
            },
            26 => match state {
                0 | 25 | 27 | 33 | 36..=38 => 39,
                1 => 43,
                15 => 72,
                16 => 73,
                19 => 81,
                28 => 92,
                _ => 48,
            },
            28 => 16,
            30 => match state {
                0 => 40,
                33 | 36 | 38 => 95,
                _ => 83,
            },
            32 => match state {
                27 => 36,
                37 => 38,
                _ => 33,
            },
            33 => match state {
                13 => 69,
                22 => 82,
                _ => 49,
            },
            34 => 50,
            35 => 13,
            _ => 0,
        }
    }
    fn __expected_tokens(__state: i8) -> alloc::vec::Vec<alloc::string::String> {
        const __TERMINAL: &[&str] = &[
            r###""!""###,
            r###""&&""###,
            r###""(""###,
            r###"")""###,
            r###""*""###,
            r###""+""###,
            r###"",""###,
            r###""-""###,
            r###""/""###,
            r###"":""###,
            r###"";""###,
            r###""=""###,
            r###""==""###,
            r###""false""###,
            r###""fn""###,
            r###""if""###,
            r###""input""###,
            r###""let""###,
            r###""mut""###,
            r###""print""###,
            r###""return""###,
            r###""true""###,
            r###""while""###,
            r###""{""###,
            r###""||""###,
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
        type Success = ParsedStmt;
        type StateIndex = i8;
        type Action = i8;
        type ReduceIndex = i8;
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
        fn action(&self, state: i8, integer: usize) -> i8 {
            __action(state, integer)
        }

        #[inline]
        fn error_action(&self, state: i8) -> i8 {
            __action(state, 28 - 1)
        }

        #[inline]
        fn eof_action(&self, state: i8) -> i8 {
            __EOF_ACTION[state as usize]
        }

        #[inline]
        fn goto(&self, state: i8, nt: usize) -> i8 {
            __goto(state, nt)
        }

        fn token_to_symbol(&self, token_index: usize, token: Self::Token) -> Self::Symbol {
            __token_to_symbol(token_index, token, core::marker::PhantomData::<(&())>)
        }

        fn expected_tokens(&self, state: i8) -> alloc::vec::Vec<alloc::string::String> {
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
            action: i8,
            start_location: Option<&Self::Location>,
            states: &mut alloc::vec::Vec<i8>,
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

        fn simulate_reduce(&self, action: i8) -> __state_machine::SimulatedReduce<Self> {
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
            Token(4, _) if true => Some(0),
            Token(5, _) if true => Some(1),
            Token(6, _) if true => Some(2),
            Token(7, _) if true => Some(3),
            Token(8, _) if true => Some(4),
            Token(9, _) if true => Some(5),
            Token(10, _) if true => Some(6),
            Token(11, _) if true => Some(7),
            Token(12, _) if true => Some(8),
            Token(13, _) if true => Some(9),
            Token(14, _) if true => Some(10),
            Token(15, _) if true => Some(11),
            Token(16, _) if true => Some(12),
            Token(17, _) if true => Some(13),
            Token(18, _) if true => Some(14),
            Token(19, _) if true => Some(15),
            Token(20, _) if true => Some(16),
            Token(21, _) if true => Some(17),
            Token(22, _) if true => Some(18),
            Token(23, _) if true => Some(19),
            Token(24, _) if true => Some(20),
            Token(25, _) if true => Some(21),
            Token(26, _) if true => Some(22),
            Token(27, _) if true => Some(23),
            Token(28, _) if true => Some(24),
            Token(29, _) if true => Some(25),
            Token(1, _) if true => Some(26),
            Token(2, _) if true => Some(27),
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
            0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 => match __token {
                Token(4, __tok0) | Token(5, __tok0) | Token(6, __tok0) | Token(7, __tok0) | Token(8, __tok0) | Token(9, __tok0) | Token(10, __tok0) | Token(11, __tok0) | Token(12, __tok0) | Token(13, __tok0) | Token(14, __tok0) | Token(15, __tok0) | Token(16, __tok0) | Token(17, __tok0) | Token(18, __tok0) | Token(19, __tok0) | Token(20, __tok0) | Token(21, __tok0) | Token(22, __tok0) | Token(23, __tok0) | Token(24, __tok0) | Token(25, __tok0) | Token(26, __tok0) | Token(27, __tok0) | Token(28, __tok0) | Token(29, __tok0) | Token(1, __tok0) | Token(2, __tok0) if true => __Symbol::Variant0(__tok0),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
    pub struct StmtParser {
        builder: __lalrpop_util::lexer::MatcherBuilder,
        _priv: (),
    }

    impl StmtParser {
        pub fn new() -> StmtParser {
            let __builder = super::__intern_token::new_builder();
            StmtParser {
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
        ) -> Result<ParsedStmt, __lalrpop_util::ParseError<usize, Token<'input>, &'static str>>
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
        __action: i8,
        __lookahead_start: Option<&usize>,
        __states: &mut alloc::vec::Vec<i8>,
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>,
        _: core::marker::PhantomData<(&'input ())>,
    ) -> Option<Result<ParsedStmt,__lalrpop_util::ParseError<usize, Token<'input>, &'static str>>>
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
                // __Stmt = Stmt => ActionFn(1);
                let __sym0 = __pop_Variant14(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action1::<>(astf, input, __sym0);
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
    fn __pop_Variant7<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Binop, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant7(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant2<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, ParsedExpr, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant2(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant13<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, ParsedProgram, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant13(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant14<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, ParsedStmt, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant14(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant4<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, String, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant4(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant16<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Unop, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant16(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant9<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Vec<ParsedExpr>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant9(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant10<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Vec<String>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant10(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant3<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, alloc::vec::Vec<ParsedExpr>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant3(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant15<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, alloc::vec::Vec<ParsedStmt>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant15(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant5<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, alloc::vec::Vec<String>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant5(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant8<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, bool, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant8(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant11<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, core::option::Option<ParsedExpr>, usize)
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
    ) -> (usize, core::option::Option<String>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant12(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant1<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, core::option::Option<&'input str>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant1(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant6<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, usize, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant6(__v), __r)) => (__l, __v, __r),
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
        // "mut"? = "mut" => ActionFn(45);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action45::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (1, 0)
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
        // "mut"? =  => ActionFn(46);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action46::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (0, 0)
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
        // (<Expr> ",") = Expr, "," => ActionFn(59);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action59::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (2, 1)
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
        // (<Expr> ",")* =  => ActionFn(57);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action57::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (0, 2)
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
        // (<Expr> ",")* = (<Expr> ",")+ => ActionFn(58);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action58::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (1, 2)
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
        // (<Expr> ",")+ = Expr, "," => ActionFn(71);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action71::<>(astf, input, __sym0, __sym1);
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
        // (<Expr> ",")+ = (<Expr> ",")+, Expr, "," => ActionFn(72);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action72::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (3, 3)
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
        // (<Id> ",") = Id, "," => ActionFn(64);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action64::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (2, 4)
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
        // (<Id> ",")* =  => ActionFn(62);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action62::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (0, 5)
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
        // (<Id> ",")* = (<Id> ",")+ => ActionFn(63);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action63::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (1, 5)
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
        // (<Id> ",")+ = Id, "," => ActionFn(75);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action75::<>(astf, input, __sym0, __sym1);
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
        // (<Id> ",")+ = (<Id> ",")+, Id, "," => ActionFn(76);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant4(__symbols);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action76::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (3, 6)
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
        // @L =  => ActionFn(50);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action50::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (0, 7)
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
        // @R =  => ActionFn(49);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action49::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (0, 8)
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
        // Binop1 = "==" => ActionFn(37);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action37::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 9)
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
        // Binop2 = "+" => ActionFn(38);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action38::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 10)
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
        // Binop2 = "-" => ActionFn(39);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action39::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 10)
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
        // Binop3 = "*" => ActionFn(40);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action40::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 11)
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
        // Binop3 = "/" => ActionFn(41);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action41::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 11)
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
        // BinopAnd = "&&" => ActionFn(35);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action35::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 12)
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
        // BinopE1 = BinopE1, Binop1, BinopE2 => ActionFn(99);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action99::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (3, 13)
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
        // BinopE1 = BinopE2 => ActionFn(30);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action30::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 13)
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
        // BinopE2 = BinopE2, Binop2, BinopE3 => ActionFn(100);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action100::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
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
        // BinopE2 = BinopE3 => ActionFn(32);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action32::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 14)
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
        // BinopE3 = BinopE3, Binop3, TermExpr => ActionFn(101);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action101::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (3, 15)
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
        // BinopE3 = TermExpr => ActionFn(34);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action34::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 15)
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
        // BinopEAnd = BinopEAnd, BinopAnd, BinopE1 => ActionFn(102);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action102::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (3, 16)
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
        // BinopEAnd = BinopE1 => ActionFn(28);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action28::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 16)
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
        // BinopEOr = BinopEOr, BinopOr, BinopEAnd => ActionFn(103);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action103::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (3, 17)
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
        // BinopEOr = BinopEAnd => ActionFn(26);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action26::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 17)
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
        // BinopExpr = BinopEOr => ActionFn(24);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action24::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 18)
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
        // BinopOr = "||" => ActionFn(36);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action36::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 19)
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
        // Bool = "true" => ActionFn(20);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action20::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
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
        // Bool = "false" => ActionFn(21);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action21::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (1, 20)
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
        // CommaList<Expr> = Expr => ActionFn(119);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action119::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (1, 21)
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
        // CommaList<Expr> =  => ActionFn(120);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action120::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (0, 21)
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
        // CommaList<Expr> = (<Expr> ",")+, Expr => ActionFn(121);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action121::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (2, 21)
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
        // CommaList<Expr> = (<Expr> ",")+ => ActionFn(122);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action122::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant9(__nt), __end));
        (1, 21)
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
        // CommaList<Id> = Id => ActionFn(123);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action123::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (1, 22)
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
        // CommaList<Id> =  => ActionFn(124);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action124::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (0, 22)
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
        // CommaList<Id> = (<Id> ",")+, Id => ActionFn(125);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant4(__symbols);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action125::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (2, 22)
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
        // CommaList<Id> = (<Id> ",")+ => ActionFn(126);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action126::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant10(__nt), __end));
        (1, 22)
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
        // Constant = r#"[-]?[0-9]+([.][0-9]+)?"# => ActionFn(44);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action44::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (1, 23)
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
        // Expr = BinopExpr => ActionFn(11);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action11::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 24)
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
        // Expr? = Expr => ActionFn(55);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action55::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant11(__nt), __end));
        (1, 25)
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
        // Expr? =  => ActionFn(56);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action56::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant11(__nt), __end));
        (0, 25)
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
        // Id = r#"[a-zA-Z][a-zA-Z0-9_]*"# => ActionFn(43);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action43::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (1, 26)
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
        // Id? = Id => ActionFn(60);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action60::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant12(__nt), __end));
        (1, 27)
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
        // Id? =  => ActionFn(61);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action61::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant12(__nt), __end));
        (0, 27)
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
        // Mut = "mut" => ActionFn(69);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action69::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (1, 28)
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
        // Mut =  => ActionFn(70);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action70::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (0, 28)
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
        // Program =  => ActionFn(127);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action127::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant13(__nt), __end));
        (0, 29)
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
        // Program = Stmt+ => ActionFn(128);
        let __sym0 = __pop_Variant15(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action128::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant13(__nt), __end));
        (1, 29)
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
        // Stmt = "if", Expr, "{", "}" => ActionFn(129);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action129::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (4, 30)
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
        // Stmt = "if", Expr, "{", Stmt+, "}" => ActionFn(130);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant15(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action130::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (5, 30)
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
        // Stmt = "while", Expr, "{", "}" => ActionFn(131);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action131::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (4, 30)
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
        // Stmt = "while", Expr, "{", Stmt+, "}" => ActionFn(132);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant15(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action132::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (5, 30)
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
        // Stmt = "print", "(", Expr, ")", ";" => ActionFn(106);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action106::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (5, 30)
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
        // Stmt = "let", Mut, Id, "=", Expr, ";" => ActionFn(107);
        assert!(__symbols.len() >= 6);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant2(__symbols);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant4(__symbols);
        let __sym1 = __pop_Variant8(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym5.2.clone();
        let __nt = super::__action107::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (6, 30)
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
        // Stmt = Id, "=", Expr, ";" => ActionFn(108);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action108::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (4, 30)
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
        // Stmt = Id, "(", CommaList<Expr>, ")", ";" => ActionFn(109);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant9(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action109::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (5, 30)
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
        // Stmt = "return", Expr, ";" => ActionFn(110);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action110::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (3, 30)
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
        // Stmt = "fn", Id, "(", CommaList<Id>, ")", "{", "}" => ActionFn(133);
        assert!(__symbols.len() >= 7);
        let __sym6 = __pop_Variant0(__symbols);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant10(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant4(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym6.2.clone();
        let __nt = super::__action133::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (7, 30)
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
        // Stmt = "fn", Id, "(", CommaList<Id>, ")", "{", Stmt+, "}" => ActionFn(134);
        assert!(__symbols.len() >= 8);
        let __sym7 = __pop_Variant0(__symbols);
        let __sym6 = __pop_Variant15(__symbols);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant10(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant4(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym7.2.clone();
        let __nt = super::__action134::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (8, 30)
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
        // Stmt* =  => ActionFn(51);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action51::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (0, 31)
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
        // Stmt* = Stmt+ => ActionFn(52);
        let __sym0 = __pop_Variant15(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action52::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (1, 31)
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
        // Stmt+ = Stmt => ActionFn(53);
        let __sym0 = __pop_Variant14(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action53::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (1, 32)
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
        // Stmt+ = Stmt+, Stmt => ActionFn(54);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant14(__symbols);
        let __sym0 = __pop_Variant15(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action54::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (2, 32)
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
        // TermExpr = Constant => ActionFn(112);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action112::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 33)
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
        // TermExpr = Id => ActionFn(113);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action113::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 33)
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
        // TermExpr = Bool => ActionFn(114);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action114::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 33)
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
        // TermExpr = "input", "(", ")" => ActionFn(115);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action115::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (3, 33)
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
        // TermExpr = Id, "(", CommaList<Expr>, ")" => ActionFn(116);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant9(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action116::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (4, 33)
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
        // TermExpr = "{", Id, ":", Expr, "}" => ActionFn(117);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant2(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant4(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action117::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (5, 33)
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
        // TermExpr = UnopExpr => ActionFn(18);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action18::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 33)
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
        // TermExpr = "(", Expr, ")" => ActionFn(19);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action19::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (3, 33)
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
        // UnopExpr = UnopNot, TermExpr => ActionFn(118);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant16(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action118::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (2, 34)
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
        // UnopNot = "!" => ActionFn(23);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action23::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant16(__nt), __end));
        (1, 35)
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
        // __Program = Program => ActionFn(0);
        let __sym0 = __pop_Variant13(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action0::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant13(__nt), __end));
        (1, 36)
    }
}
pub use self::__parse__Stmt::StmtParser;
#[cfg_attr(rustfmt, rustfmt_skip)]
mod __intern_token {
    #![allow(unused_imports)]
    use crate::value_language::ast::*;
    use crate::value_language::ast_factory::ASTFactory;
    #[allow(unused_extern_crates)]
    extern crate lalrpop_util as __lalrpop_util;
    #[allow(unused_imports)]
    use self::__lalrpop_util::state_machine as __state_machine;
    extern crate core;
    extern crate alloc;
    pub fn new_builder() -> __lalrpop_util::lexer::MatcherBuilder {
        let __strs: &[(&str, bool)] = &[
            ("^(//[\0-\t\u{b}-\u{c}\u{e}-\u{10ffff}]*[\n\r]*)", true),
            ("^([\\-]?[0-9]+([\\.][0-9]+)?)", false),
            ("^([A-Za-z][0-9A-Z_a-z]*)", false),
            ("^([\t-\r \u{85}\u{a0}\u{1680}\u{2000}-\u{200a}\u{2028}-\u{2029}\u{202f}\u{205f}\u{3000}]*)", true),
            ("^(!)", false),
            ("^(\\&\\&)", false),
            ("^(\\()", false),
            ("^(\\))", false),
            ("^(\\*)", false),
            ("^(\\+)", false),
            ("^(,)", false),
            ("^(\\-)", false),
            ("^(/)", false),
            ("^(:)", false),
            ("^(;)", false),
            ("^(=)", false),
            ("^(==)", false),
            ("^(false)", false),
            ("^(fn)", false),
            ("^(if)", false),
            ("^(input)", false),
            ("^(let)", false),
            ("^(mut)", false),
            ("^(print)", false),
            ("^(return)", false),
            ("^(true)", false),
            ("^(while)", false),
            ("^(\\{)", false),
            ("^(\\|\\|)", false),
            ("^(\\})", false),
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
    (_, __0, _): (usize, ParsedProgram, usize),
) -> ParsedProgram
{
    __0
}

#[allow(unused_variables)]
fn __action1<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, ParsedStmt, usize),
) -> ParsedStmt
{
    __0
}

#[allow(unused_variables)]
fn __action2<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, alloc::vec::Vec<ParsedStmt>, usize),
) -> ParsedProgram
{
    __0
}

#[allow(unused_variables)]
fn __action3<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __1, _): (usize, ParsedExpr, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, alloc::vec::Vec<ParsedStmt>, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __3, _): (usize, usize, usize),
) -> ParsedStmt
{
    astf.if_stmt(__0, __1, __2, __3)
}

#[allow(unused_variables)]
fn __action4<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __1, _): (usize, ParsedExpr, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, alloc::vec::Vec<ParsedStmt>, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __3, _): (usize, usize, usize),
) -> ParsedStmt
{
    astf.while_stmt(__0, __1, __2, __3)
}

#[allow(unused_variables)]
fn __action5<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __1, _): (usize, ParsedExpr, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, usize, usize),
) -> ParsedStmt
{
    astf.print(__0, __1, __2)
}

#[allow(unused_variables)]
fn __action6<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __1, _): (usize, bool, usize),
    (_, __2, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __3, _): (usize, ParsedExpr, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __4, _): (usize, usize, usize),
) -> ParsedStmt
{
    astf.let_stmt(__0, __1, __2, __3, __4)
}

#[allow(unused_variables)]
fn __action7<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, __1, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, ParsedExpr, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __3, _): (usize, usize, usize),
) -> ParsedStmt
{
    astf.assign(__0, __1, __2, __3)
}

#[allow(unused_variables)]
fn __action8<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, __1, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, Vec<ParsedExpr>, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __3, _): (usize, usize, usize),
) -> ParsedStmt
{
    astf.ccall(__0, __1, __2, __3)
}

#[allow(unused_variables)]
fn __action9<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __1, _): (usize, ParsedExpr, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, usize, usize),
) -> ParsedStmt
{
    astf.return_stmt(__0, __1, __2)
}

#[allow(unused_variables)]
fn __action10<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __1, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, Vec<String>, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __3, _): (usize, alloc::vec::Vec<ParsedStmt>, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __4, _): (usize, usize, usize),
) -> ParsedStmt
{
    astf.function(__0, __1, __2, __3, __4)
}

#[allow(unused_variables)]
fn __action11<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, ParsedExpr, usize),
) -> ParsedExpr
{
    __0
}

#[allow(unused_variables)]
fn __action12<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, __1, _): (usize, String, usize),
    (_, __2, _): (usize, usize, usize),
) -> ParsedExpr
{
    astf.num(__0, __1, __2)
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
) -> ParsedExpr
{
    astf.var(__0, __1, __2)
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
) -> ParsedExpr
{
    astf.bool_expr(__0, __1, __2)
}

#[allow(unused_variables)]
fn __action15<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __1, _): (usize, usize, usize),
) -> ParsedExpr
{
    astf.input(__0, __1)
}

#[allow(unused_variables)]
fn __action16<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, __1, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, Vec<ParsedExpr>, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __3, _): (usize, usize, usize),
) -> ParsedExpr
{
    astf.ecall(__0, __1, __2, __3)
}

#[allow(unused_variables)]
fn __action17<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __1, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, ParsedExpr, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __3, _): (usize, usize, usize),
) -> ParsedExpr
{
    astf.labeled(__0, __1, __2, __3)
}

#[allow(unused_variables)]
fn __action18<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, ParsedExpr, usize),
) -> ParsedExpr
{
    __0
}

#[allow(unused_variables)]
fn __action19<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, _, _): (usize, &'input str, usize),
    (_, __0, _): (usize, ParsedExpr, usize),
    (_, _, _): (usize, &'input str, usize),
) -> ParsedExpr
{
    __0
}

#[allow(unused_variables)]
fn __action20<
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
fn __action21<
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
fn __action22<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, __1, _): (usize, Unop, usize),
    (_, __2, _): (usize, ParsedExpr, usize),
    (_, __3, _): (usize, usize, usize),
) -> ParsedExpr
{
    astf.unop(__0, __1, __2, __3)
}

#[allow(unused_variables)]
fn __action23<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> Unop
{
    Unop::Not
}

#[allow(unused_variables)]
fn __action24<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, ParsedExpr, usize),
) -> ParsedExpr
{
    __0
}

#[allow(unused_variables)]
fn __action25<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, __1, _): (usize, ParsedExpr, usize),
    (_, __2, _): (usize, Binop, usize),
    (_, __3, _): (usize, ParsedExpr, usize),
    (_, __4, _): (usize, usize, usize),
) -> ParsedExpr
{
    astf.binop(__0, __1, __2, __3, __4)
}

#[allow(unused_variables)]
fn __action26<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, ParsedExpr, usize),
) -> ParsedExpr
{
    __0
}

#[allow(unused_variables)]
fn __action27<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, __1, _): (usize, ParsedExpr, usize),
    (_, __2, _): (usize, Binop, usize),
    (_, __3, _): (usize, ParsedExpr, usize),
    (_, __4, _): (usize, usize, usize),
) -> ParsedExpr
{
    astf.binop(__0, __1, __2, __3, __4)
}

#[allow(unused_variables)]
fn __action28<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, ParsedExpr, usize),
) -> ParsedExpr
{
    __0
}

#[allow(unused_variables)]
fn __action29<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, __1, _): (usize, ParsedExpr, usize),
    (_, __2, _): (usize, Binop, usize),
    (_, __3, _): (usize, ParsedExpr, usize),
    (_, __4, _): (usize, usize, usize),
) -> ParsedExpr
{
    astf.binop(__0, __1, __2, __3, __4)
}

#[allow(unused_variables)]
fn __action30<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, ParsedExpr, usize),
) -> ParsedExpr
{
    __0
}

#[allow(unused_variables)]
fn __action31<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, __1, _): (usize, ParsedExpr, usize),
    (_, __2, _): (usize, Binop, usize),
    (_, __3, _): (usize, ParsedExpr, usize),
    (_, __4, _): (usize, usize, usize),
) -> ParsedExpr
{
    astf.binop(__0, __1, __2, __3, __4)
}

#[allow(unused_variables)]
fn __action32<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, ParsedExpr, usize),
) -> ParsedExpr
{
    __0
}

#[allow(unused_variables)]
fn __action33<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, usize, usize),
    (_, __1, _): (usize, ParsedExpr, usize),
    (_, __2, _): (usize, Binop, usize),
    (_, __3, _): (usize, ParsedExpr, usize),
    (_, __4, _): (usize, usize, usize),
) -> ParsedExpr
{
    astf.binop(__0, __1, __2, __3, __4)
}

#[allow(unused_variables)]
fn __action34<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, ParsedExpr, usize),
) -> ParsedExpr
{
    __0
}

#[allow(unused_variables)]
fn __action35<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> Binop
{
    Binop::And
}

#[allow(unused_variables)]
fn __action36<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> Binop
{
    Binop::Or
}

#[allow(unused_variables)]
fn __action37<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> Binop
{
    Binop::Equals
}

#[allow(unused_variables)]
fn __action38<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> Binop
{
    Binop::Plus
}

#[allow(unused_variables)]
fn __action39<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> Binop
{
    Binop::Minus
}

#[allow(unused_variables)]
fn __action40<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> Binop
{
    Binop::Mult
}

#[allow(unused_variables)]
fn __action41<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> Binop
{
    Binop::Div
}

#[allow(unused_variables)]
fn __action42<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, m, _): (usize, core::option::Option<&'input str>, usize),
) -> bool
{
    m.is_some()
}

#[allow(unused_variables)]
fn __action43<
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
fn __action44<
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
fn __action45<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> core::option::Option<&'input str>
{
    Some(__0)
}

#[allow(unused_variables)]
fn __action46<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> core::option::Option<&'input str>
{
    None
}

#[allow(unused_variables)]
fn __action47<
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
fn __action48<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, mut v, _): (usize, alloc::vec::Vec<ParsedExpr>, usize),
    (_, e, _): (usize, core::option::Option<ParsedExpr>, usize),
) -> Vec<ParsedExpr>
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
fn __action49<
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
fn __action50<
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
fn __action51<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> alloc::vec::Vec<ParsedStmt>
{
    alloc::vec![]
}

#[allow(unused_variables)]
fn __action52<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<ParsedStmt>, usize),
) -> alloc::vec::Vec<ParsedStmt>
{
    v
}

#[allow(unused_variables)]
fn __action53<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, ParsedStmt, usize),
) -> alloc::vec::Vec<ParsedStmt>
{
    alloc::vec![__0]
}

#[allow(unused_variables)]
fn __action54<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<ParsedStmt>, usize),
    (_, e, _): (usize, ParsedStmt, usize),
) -> alloc::vec::Vec<ParsedStmt>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
fn __action55<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, ParsedExpr, usize),
) -> core::option::Option<ParsedExpr>
{
    Some(__0)
}

#[allow(unused_variables)]
fn __action56<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> core::option::Option<ParsedExpr>
{
    None
}

#[allow(unused_variables)]
fn __action57<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> alloc::vec::Vec<ParsedExpr>
{
    alloc::vec![]
}

#[allow(unused_variables)]
fn __action58<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<ParsedExpr>, usize),
) -> alloc::vec::Vec<ParsedExpr>
{
    v
}

#[allow(unused_variables)]
fn __action59<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, ParsedExpr, usize),
    (_, _, _): (usize, &'input str, usize),
) -> ParsedExpr
{
    __0
}

#[allow(unused_variables)]
fn __action60<
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
fn __action61<
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
fn __action62<
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
fn __action63<
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
fn __action64<
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
fn __action65<
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
fn __action66<
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
fn __action67<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, ParsedExpr, usize),
) -> alloc::vec::Vec<ParsedExpr>
{
    alloc::vec![__0]
}

#[allow(unused_variables)]
fn __action68<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<ParsedExpr>, usize),
    (_, e, _): (usize, ParsedExpr, usize),
) -> alloc::vec::Vec<ParsedExpr>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
fn __action69<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
) -> bool
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action45(
        astf,
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action42(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action70<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> bool
{
    let __start0 = __lookbehind.clone();
    let __end0 = __lookahead.clone();
    let __temp0 = __action46(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action42(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action71<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, ParsedExpr, usize),
    __1: (usize, &'input str, usize),
) -> alloc::vec::Vec<ParsedExpr>
{
    let __start0 = __0.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action59(
        astf,
        input,
        __0,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action67(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action72<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<ParsedExpr>, usize),
    __1: (usize, ParsedExpr, usize),
    __2: (usize, &'input str, usize),
) -> alloc::vec::Vec<ParsedExpr>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action59(
        astf,
        input,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action68(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action73<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, core::option::Option<ParsedExpr>, usize),
) -> Vec<ParsedExpr>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action57(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action48(
        astf,
        input,
        __temp0,
        __0,
    )
}

#[allow(unused_variables)]
fn __action74<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<ParsedExpr>, usize),
    __1: (usize, core::option::Option<ParsedExpr>, usize),
) -> Vec<ParsedExpr>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action58(
        astf,
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action48(
        astf,
        input,
        __temp0,
        __1,
    )
}

#[allow(unused_variables)]
fn __action75<
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
    let __temp0 = __action64(
        astf,
        input,
        __0,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action65(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action76<
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
    let __temp0 = __action64(
        astf,
        input,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action66(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action77<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, core::option::Option<String>, usize),
) -> Vec<String>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action62(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action47(
        astf,
        input,
        __temp0,
        __0,
    )
}

#[allow(unused_variables)]
fn __action78<
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
    let __temp0 = __action63(
        astf,
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action47(
        astf,
        input,
        __temp0,
        __1,
    )
}

#[allow(unused_variables)]
fn __action79<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, ParsedExpr, usize),
    __1: (usize, Binop, usize),
    __2: (usize, ParsedExpr, usize),
    __3: (usize, usize, usize),
) -> ParsedExpr
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action29(
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
fn __action80<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, ParsedExpr, usize),
    __1: (usize, Binop, usize),
    __2: (usize, ParsedExpr, usize),
    __3: (usize, usize, usize),
) -> ParsedExpr
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
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
fn __action81<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, ParsedExpr, usize),
    __1: (usize, Binop, usize),
    __2: (usize, ParsedExpr, usize),
    __3: (usize, usize, usize),
) -> ParsedExpr
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action33(
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
fn __action82<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, ParsedExpr, usize),
    __1: (usize, Binop, usize),
    __2: (usize, ParsedExpr, usize),
    __3: (usize, usize, usize),
) -> ParsedExpr
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
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
fn __action83<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, ParsedExpr, usize),
    __1: (usize, Binop, usize),
    __2: (usize, ParsedExpr, usize),
    __3: (usize, usize, usize),
) -> ParsedExpr
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action25(
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
fn __action84<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, ParsedExpr, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, alloc::vec::Vec<ParsedStmt>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, usize, usize),
) -> ParsedStmt
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
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
    )
}

#[allow(unused_variables)]
fn __action85<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, ParsedExpr, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, alloc::vec::Vec<ParsedStmt>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, usize, usize),
) -> ParsedStmt
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
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
fn __action86<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, &'input str, usize),
    __2: (usize, ParsedExpr, usize),
    __3: (usize, &'input str, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, usize, usize),
) -> ParsedStmt
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
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
    )
}

#[allow(unused_variables)]
fn __action87<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, bool, usize),
    __2: (usize, String, usize),
    __3: (usize, &'input str, usize),
    __4: (usize, ParsedExpr, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, usize, usize),
) -> ParsedStmt
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
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
        __6,
    )
}

#[allow(unused_variables)]
fn __action88<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
    __1: (usize, &'input str, usize),
    __2: (usize, ParsedExpr, usize),
    __3: (usize, &'input str, usize),
    __4: (usize, usize, usize),
) -> ParsedStmt
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action7(
        astf,
        input,
        __temp0,
        __0,
        __1,
        __2,
        __3,
        __4,
    )
}

#[allow(unused_variables)]
fn __action89<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
    __1: (usize, &'input str, usize),
    __2: (usize, Vec<ParsedExpr>, usize),
    __3: (usize, &'input str, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, usize, usize),
) -> ParsedStmt
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action8(
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
fn __action90<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, ParsedExpr, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, usize, usize),
) -> ParsedStmt
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action9(
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
fn __action91<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<String>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, alloc::vec::Vec<ParsedStmt>, usize),
    __7: (usize, &'input str, usize),
    __8: (usize, usize, usize),
) -> ParsedStmt
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action10(
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
    )
}

#[allow(unused_variables)]
fn __action92<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
    __1: (usize, usize, usize),
) -> ParsedExpr
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
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
    )
}

#[allow(unused_variables)]
fn __action93<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
    __1: (usize, usize, usize),
) -> ParsedExpr
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
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
fn __action94<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, bool, usize),
    __1: (usize, usize, usize),
) -> ParsedExpr
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
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
fn __action95<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, &'input str, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, usize, usize),
) -> ParsedExpr
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
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
        __2,
        __3,
    )
}

#[allow(unused_variables)]
fn __action96<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
    __1: (usize, &'input str, usize),
    __2: (usize, Vec<ParsedExpr>, usize),
    __3: (usize, &'input str, usize),
    __4: (usize, usize, usize),
) -> ParsedExpr
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action16(
        astf,
        input,
        __temp0,
        __0,
        __1,
        __2,
        __3,
        __4,
    )
}

#[allow(unused_variables)]
fn __action97<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, ParsedExpr, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, usize, usize),
) -> ParsedExpr
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action17(
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
fn __action98<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, Unop, usize),
    __1: (usize, ParsedExpr, usize),
    __2: (usize, usize, usize),
) -> ParsedExpr
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action50(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action22(
        astf,
        input,
        __temp0,
        __0,
        __1,
        __2,
    )
}

#[allow(unused_variables)]
fn __action99<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, ParsedExpr, usize),
    __1: (usize, Binop, usize),
    __2: (usize, ParsedExpr, usize),
) -> ParsedExpr
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action49(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action79(
        astf,
        input,
        __0,
        __1,
        __2,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action100<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, ParsedExpr, usize),
    __1: (usize, Binop, usize),
    __2: (usize, ParsedExpr, usize),
) -> ParsedExpr
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action49(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action80(
        astf,
        input,
        __0,
        __1,
        __2,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action101<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, ParsedExpr, usize),
    __1: (usize, Binop, usize),
    __2: (usize, ParsedExpr, usize),
) -> ParsedExpr
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action49(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action81(
        astf,
        input,
        __0,
        __1,
        __2,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action102<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, ParsedExpr, usize),
    __1: (usize, Binop, usize),
    __2: (usize, ParsedExpr, usize),
) -> ParsedExpr
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action49(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action82(
        astf,
        input,
        __0,
        __1,
        __2,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action103<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, ParsedExpr, usize),
    __1: (usize, Binop, usize),
    __2: (usize, ParsedExpr, usize),
) -> ParsedExpr
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action49(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action83(
        astf,
        input,
        __0,
        __1,
        __2,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action104<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, ParsedExpr, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, alloc::vec::Vec<ParsedStmt>, usize),
    __4: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __4.2.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action49(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action84(
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
fn __action105<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, ParsedExpr, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, alloc::vec::Vec<ParsedStmt>, usize),
    __4: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __4.2.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action49(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action85(
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
fn __action106<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, &'input str, usize),
    __2: (usize, ParsedExpr, usize),
    __3: (usize, &'input str, usize),
    __4: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __4.2.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action49(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action86(
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
fn __action107<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, bool, usize),
    __2: (usize, String, usize),
    __3: (usize, &'input str, usize),
    __4: (usize, ParsedExpr, usize),
    __5: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __5.2.clone();
    let __end0 = __5.2.clone();
    let __temp0 = __action49(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action87(
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
fn __action108<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
    __1: (usize, &'input str, usize),
    __2: (usize, ParsedExpr, usize),
    __3: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __3.2.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action49(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action88(
        astf,
        input,
        __0,
        __1,
        __2,
        __3,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action109<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
    __1: (usize, &'input str, usize),
    __2: (usize, Vec<ParsedExpr>, usize),
    __3: (usize, &'input str, usize),
    __4: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __4.2.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action49(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action89(
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
fn __action110<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, ParsedExpr, usize),
    __2: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action49(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action90(
        astf,
        input,
        __0,
        __1,
        __2,
        __temp0,
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
    __3: (usize, Vec<String>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, alloc::vec::Vec<ParsedStmt>, usize),
    __7: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __7.2.clone();
    let __end0 = __7.2.clone();
    let __temp0 = __action49(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action91(
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
    )
}

#[allow(unused_variables)]
fn __action112<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
) -> ParsedExpr
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action49(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action92(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action113<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
) -> ParsedExpr
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action49(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action93(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action114<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, bool, usize),
) -> ParsedExpr
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action49(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action94(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action115<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, &'input str, usize),
    __2: (usize, &'input str, usize),
) -> ParsedExpr
{
    let __start0 = __2.2.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action49(
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
        __1,
        __2,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action116<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
    __1: (usize, &'input str, usize),
    __2: (usize, Vec<ParsedExpr>, usize),
    __3: (usize, &'input str, usize),
) -> ParsedExpr
{
    let __start0 = __3.2.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action49(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action96(
        astf,
        input,
        __0,
        __1,
        __2,
        __3,
        __temp0,
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
    __3: (usize, ParsedExpr, usize),
    __4: (usize, &'input str, usize),
) -> ParsedExpr
{
    let __start0 = __4.2.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action49(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action97(
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
fn __action118<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, Unop, usize),
    __1: (usize, ParsedExpr, usize),
) -> ParsedExpr
{
    let __start0 = __1.2.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action49(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action98(
        astf,
        input,
        __0,
        __1,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action119<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, ParsedExpr, usize),
) -> Vec<ParsedExpr>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action55(
        astf,
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action73(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action120<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Vec<ParsedExpr>
{
    let __start0 = __lookbehind.clone();
    let __end0 = __lookahead.clone();
    let __temp0 = __action56(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action73(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action121<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<ParsedExpr>, usize),
    __1: (usize, ParsedExpr, usize),
) -> Vec<ParsedExpr>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action55(
        astf,
        input,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action74(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action122<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<ParsedExpr>, usize),
) -> Vec<ParsedExpr>
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action56(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action74(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action123<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
) -> Vec<String>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action60(
        astf,
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action77(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action124<
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
    let __temp0 = __action61(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action77(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action125<
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
    let __temp0 = __action60(
        astf,
        input,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action78(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action126<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<String>, usize),
) -> Vec<String>
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action61(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action78(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action127<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> ParsedProgram
{
    let __start0 = __lookbehind.clone();
    let __end0 = __lookahead.clone();
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
    )
}

#[allow(unused_variables)]
fn __action128<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<ParsedStmt>, usize),
) -> ParsedProgram
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action52(
        astf,
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action2(
        astf,
        input,
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
    __1: (usize, ParsedExpr, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __2.2.clone();
    let __end0 = __3.0.clone();
    let __temp0 = __action51(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action104(
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
fn __action130<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, ParsedExpr, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, alloc::vec::Vec<ParsedStmt>, usize),
    __4: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __3.0.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action52(
        astf,
        input,
        __3,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action104(
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
fn __action131<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, ParsedExpr, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __2.2.clone();
    let __end0 = __3.0.clone();
    let __temp0 = __action51(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action105(
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
fn __action132<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, ParsedExpr, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, alloc::vec::Vec<ParsedStmt>, usize),
    __4: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __3.0.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action52(
        astf,
        input,
        __3,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action105(
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
fn __action133<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<String>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __5.2.clone();
    let __end0 = __6.0.clone();
    let __temp0 = __action51(
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
        __temp0,
        __6,
    )
}

#[allow(unused_variables)]
fn __action134<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<String>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, alloc::vec::Vec<ParsedStmt>, usize),
    __7: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __6.0.clone();
    let __end0 = __6.2.clone();
    let __temp0 = __action52(
        astf,
        input,
        __6,
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
        __temp0,
        __7,
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

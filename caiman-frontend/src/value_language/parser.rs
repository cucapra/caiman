// auto-generated: "lalrpop 0.19.8"
// sha3: 4acf8f77a3f79ddde5056acaac4ea9a452d1fd2fd0dd3eb16e4ce72bc1bff42f
use crate::value_language::ast::*;
use crate::value_language::typing::Type;
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
    use crate::value_language::typing::Type;
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
        Variant4(VarWithType),
        Variant5(alloc::vec::Vec<VarWithType>),
        Variant6(usize),
        Variant7(Binop),
        Variant8(bool),
        Variant9(Vec<ParsedExpr>),
        Variant10(Vec<VarWithType>),
        Variant11(String),
        Variant12(core::option::Option<ParsedExpr>),
        Variant13(ParsedProgram),
        Variant14(ParsedStmt),
        Variant15(alloc::vec::Vec<ParsedStmt>),
        Variant16(Type),
        Variant17(Unop),
        Variant18(core::option::Option<VarWithType>),
    }
    const __ACTION: &[i8] = &[
        // State 0
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 4, 0, 5, 0, 46, 0, 0, 6, 0, 0, 0, 0, 47,
        // State 1
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 4, 0, 5, 0, 46, 0, 0, 6, 0, 0, 0, 0, 47,
        // State 2
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 47,
        // State 3
        57, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 0, 59, 0, 0, 0, 0, 60, 0, 16, 0, 0, 61, 47,
        // State 4
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 63, 0, 0, 0, 0, 0, 0, 0, 0, -49,
        // State 5
        57, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 0, 59, 0, 0, 0, 0, 60, 0, 16, 0, 0, 61, 47,
        // State 6
        57, 0, 15, -36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 0, 59, 0, 0, 0, 0, 60, 0, 16, 0, 0, 61, 47,
        // State 7
        57, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 0, 59, 0, 0, 0, 0, 60, 0, 16, 0, 0, 61, 47,
        // State 8
        0, -28, 0, -28, 0, 0, -28, 0, 0, 0, 0, -28, 0, 68, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -28, -28, -28, 0, 0,
        // State 9
        0, -22, 0, -22, 0, 69, -22, 70, 0, 0, 0, -22, 0, -22, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -22, -22, -22, 0, 0,
        // State 10
        0, -24, 0, -24, 71, -24, -24, -24, 0, 72, 0, -24, 0, -24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -24, -24, -24, 0, 0,
        // State 11
        0, 73, 0, -30, 0, 0, -30, 0, 0, 0, 0, -30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -30, -30, -30, 0, 0,
        // State 12
        0, 0, 0, -31, 0, 0, -31, 0, 0, 0, 0, -31, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -31, 74, -31, 0, 0,
        // State 13
        57, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 0, 59, 0, 0, 0, 0, 60, 0, 16, 0, 0, 61, 47,
        // State 14
        57, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 0, 59, 0, 0, 0, 0, 60, 0, 16, 0, 0, 61, 47,
        // State 15
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 47,
        // State 16
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 47,
        // State 17
        57, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 0, 59, 0, 0, 0, 0, 60, 0, 16, 0, 0, 61, 47,
        // State 18
        57, 0, 15, -38, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 0, 59, 0, 0, 0, 0, 60, 0, 16, 0, 0, 61, 47,
        // State 19
        0, 0, 0, -40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 63, 0, 0, 0, 0, 0, 0, 0, 0, -49,
        // State 20
        57, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 0, 59, 0, 0, 0, 0, 60, 0, 16, 0, 0, 61, 47,
        // State 21
        57, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 0, 59, 0, 0, 0, 0, 60, 0, 16, 0, 0, 61, 47,
        // State 22
        57, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 0, 59, 0, 0, 0, 0, 60, 0, 16, 0, 0, 61, 47,
        // State 23
        57, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 0, 59, 0, 0, 0, 0, 60, 0, 16, 0, 0, 61, 47,
        // State 24
        57, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 0, 59, 0, 0, 0, 0, 60, 0, 16, 0, 0, 61, 47,
        // State 25
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 4, 0, 5, 0, 46, 0, 0, 6, 0, 0, 88, 0, 47,
        // State 26
        57, 0, 15, -36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 0, 59, 0, 0, 0, 0, 60, 0, 16, 0, 0, 61, 47,
        // State 27
        57, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 0, 59, 0, 0, 0, 0, 60, 0, 16, 0, 0, 61, 47,
        // State 28
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 4, 0, 5, 0, 46, 0, 0, 6, 0, 0, 94, 0, 47,
        // State 29
        0, 0, 0, -42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 63, 0, 0, 0, 0, 0, 0, 0, 0, -49,
        // State 30
        0, -21, 0, -21, 0, 69, -21, 70, 0, 0, 0, -21, 0, -21, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -21, -21, -21, 0, 0,
        // State 31
        0, -23, 0, -23, 71, -23, -23, -23, 0, 72, 0, -23, 0, -23, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -23, -23, -23, 0, 0,
        // State 32
        0, -27, 0, -27, 0, 0, -27, 0, 0, 0, 0, -27, 0, 68, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -27, -27, -27, 0, 0,
        // State 33
        0, 73, 0, -29, 0, 0, -29, 0, 0, 0, 0, -29, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -29, -29, -29, 0, 0,
        // State 34
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 4, 0, 5, 0, 46, 0, 0, 6, 0, 0, 100, 0, 47,
        // State 35
        57, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 0, 59, 0, 0, 0, 0, 60, 0, 16, 0, 0, 61, 47,
        // State 36
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 104, 0, 0, 105, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 37
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 4, 0, 5, 0, 46, 0, 0, 6, 0, 0, 108, 0, 47,
        // State 38
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 104, 0, 0, 105, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 39
        57, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 3, 0, 4, 59, 5, 0, 46, 0, 60, 6, 16, 0, 0, 61, 47,
        // State 40
        57, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 3, 0, 4, 59, 5, 0, 46, 0, 60, 6, 16, 0, 0, 61, 47,
        // State 41
        57, 0, 15, -36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 58, 0, 0, 0, 59, 0, 0, 0, 0, 60, 0, 16, 0, 0, 61, 47,
        // State 42
        0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 43
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 44
        -64, 0, -64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -64, -64, 0, -64, -64, -64, 0, -64, 0, -64, -64, -64, 0, -64, -64, -64,
        // State 45
        0, 0, 18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 46
        0, -47, -47, -47, -47, -47, -47, -47, 0, -47, -47, -47, -47, -47, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -47, -47, -47, 0, 0,
        // State 47
        -65, 0, -65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -65, -65, 0, -65, -65, -65, 0, -65, 0, -65, -65, -65, 0, -65, -65, -65,
        // State 48
        0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 49
        0, 0, 0, -44, 0, 0, -44, 0, 0, 0, 0, -44, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -44, 0, -44, 0, 0,
        // State 50
        0, -68, 0, -68, -68, -68, -68, -68, 0, -68, 0, -68, 0, -68, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -68, -68, -68, 0, 0,
        // State 51
        0, -66, 0, -66, -66, -66, -66, -66, 0, -66, 0, -66, 0, -66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -66, -66, -66, 0, 0,
        // State 52
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 26, 0, 0, 0, 0,
        // State 53
        0, -67, 27, -67, -67, -67, -67, -67, 0, -67, 0, -67, 0, -67, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -67, -67, -67, 0, 0,
        // State 54
        0, -26, 0, -26, -26, -26, -26, -26, 0, -26, 0, -26, 0, -26, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -26, -26, -26, 0, 0,
        // State 55
        0, -72, 0, -72, -72, -72, -72, -72, 0, -72, 0, -72, 0, -72, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -72, -72, -72, 0, 0,
        // State 56
        -77, 0, -77, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -77, 0, 0, 0, -77, 0, 0, 0, 0, -77, 0, -77, 0, 0, -77, -77,
        // State 57
        0, -34, 0, -34, -34, -34, -34, -34, 0, -34, 0, -34, 0, -34, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -34, -34, -34, 0, 0,
        // State 58
        0, 0, 77, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 59
        0, -33, 0, -33, -33, -33, -33, -33, 0, -33, 0, -33, 0, -33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -33, -33, -33, 0, 0,
        // State 60
        0, -43, 0, -43, -43, -43, -43, -43, 0, -43, 0, -43, 0, -43, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -43, -43, -43, 0, 0,
        // State 61
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 28, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 62
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -48,
        // State 63
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 29, 0, 0, 0, 0,
        // State 64
        0, 0, 0, 82, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 65
        0, 0, 0, -35, 0, 0, 83, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 66
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 84, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 67
        -15, 0, -15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -15, 0, 0, 0, -15, 0, 0, 0, 0, -15, 0, -15, 0, 0, -15, -15,
        // State 68
        -16, 0, -16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -16, 0, 0, 0, -16, 0, 0, 0, 0, -16, 0, -16, 0, 0, -16, -16,
        // State 69
        -17, 0, -17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -17, 0, 0, 0, -17, 0, 0, 0, 0, -17, 0, -17, 0, 0, -17, -17,
        // State 70
        -18, 0, -18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -18, 0, 0, 0, -18, 0, 0, 0, 0, -18, 0, -18, 0, 0, -18, -18,
        // State 71
        -19, 0, -19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -19, 0, 0, 0, -19, 0, 0, 0, 0, -19, 0, -19, 0, 0, -19, -19,
        // State 72
        -20, 0, -20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -20, 0, 0, 0, -20, 0, 0, 0, 0, -20, 0, -20, 0, 0, -20, -20,
        // State 73
        -32, 0, -32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -32, 0, 0, 0, -32, 0, 0, 0, 0, -32, 0, -32, 0, 0, -32, -32,
        // State 74
        0, -76, 0, -76, -76, -76, -76, -76, 0, -76, 0, -76, 0, -76, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -76, -76, -76, 0, 0,
        // State 75
        0, 0, 0, 90, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 76
        0, 0, 0, 91, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 77
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 78
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 37, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 79
        0, 0, 0, 93, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 80
        0, 0, 0, -37, 0, 0, 95, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 81
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 96, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 82
        -6, 0, -6, -6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -6, 0, 0, 0, -6, 0, 0, 0, 0, -6, 0, -6, 0, 0, -6, -6,
        // State 83
        -58, 0, -58, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -58, -58, 0, -58, -58, -58, 0, -58, 0, -58, -58, -58, 0, -58, -58, -58,
        // State 84
        0, 0, 0, 98, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 85
        0, 0, 0, -39, 0, 0, 99, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 86
        0, -25, 0, -25, -25, -25, -25, -25, 0, -25, 0, -25, 0, -25, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -25, -25, -25, 0, 0,
        // State 87
        -52, 0, -52, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -52, -52, 0, -52, -52, -52, 0, -52, 0, -52, -52, -52, 0, -52, -52, -52,
        // State 88
        0, 0, 0, 101, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 89
        0, -73, 0, -73, -73, -73, -73, -73, 0, -73, 0, -73, 0, -73, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -73, -73, -73, 0, 0,
        // State 90
        0, -69, 0, -69, -69, -69, -69, -69, 0, -69, 0, -69, 0, -69, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -69, -69, -69, 0, 0,
        // State 91
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 106, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 92
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 107, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 93
        -54, 0, -54, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -54, -54, 0, -54, -54, -54, 0, -54, 0, -54, -54, -54, 0, -54, -54, -54,
        // State 94
        -7, 0, -7, -7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -7, 0, 0, 0, -7, 0, 0, 0, 0, -7, 0, -7, 0, 0, -7, -7,
        // State 95
        -59, 0, -59, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -59, -59, 0, -59, -59, -59, 0, -59, 0, -59, -59, -59, 0, -59, -59, -59,
        // State 96
        0, 0, 0, -41, 0, 0, 109, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 97
        0, 0, 0, 0, 0, 0, 0, 0, 39, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 98
        0, 0, 0, -11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -11, 0, 0, 0, 0, 0, 0, 0, 0, -11,
        // State 99
        -53, 0, -53, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -53, -53, 0, -53, -53, -53, 0, -53, 0, -53, -53, -53, 0, -53, -53, -53,
        // State 100
        0, -70, 0, -70, -70, -70, -70, -70, 0, -70, 0, -70, 0, -70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -70, -70, -70, 0, 0,
        // State 101
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 110, 0, 0,
        // State 102
        0, 0, 0, -78, 0, 0, -78, 0, 0, 0, 0, 0, -78, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 103
        0, 0, 0, -75, 0, 0, -75, 0, 0, 0, 0, 0, -75, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -75, 0, 0, 0, 0,
        // State 104
        0, 0, 0, -74, 0, 0, -74, 0, 0, 0, 0, 0, -74, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -74, 0, 0, 0, 0,
        // State 105
        -57, 0, -57, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -57, -57, 0, -57, -57, -57, 0, -57, 0, -57, -57, -57, 0, -57, -57, -57,
        // State 106
        -56, 0, -56, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -56, -56, 0, -56, -56, -56, 0, -56, 0, -56, -56, -56, 0, -56, -56, -56,
        // State 107
        -55, 0, -55, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -55, -55, 0, -55, -55, -55, 0, -55, 0, -55, -55, -55, 0, -55, -55, -55,
        // State 108
        0, 0, 0, -12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -12, 0, 0, 0, 0, 0, 0, 0, 0, -12,
        // State 109
        0, -71, 0, -71, -71, -71, -71, -71, 0, -71, 0, -71, 0, -71, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -71, -71, -71, 0, 0,
        // State 110
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 40, 0, 0, 0, 0,
        // State 111
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 114, 0, 0,
        // State 112
        0, -67, 42, 0, -67, -67, 0, -67, 0, -67, 0, 0, 8, -67, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -67, -67, 0, 0,
        // State 113
        -60, 0, -60, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -60, -60, 0, -60, -60, -60, 0, -60, 0, -60, -60, -60, 0, -60, -60, -60,
        // State 114
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 117, 0, 0,
        // State 115
        0, 0, 0, 118, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 116
        -61, 0, -61, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -61, -61, 0, -61, -61, -61, 0, -61, 0, -61, -61, -61, 0, -61, -61, -61,
        // State 117
        0, -70, 0, 0, -70, -70, 0, -70, 0, -70, 0, 96, 0, -70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -70, -70, 0, 0,
    ];
    fn __action(state: i8, integer: usize) -> i8 {
        __ACTION[(state as usize) * 31 + integer]
    }
    const __EOF_ACTION: &[i8] = &[
        // State 0
        -50,
        // State 1
        -51,
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
        -81,
        // State 44
        -64,
        // State 45
        0,
        // State 46
        0,
        // State 47
        -65,
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
        0,
        // State 79
        0,
        // State 80
        0,
        // State 81
        0,
        // State 82
        0,
        // State 83
        -58,
        // State 84
        0,
        // State 85
        0,
        // State 86
        0,
        // State 87
        -52,
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
        -54,
        // State 94
        0,
        // State 95
        -59,
        // State 96
        0,
        // State 97
        0,
        // State 98
        0,
        // State 99
        -53,
        // State 100
        0,
        // State 101
        0,
        // State 102
        0,
        // State 103
        0,
        // State 104
        0,
        // State 105
        -57,
        // State 106
        -56,
        // State 107
        -55,
        // State 108
        0,
        // State 109
        0,
        // State 110
        0,
        // State 111
        0,
        // State 112
        0,
        // State 113
        -60,
        // State 114
        0,
        // State 115
        0,
        // State 116
        -61,
        // State 117
        0,
    ];
    fn __goto(state: i8, nt: usize) -> i8 {
        match nt {
            3 => 18,
            6 => 29,
            9 => 20,
            10 => 21,
            11 => 22,
            12 => 23,
            13 => match state {
                23 => 32,
                _ => 8,
            },
            14 => match state {
                20 => 30,
                _ => 9,
            },
            15 => match state {
                21 => 31,
                _ => 10,
            },
            16 => match state {
                24 => 33,
                _ => 11,
            },
            17 => 12,
            18 => 49,
            19 => 24,
            20 => 50,
            21 => match state {
                26 => 88,
                41 => 115,
                _ => 64,
            },
            22 => 84,
            23 => 51,
            24 => match state {
                3 => 52,
                5 => 63,
                7 => 66,
                14 => 75,
                17 => 79,
                18 => 80,
                27 => 91,
                35 => 101,
                39 => 111,
                40 => 114,
                _ => 65,
            },
            26 => match state {
                0..=1 | 25 | 28 | 34 | 37 => 42,
                2 => 48,
                15 => 77,
                16 => 78,
                39..=40 => 112,
                _ => 53,
            },
            27 => 16,
            28 => 43,
            29 => match state {
                1 | 34 | 37 | 40 => 47,
                _ => 44,
            },
            31 => match state {
                25 => 34,
                28 => 37,
                39 => 40,
                _ => 1,
            },
            32 => match state {
                13 => 74,
                22 => 86,
                _ => 54,
            },
            33 => match state {
                38 => 110,
                _ => 102,
            },
            34 => 55,
            35 => 13,
            36 => match state {
                19 => 85,
                29 => 96,
                _ => 61,
            },
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
            r###""->""###,
            r###""/""###,
            r###"":""###,
            r###"";""###,
            r###""=""###,
            r###""==""###,
            r###""bool""###,
            r###""false""###,
            r###""fn""###,
            r###""i32""###,
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
            __action(state, 31 - 1)
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
            Token(30, _) if true => Some(26),
            Token(31, _) if true => Some(27),
            Token(32, _) if true => Some(28),
            Token(1, _) if true => Some(29),
            Token(2, _) if true => Some(30),
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
            0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 30 => match __token {
                Token(4, __tok0) | Token(5, __tok0) | Token(6, __tok0) | Token(7, __tok0) | Token(8, __tok0) | Token(9, __tok0) | Token(10, __tok0) | Token(11, __tok0) | Token(12, __tok0) | Token(13, __tok0) | Token(14, __tok0) | Token(15, __tok0) | Token(16, __tok0) | Token(17, __tok0) | Token(18, __tok0) | Token(19, __tok0) | Token(20, __tok0) | Token(21, __tok0) | Token(22, __tok0) | Token(23, __tok0) | Token(24, __tok0) | Token(25, __tok0) | Token(26, __tok0) | Token(27, __tok0) | Token(28, __tok0) | Token(29, __tok0) | Token(30, __tok0) | Token(31, __tok0) | Token(32, __tok0) | Token(1, __tok0) | Token(2, __tok0) if true => __Symbol::Variant0(__tok0),
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
                __reduce78(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            79 => {
                __reduce79(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            80 => {
                // __Program = Program => ActionFn(0);
                let __sym0 = __pop_Variant13(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action0::<>(astf, input, __sym0);
                return Some(Ok(__nt));
            }
            81 => {
                __reduce81(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
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
    fn __pop_Variant11<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, String, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant11(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant16<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Type, usize)
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
    ) -> (usize, Unop, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant17(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant4<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, VarWithType, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant4(__v), __r)) => (__l, __v, __r),
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
    ) -> (usize, Vec<VarWithType>, usize)
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
    ) -> (usize, alloc::vec::Vec<VarWithType>, usize)
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
    fn __pop_Variant12<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, core::option::Option<ParsedExpr>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant12(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant18<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, core::option::Option<VarWithType>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant18(__v), __r)) => (__l, __v, __r),
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
        // "mut"? = "mut" => ActionFn(47);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action47::<>(astf, input, __sym0);
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
        // "mut"? =  => ActionFn(48);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action48::<>(astf, input, &__start, &__end);
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
        // (<Expr> ",") = Expr, "," => ActionFn(61);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action61::<>(astf, input, __sym0, __sym1);
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
        // (<Expr> ",")* =  => ActionFn(59);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action59::<>(astf, input, &__start, &__end);
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
        // (<Expr> ",")* = (<Expr> ",")+ => ActionFn(60);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action60::<>(astf, input, __sym0);
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
        // (<Expr> ",")+ = Expr, "," => ActionFn(73);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action73::<>(astf, input, __sym0, __sym1);
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
        // (<Expr> ",")+ = (<Expr> ",")+, Expr, "," => ActionFn(74);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action74::<>(astf, input, __sym0, __sym1, __sym2);
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
        // (<VarWithType> ",") = VarWithType, "," => ActionFn(66);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action66::<>(astf, input, __sym0, __sym1);
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
        // (<VarWithType> ",")* =  => ActionFn(64);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action64::<>(astf, input, &__start, &__end);
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
        // (<VarWithType> ",")* = (<VarWithType> ",")+ => ActionFn(65);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action65::<>(astf, input, __sym0);
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
        // (<VarWithType> ",")+ = VarWithType, "," => ActionFn(77);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action77::<>(astf, input, __sym0, __sym1);
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
        // (<VarWithType> ",")+ = (<VarWithType> ",")+, VarWithType, "," => ActionFn(78);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant4(__symbols);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action78::<>(astf, input, __sym0, __sym1, __sym2);
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
        // @L =  => ActionFn(52);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action52::<>(astf, input, &__start, &__end);
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
        // @R =  => ActionFn(51);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action51::<>(astf, input, &__start, &__end);
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
        // BinopE1 = BinopE1, Binop1, BinopE2 => ActionFn(100);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action100::<>(astf, input, __sym0, __sym1, __sym2);
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
        // BinopE2 = BinopE2, Binop2, BinopE3 => ActionFn(101);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action101::<>(astf, input, __sym0, __sym1, __sym2);
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
        // BinopE3 = BinopE3, Binop3, TermExpr => ActionFn(102);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action102::<>(astf, input, __sym0, __sym1, __sym2);
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
        // BinopEAnd = BinopEAnd, BinopAnd, BinopE1 => ActionFn(103);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action103::<>(astf, input, __sym0, __sym1, __sym2);
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
        // BinopEOr = BinopEOr, BinopOr, BinopEAnd => ActionFn(104);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action104::<>(astf, input, __sym0, __sym1, __sym2);
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
        // CommaList<VarWithType> = VarWithType => ActionFn(131);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action131::<>(astf, input, __sym0);
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
        // CommaList<VarWithType> =  => ActionFn(132);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action132::<>(astf, input, &__start, &__end);
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
        // CommaList<VarWithType> = (<VarWithType> ",")+, VarWithType => ActionFn(133);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant4(__symbols);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action133::<>(astf, input, __sym0, __sym1);
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
        // CommaList<VarWithType> = (<VarWithType> ",")+ => ActionFn(134);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action134::<>(astf, input, __sym0);
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
        __symbols.push((__start, __Symbol::Variant11(__nt), __end));
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
        // Expr? = Expr => ActionFn(57);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action57::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant12(__nt), __end));
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
        // Expr? =  => ActionFn(58);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action58::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant12(__nt), __end));
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
        __symbols.push((__start, __Symbol::Variant11(__nt), __end));
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
        // Mut = "mut" => ActionFn(71);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action71::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
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
        // Mut =  => ActionFn(72);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action72::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
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
        // Program =  => ActionFn(123);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action123::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant13(__nt), __end));
        (0, 28)
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
        // Program = Stmt+ => ActionFn(124);
        let __sym0 = __pop_Variant15(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action124::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant13(__nt), __end));
        (1, 28)
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
        // Stmt = "if", Expr, "{", "}" => ActionFn(125);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action125::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (4, 29)
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
        // Stmt = "if", Expr, "{", Stmt+, "}" => ActionFn(126);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant15(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action126::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (5, 29)
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
        // Stmt = "while", Expr, "{", "}" => ActionFn(127);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action127::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (4, 29)
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
        // Stmt = "while", Expr, "{", Stmt+, "}" => ActionFn(128);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant15(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action128::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (5, 29)
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
        // Stmt = "print", "(", Expr, ")", ";" => ActionFn(107);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action107::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (5, 29)
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
        // Stmt = "let", VarWithType, "=", Expr, ";" => ActionFn(108);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant2(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant4(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action108::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (5, 29)
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
        // Stmt = Id, "=", Expr, ";" => ActionFn(109);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant11(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action109::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (4, 29)
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
        // Stmt = Id, "(", CommaList<Expr>, ")", ";" => ActionFn(110);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant9(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant11(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action110::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (5, 29)
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
        // Stmt = "fn", Id, "(", CommaList<VarWithType>, ")", "->", Type, "{", Expr, "}" => ActionFn(129);
        assert!(__symbols.len() >= 10);
        let __sym9 = __pop_Variant0(__symbols);
        let __sym8 = __pop_Variant2(__symbols);
        let __sym7 = __pop_Variant0(__symbols);
        let __sym6 = __pop_Variant16(__symbols);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant10(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant11(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym9.2.clone();
        let __nt = super::__action129::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (10, 29)
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
        // Stmt = "fn", Id, "(", CommaList<VarWithType>, ")", "->", Type, "{", Stmt+, Expr, "}" => ActionFn(130);
        assert!(__symbols.len() >= 11);
        let __sym10 = __pop_Variant0(__symbols);
        let __sym9 = __pop_Variant2(__symbols);
        let __sym8 = __pop_Variant15(__symbols);
        let __sym7 = __pop_Variant0(__symbols);
        let __sym6 = __pop_Variant16(__symbols);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant10(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant11(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym10.2.clone();
        let __nt = super::__action130::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9, __sym10);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (11, 29)
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
        // Stmt* =  => ActionFn(53);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action53::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (0, 30)
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
        // Stmt* = Stmt+ => ActionFn(54);
        let __sym0 = __pop_Variant15(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action54::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (1, 30)
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
        // Stmt+ = Stmt => ActionFn(55);
        let __sym0 = __pop_Variant14(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action55::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (1, 31)
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
        // Stmt+ = Stmt+, Stmt => ActionFn(56);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant14(__symbols);
        let __sym0 = __pop_Variant15(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action56::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (2, 31)
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
        // TermExpr = Constant => ActionFn(112);
        let __sym0 = __pop_Variant11(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action112::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 32)
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
        // TermExpr = Id => ActionFn(113);
        let __sym0 = __pop_Variant11(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action113::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
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
        // TermExpr = Bool => ActionFn(114);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action114::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
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
        // TermExpr = "input", "(", ")" => ActionFn(115);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action115::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (3, 32)
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
        // TermExpr = Id, "(", CommaList<Expr>, ")" => ActionFn(116);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant9(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant11(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action116::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (4, 32)
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
        // TermExpr = "{", Id, ":", Expr, "}" => ActionFn(117);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant2(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant11(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action117::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (5, 32)
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
        // TermExpr = UnopExpr => ActionFn(18);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action18::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 32)
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
        // TermExpr = "(", Expr, ")" => ActionFn(19);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action19::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (3, 32)
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
        // Type = "i32" => ActionFn(45);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action45::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant16(__nt), __end));
        (1, 33)
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
        // Type = "bool" => ActionFn(46);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action46::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant16(__nt), __end));
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
        // UnopExpr = UnopNot, TermExpr => ActionFn(118);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant17(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action118::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (2, 34)
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
        // UnopNot = "!" => ActionFn(23);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action23::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant17(__nt), __end));
        (1, 35)
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
        // VarWithType = Mut, Id, ":", Type => ActionFn(10);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant16(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant11(__symbols);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action10::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (4, 36)
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
        // VarWithType? = VarWithType => ActionFn(62);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action62::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant18(__nt), __end));
        (1, 37)
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
        // VarWithType? =  => ActionFn(63);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action63::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant18(__nt), __end));
        (0, 37)
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
        // __Stmt = Stmt => ActionFn(1);
        let __sym0 = __pop_Variant14(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action1::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (1, 39)
    }
}
pub use self::__parse__Program::ProgramParser;

#[cfg_attr(rustfmt, rustfmt_skip)]
mod __parse__Stmt {
    #![allow(non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens, clippy::all)]

    use crate::value_language::ast::*;
    use crate::value_language::typing::Type;
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
        Variant4(VarWithType),
        Variant5(alloc::vec::Vec<VarWithType>),
        Variant6(usize),
        Variant7(Binop),
        Variant8(bool),
        Variant9(Vec<ParsedExpr>),
        Variant10(Vec<VarWithType>),
        Variant11(String),
        Variant12(core::option::Option<ParsedExpr>),
        Variant13(ParsedProgram),
        Variant14(ParsedStmt),
        Variant15(alloc::vec::Vec<ParsedStmt>),
        Variant16(Type),
        Variant17(Unop),
        Variant18(core::option::Option<VarWithType>),
    }
    const __ACTION: &[i8] = &[
        // State 0
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 3, 0, 4, 0, 44, 0, 0, 5, 0, 0, 0, 0, 45,
        // State 1
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 45,
        // State 2
        54, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 0, 0, 0, 56, 0, 0, 0, 0, 57, 0, 15, 0, 0, 58, 45,
        // State 3
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 60, 0, 0, 0, 0, 0, 0, 0, 0, -49,
        // State 4
        54, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 0, 0, 0, 56, 0, 0, 0, 0, 57, 0, 15, 0, 0, 58, 45,
        // State 5
        54, 0, 14, -36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 0, 0, 0, 56, 0, 0, 0, 0, 57, 0, 15, 0, 0, 58, 45,
        // State 6
        54, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 0, 0, 0, 56, 0, 0, 0, 0, 57, 0, 15, 0, 0, 58, 45,
        // State 7
        0, -28, 0, -28, 0, 0, -28, 0, 0, 0, 0, -28, 0, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -28, -28, -28, 0, 0,
        // State 8
        0, -22, 0, -22, 0, 66, -22, 67, 0, 0, 0, -22, 0, -22, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -22, -22, -22, 0, 0,
        // State 9
        0, -24, 0, -24, 68, -24, -24, -24, 0, 69, 0, -24, 0, -24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -24, -24, -24, 0, 0,
        // State 10
        0, 70, 0, -30, 0, 0, -30, 0, 0, 0, 0, -30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -30, -30, -30, 0, 0,
        // State 11
        0, 0, 0, -31, 0, 0, -31, 0, 0, 0, 0, -31, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -31, 71, -31, 0, 0,
        // State 12
        54, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 0, 0, 0, 56, 0, 0, 0, 0, 57, 0, 15, 0, 0, 58, 45,
        // State 13
        54, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 0, 0, 0, 56, 0, 0, 0, 0, 57, 0, 15, 0, 0, 58, 45,
        // State 14
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 45,
        // State 15
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 45,
        // State 16
        54, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 0, 0, 0, 56, 0, 0, 0, 0, 57, 0, 15, 0, 0, 58, 45,
        // State 17
        54, 0, 14, -38, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 0, 0, 0, 56, 0, 0, 0, 0, 57, 0, 15, 0, 0, 58, 45,
        // State 18
        0, 0, 0, -40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 60, 0, 0, 0, 0, 0, 0, 0, 0, -49,
        // State 19
        54, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 0, 0, 0, 56, 0, 0, 0, 0, 57, 0, 15, 0, 0, 58, 45,
        // State 20
        54, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 0, 0, 0, 56, 0, 0, 0, 0, 57, 0, 15, 0, 0, 58, 45,
        // State 21
        54, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 0, 0, 0, 56, 0, 0, 0, 0, 57, 0, 15, 0, 0, 58, 45,
        // State 22
        54, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 0, 0, 0, 56, 0, 0, 0, 0, 57, 0, 15, 0, 0, 58, 45,
        // State 23
        54, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 0, 0, 0, 56, 0, 0, 0, 0, 57, 0, 15, 0, 0, 58, 45,
        // State 24
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 3, 0, 4, 0, 44, 0, 0, 5, 0, 0, 86, 0, 45,
        // State 25
        54, 0, 14, -36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 0, 0, 0, 56, 0, 0, 0, 0, 57, 0, 15, 0, 0, 58, 45,
        // State 26
        54, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 0, 0, 0, 56, 0, 0, 0, 0, 57, 0, 15, 0, 0, 58, 45,
        // State 27
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 3, 0, 4, 0, 44, 0, 0, 5, 0, 0, 92, 0, 45,
        // State 28
        0, 0, 0, -42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 60, 0, 0, 0, 0, 0, 0, 0, 0, -49,
        // State 29
        0, -21, 0, -21, 0, 66, -21, 67, 0, 0, 0, -21, 0, -21, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -21, -21, -21, 0, 0,
        // State 30
        0, -23, 0, -23, 68, -23, -23, -23, 0, 69, 0, -23, 0, -23, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -23, -23, -23, 0, 0,
        // State 31
        0, -27, 0, -27, 0, 0, -27, 0, 0, 0, 0, -27, 0, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -27, -27, -27, 0, 0,
        // State 32
        0, 70, 0, -29, 0, 0, -29, 0, 0, 0, 0, -29, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -29, -29, -29, 0, 0,
        // State 33
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 3, 0, 4, 0, 44, 0, 0, 5, 0, 0, 99, 0, 45,
        // State 34
        54, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 0, 0, 0, 56, 0, 0, 0, 0, 57, 0, 15, 0, 0, 58, 45,
        // State 35
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 103, 0, 0, 104, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 36
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 3, 0, 4, 0, 44, 0, 0, 5, 0, 0, 107, 0, 45,
        // State 37
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 103, 0, 0, 104, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 38
        54, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 2, 0, 3, 56, 4, 0, 44, 0, 57, 5, 15, 0, 0, 58, 45,
        // State 39
        54, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 2, 0, 3, 56, 4, 0, 44, 0, 57, 5, 15, 0, 0, 58, 45,
        // State 40
        54, 0, 14, -36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 0, 0, 0, 56, 0, 0, 0, 0, 57, 0, 15, 0, 0, 58, 45,
        // State 41
        0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 42
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 43
        0, 0, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 44
        0, -47, -47, -47, -47, -47, -47, -47, 0, -47, -47, -47, -47, -47, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -47, -47, -47, 0, 0,
        // State 45
        0, 0, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 46
        0, 0, 0, -44, 0, 0, -44, 0, 0, 0, 0, -44, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -44, 0, -44, 0, 0,
        // State 47
        0, -68, 0, -68, -68, -68, -68, -68, 0, -68, 0, -68, 0, -68, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -68, -68, -68, 0, 0,
        // State 48
        0, -66, 0, -66, -66, -66, -66, -66, 0, -66, 0, -66, 0, -66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -66, -66, -66, 0, 0,
        // State 49
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 25, 0, 0, 0, 0,
        // State 50
        0, -67, 26, -67, -67, -67, -67, -67, 0, -67, 0, -67, 0, -67, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -67, -67, -67, 0, 0,
        // State 51
        0, -26, 0, -26, -26, -26, -26, -26, 0, -26, 0, -26, 0, -26, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -26, -26, -26, 0, 0,
        // State 52
        0, -72, 0, -72, -72, -72, -72, -72, 0, -72, 0, -72, 0, -72, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -72, -72, -72, 0, 0,
        // State 53
        -77, 0, -77, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -77, 0, 0, 0, -77, 0, 0, 0, 0, -77, 0, -77, 0, 0, -77, -77,
        // State 54
        0, -34, 0, -34, -34, -34, -34, -34, 0, -34, 0, -34, 0, -34, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -34, -34, -34, 0, 0,
        // State 55
        0, 0, 74, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 56
        0, -33, 0, -33, -33, -33, -33, -33, 0, -33, 0, -33, 0, -33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -33, -33, -33, 0, 0,
        // State 57
        0, -43, 0, -43, -43, -43, -43, -43, 0, -43, 0, -43, 0, -43, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -43, -43, -43, 0, 0,
        // State 58
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 27, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 59
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -48,
        // State 60
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 28, 0, 0, 0, 0,
        // State 61
        0, 0, 0, 79, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 62
        0, 0, 0, -35, 0, 0, 80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 63
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 81, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 64
        -15, 0, -15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -15, 0, 0, 0, -15, 0, 0, 0, 0, -15, 0, -15, 0, 0, -15, -15,
        // State 65
        -16, 0, -16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -16, 0, 0, 0, -16, 0, 0, 0, 0, -16, 0, -16, 0, 0, -16, -16,
        // State 66
        -17, 0, -17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -17, 0, 0, 0, -17, 0, 0, 0, 0, -17, 0, -17, 0, 0, -17, -17,
        // State 67
        -18, 0, -18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -18, 0, 0, 0, -18, 0, 0, 0, 0, -18, 0, -18, 0, 0, -18, -18,
        // State 68
        -19, 0, -19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -19, 0, 0, 0, -19, 0, 0, 0, 0, -19, 0, -19, 0, 0, -19, -19,
        // State 69
        -20, 0, -20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -20, 0, 0, 0, -20, 0, 0, 0, 0, -20, 0, -20, 0, 0, -20, -20,
        // State 70
        -32, 0, -32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -32, 0, 0, 0, -32, 0, 0, 0, 0, -32, 0, -32, 0, 0, -32, -32,
        // State 71
        0, -76, 0, -76, -76, -76, -76, -76, 0, -76, 0, -76, 0, -76, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -76, -76, -76, 0, 0,
        // State 72
        0, 0, 0, 88, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 73
        0, 0, 0, 89, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 74
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 35, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 75
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 76
        0, 0, 0, 91, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 77
        0, 0, 0, -37, 0, 0, 93, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 78
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 94, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 79
        -6, 0, -6, -6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -6, 0, 0, 0, -6, 0, 0, 0, 0, -6, 0, -6, 0, 0, -6, -6,
        // State 80
        -58, 0, -58, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -58, -58, 0, -58, -58, -58, 0, -58, 0, -58, -58, -58, 0, -58, -58, -58,
        // State 81
        0, 0, 0, 96, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 82
        0, 0, 0, -39, 0, 0, 97, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 83
        0, -25, 0, -25, -25, -25, -25, -25, 0, -25, 0, -25, 0, -25, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -25, -25, -25, 0, 0,
        // State 84
        -64, 0, -64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -64, -64, 0, -64, -64, -64, 0, -64, 0, -64, -64, -64, 0, -64, -64, -64,
        // State 85
        -52, 0, -52, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -52, -52, 0, -52, -52, -52, 0, -52, 0, -52, -52, -52, 0, -52, -52, -52,
        // State 86
        0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 87
        0, -73, 0, -73, -73, -73, -73, -73, 0, -73, 0, -73, 0, -73, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -73, -73, -73, 0, 0,
        // State 88
        0, -69, 0, -69, -69, -69, -69, -69, 0, -69, 0, -69, 0, -69, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -69, -69, -69, 0, 0,
        // State 89
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 105, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 90
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 106, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 91
        -54, 0, -54, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -54, -54, 0, -54, -54, -54, 0, -54, 0, -54, -54, -54, 0, -54, -54, -54,
        // State 92
        -7, 0, -7, -7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -7, 0, 0, 0, -7, 0, 0, 0, 0, -7, 0, -7, 0, 0, -7, -7,
        // State 93
        -59, 0, -59, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -59, -59, 0, -59, -59, -59, 0, -59, 0, -59, -59, -59, 0, -59, -59, -59,
        // State 94
        0, 0, 0, -41, 0, 0, 108, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 95
        0, 0, 0, 0, 0, 0, 0, 0, 38, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 96
        0, 0, 0, -11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -11, 0, 0, 0, 0, 0, 0, 0, 0, -11,
        // State 97
        -65, 0, -65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -65, -65, 0, -65, -65, -65, 0, -65, 0, -65, -65, -65, 0, -65, -65, -65,
        // State 98
        -53, 0, -53, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -53, -53, 0, -53, -53, -53, 0, -53, 0, -53, -53, -53, 0, -53, -53, -53,
        // State 99
        0, -70, 0, -70, -70, -70, -70, -70, 0, -70, 0, -70, 0, -70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -70, -70, -70, 0, 0,
        // State 100
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 109, 0, 0,
        // State 101
        0, 0, 0, -78, 0, 0, -78, 0, 0, 0, 0, 0, -78, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 102
        0, 0, 0, -75, 0, 0, -75, 0, 0, 0, 0, 0, -75, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -75, 0, 0, 0, 0,
        // State 103
        0, 0, 0, -74, 0, 0, -74, 0, 0, 0, 0, 0, -74, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -74, 0, 0, 0, 0,
        // State 104
        -57, 0, -57, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -57, -57, 0, -57, -57, -57, 0, -57, 0, -57, -57, -57, 0, -57, -57, -57,
        // State 105
        -56, 0, -56, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -56, -56, 0, -56, -56, -56, 0, -56, 0, -56, -56, -56, 0, -56, -56, -56,
        // State 106
        -55, 0, -55, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -55, -55, 0, -55, -55, -55, 0, -55, 0, -55, -55, -55, 0, -55, -55, -55,
        // State 107
        0, 0, 0, -12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -12, 0, 0, 0, 0, 0, 0, 0, 0, -12,
        // State 108
        0, -71, 0, -71, -71, -71, -71, -71, 0, -71, 0, -71, 0, -71, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -71, -71, -71, 0, 0,
        // State 109
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 39, 0, 0, 0, 0,
        // State 110
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 113, 0, 0,
        // State 111
        0, -67, 41, 0, -67, -67, 0, -67, 0, -67, 0, 0, 7, -67, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -67, -67, 0, 0,
        // State 112
        -60, 0, -60, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -60, -60, 0, -60, -60, -60, 0, -60, 0, -60, -60, -60, 0, -60, -60, -60,
        // State 113
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 116, 0, 0,
        // State 114
        0, 0, 0, 117, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 115
        -61, 0, -61, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -61, -61, 0, -61, -61, -61, 0, -61, 0, -61, -61, -61, 0, -61, -61, -61,
        // State 116
        0, -70, 0, 0, -70, -70, 0, -70, 0, -70, 0, 94, 0, -70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -70, -70, 0, 0,
    ];
    fn __action(state: i8, integer: usize) -> i8 {
        __ACTION[(state as usize) * 31 + integer]
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
        0,
        // State 41
        0,
        // State 42
        -82,
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
        -58,
        // State 81
        0,
        // State 82
        0,
        // State 83
        0,
        // State 84
        0,
        // State 85
        -52,
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
        -54,
        // State 92
        0,
        // State 93
        -59,
        // State 94
        0,
        // State 95
        0,
        // State 96
        0,
        // State 97
        0,
        // State 98
        -53,
        // State 99
        0,
        // State 100
        0,
        // State 101
        0,
        // State 102
        0,
        // State 103
        0,
        // State 104
        -57,
        // State 105
        -56,
        // State 106
        -55,
        // State 107
        0,
        // State 108
        0,
        // State 109
        0,
        // State 110
        0,
        // State 111
        0,
        // State 112
        -60,
        // State 113
        0,
        // State 114
        0,
        // State 115
        -61,
        // State 116
        0,
    ];
    fn __goto(state: i8, nt: usize) -> i8 {
        match nt {
            3 => 17,
            6 => 28,
            9 => 19,
            10 => 20,
            11 => 21,
            12 => 22,
            13 => match state {
                22 => 31,
                _ => 7,
            },
            14 => match state {
                19 => 29,
                _ => 8,
            },
            15 => match state {
                20 => 30,
                _ => 9,
            },
            16 => match state {
                23 => 32,
                _ => 10,
            },
            17 => 11,
            18 => 46,
            19 => 23,
            20 => 47,
            21 => match state {
                25 => 86,
                40 => 114,
                _ => 61,
            },
            22 => 81,
            23 => 48,
            24 => match state {
                2 => 49,
                4 => 60,
                6 => 63,
                13 => 72,
                16 => 76,
                17 => 77,
                26 => 89,
                34 => 100,
                38 => 110,
                39 => 113,
                _ => 62,
            },
            26 => match state {
                0 | 24 | 27 | 33 | 36 => 41,
                1 => 45,
                14 => 74,
                15 => 75,
                38..=39 => 111,
                _ => 50,
            },
            27 => 15,
            29 => match state {
                0 => 42,
                33 | 36 | 39 => 97,
                _ => 84,
            },
            31 => match state {
                27 => 36,
                38 => 39,
                _ => 33,
            },
            32 => match state {
                12 => 71,
                21 => 83,
                _ => 51,
            },
            33 => match state {
                37 => 109,
                _ => 101,
            },
            34 => 52,
            35 => 12,
            36 => match state {
                18 => 82,
                28 => 94,
                _ => 58,
            },
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
            r###""->""###,
            r###""/""###,
            r###"":""###,
            r###"";""###,
            r###""=""###,
            r###""==""###,
            r###""bool""###,
            r###""false""###,
            r###""fn""###,
            r###""i32""###,
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
            __action(state, 31 - 1)
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
            Token(30, _) if true => Some(26),
            Token(31, _) if true => Some(27),
            Token(32, _) if true => Some(28),
            Token(1, _) if true => Some(29),
            Token(2, _) if true => Some(30),
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
            0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 30 => match __token {
                Token(4, __tok0) | Token(5, __tok0) | Token(6, __tok0) | Token(7, __tok0) | Token(8, __tok0) | Token(9, __tok0) | Token(10, __tok0) | Token(11, __tok0) | Token(12, __tok0) | Token(13, __tok0) | Token(14, __tok0) | Token(15, __tok0) | Token(16, __tok0) | Token(17, __tok0) | Token(18, __tok0) | Token(19, __tok0) | Token(20, __tok0) | Token(21, __tok0) | Token(22, __tok0) | Token(23, __tok0) | Token(24, __tok0) | Token(25, __tok0) | Token(26, __tok0) | Token(27, __tok0) | Token(28, __tok0) | Token(29, __tok0) | Token(30, __tok0) | Token(31, __tok0) | Token(32, __tok0) | Token(1, __tok0) | Token(2, __tok0) if true => __Symbol::Variant0(__tok0),
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
                __reduce79(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            80 => {
                __reduce80(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
            }
            81 => {
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
    fn __pop_Variant11<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, String, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant11(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant16<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, Type, usize)
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
    ) -> (usize, Unop, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant17(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant4<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, VarWithType, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant4(__v), __r)) => (__l, __v, __r),
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
    ) -> (usize, Vec<VarWithType>, usize)
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
    ) -> (usize, alloc::vec::Vec<VarWithType>, usize)
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
    fn __pop_Variant12<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, core::option::Option<ParsedExpr>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant12(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant18<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, core::option::Option<VarWithType>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant18(__v), __r)) => (__l, __v, __r),
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
        // "mut"? = "mut" => ActionFn(47);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action47::<>(astf, input, __sym0);
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
        // "mut"? =  => ActionFn(48);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action48::<>(astf, input, &__start, &__end);
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
        // (<Expr> ",") = Expr, "," => ActionFn(61);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action61::<>(astf, input, __sym0, __sym1);
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
        // (<Expr> ",")* =  => ActionFn(59);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action59::<>(astf, input, &__start, &__end);
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
        // (<Expr> ",")* = (<Expr> ",")+ => ActionFn(60);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action60::<>(astf, input, __sym0);
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
        // (<Expr> ",")+ = Expr, "," => ActionFn(73);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action73::<>(astf, input, __sym0, __sym1);
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
        // (<Expr> ",")+ = (<Expr> ",")+, Expr, "," => ActionFn(74);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant3(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action74::<>(astf, input, __sym0, __sym1, __sym2);
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
        // (<VarWithType> ",") = VarWithType, "," => ActionFn(66);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action66::<>(astf, input, __sym0, __sym1);
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
        // (<VarWithType> ",")* =  => ActionFn(64);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action64::<>(astf, input, &__start, &__end);
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
        // (<VarWithType> ",")* = (<VarWithType> ",")+ => ActionFn(65);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action65::<>(astf, input, __sym0);
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
        // (<VarWithType> ",")+ = VarWithType, "," => ActionFn(77);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action77::<>(astf, input, __sym0, __sym1);
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
        // (<VarWithType> ",")+ = (<VarWithType> ",")+, VarWithType, "," => ActionFn(78);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant4(__symbols);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action78::<>(astf, input, __sym0, __sym1, __sym2);
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
        // @L =  => ActionFn(52);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action52::<>(astf, input, &__start, &__end);
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
        // @R =  => ActionFn(51);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action51::<>(astf, input, &__start, &__end);
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
        // BinopE1 = BinopE1, Binop1, BinopE2 => ActionFn(100);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action100::<>(astf, input, __sym0, __sym1, __sym2);
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
        // BinopE2 = BinopE2, Binop2, BinopE3 => ActionFn(101);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action101::<>(astf, input, __sym0, __sym1, __sym2);
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
        // BinopE3 = BinopE3, Binop3, TermExpr => ActionFn(102);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action102::<>(astf, input, __sym0, __sym1, __sym2);
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
        // BinopEAnd = BinopEAnd, BinopAnd, BinopE1 => ActionFn(103);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action103::<>(astf, input, __sym0, __sym1, __sym2);
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
        // BinopEOr = BinopEOr, BinopOr, BinopEAnd => ActionFn(104);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action104::<>(astf, input, __sym0, __sym1, __sym2);
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
        // CommaList<VarWithType> = VarWithType => ActionFn(131);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action131::<>(astf, input, __sym0);
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
        // CommaList<VarWithType> =  => ActionFn(132);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action132::<>(astf, input, &__start, &__end);
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
        // CommaList<VarWithType> = (<VarWithType> ",")+, VarWithType => ActionFn(133);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant4(__symbols);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action133::<>(astf, input, __sym0, __sym1);
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
        // CommaList<VarWithType> = (<VarWithType> ",")+ => ActionFn(134);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action134::<>(astf, input, __sym0);
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
        __symbols.push((__start, __Symbol::Variant11(__nt), __end));
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
        // Expr? = Expr => ActionFn(57);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action57::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant12(__nt), __end));
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
        // Expr? =  => ActionFn(58);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action58::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant12(__nt), __end));
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
        __symbols.push((__start, __Symbol::Variant11(__nt), __end));
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
        // Mut = "mut" => ActionFn(71);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action71::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
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
        // Mut =  => ActionFn(72);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action72::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
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
        // Program =  => ActionFn(123);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action123::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant13(__nt), __end));
        (0, 28)
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
        // Program = Stmt+ => ActionFn(124);
        let __sym0 = __pop_Variant15(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action124::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant13(__nt), __end));
        (1, 28)
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
        // Stmt = "if", Expr, "{", "}" => ActionFn(125);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action125::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (4, 29)
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
        // Stmt = "if", Expr, "{", Stmt+, "}" => ActionFn(126);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant15(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action126::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (5, 29)
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
        // Stmt = "while", Expr, "{", "}" => ActionFn(127);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action127::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (4, 29)
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
        // Stmt = "while", Expr, "{", Stmt+, "}" => ActionFn(128);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant15(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action128::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (5, 29)
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
        // Stmt = "print", "(", Expr, ")", ";" => ActionFn(107);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action107::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (5, 29)
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
        // Stmt = "let", VarWithType, "=", Expr, ";" => ActionFn(108);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant2(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant4(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action108::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (5, 29)
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
        // Stmt = Id, "=", Expr, ";" => ActionFn(109);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant2(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant11(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action109::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (4, 29)
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
        // Stmt = Id, "(", CommaList<Expr>, ")", ";" => ActionFn(110);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant9(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant11(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action110::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (5, 29)
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
        // Stmt = "fn", Id, "(", CommaList<VarWithType>, ")", "->", Type, "{", Expr, "}" => ActionFn(129);
        assert!(__symbols.len() >= 10);
        let __sym9 = __pop_Variant0(__symbols);
        let __sym8 = __pop_Variant2(__symbols);
        let __sym7 = __pop_Variant0(__symbols);
        let __sym6 = __pop_Variant16(__symbols);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant10(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant11(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym9.2.clone();
        let __nt = super::__action129::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (10, 29)
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
        // Stmt = "fn", Id, "(", CommaList<VarWithType>, ")", "->", Type, "{", Stmt+, Expr, "}" => ActionFn(130);
        assert!(__symbols.len() >= 11);
        let __sym10 = __pop_Variant0(__symbols);
        let __sym9 = __pop_Variant2(__symbols);
        let __sym8 = __pop_Variant15(__symbols);
        let __sym7 = __pop_Variant0(__symbols);
        let __sym6 = __pop_Variant16(__symbols);
        let __sym5 = __pop_Variant0(__symbols);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant10(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant11(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym10.2.clone();
        let __nt = super::__action130::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4, __sym5, __sym6, __sym7, __sym8, __sym9, __sym10);
        __symbols.push((__start, __Symbol::Variant14(__nt), __end));
        (11, 29)
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
        // Stmt* =  => ActionFn(53);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action53::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (0, 30)
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
        // Stmt* = Stmt+ => ActionFn(54);
        let __sym0 = __pop_Variant15(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action54::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (1, 30)
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
        // Stmt+ = Stmt => ActionFn(55);
        let __sym0 = __pop_Variant14(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action55::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (1, 31)
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
        // Stmt+ = Stmt+, Stmt => ActionFn(56);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant14(__symbols);
        let __sym0 = __pop_Variant15(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action56::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant15(__nt), __end));
        (2, 31)
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
        // TermExpr = Constant => ActionFn(112);
        let __sym0 = __pop_Variant11(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action112::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 32)
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
        // TermExpr = Id => ActionFn(113);
        let __sym0 = __pop_Variant11(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action113::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
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
        // TermExpr = Bool => ActionFn(114);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action114::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
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
        // TermExpr = "input", "(", ")" => ActionFn(115);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action115::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (3, 32)
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
        // TermExpr = Id, "(", CommaList<Expr>, ")" => ActionFn(116);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant9(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant11(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action116::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (4, 32)
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
        // TermExpr = "{", Id, ":", Expr, "}" => ActionFn(117);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant2(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant11(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action117::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (5, 32)
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
        // TermExpr = UnopExpr => ActionFn(18);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action18::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (1, 32)
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
        // TermExpr = "(", Expr, ")" => ActionFn(19);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action19::<>(astf, input, __sym0, __sym1, __sym2);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (3, 32)
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
        // Type = "i32" => ActionFn(45);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action45::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant16(__nt), __end));
        (1, 33)
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
        // Type = "bool" => ActionFn(46);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action46::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant16(__nt), __end));
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
        // UnopExpr = UnopNot, TermExpr => ActionFn(118);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant17(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action118::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant2(__nt), __end));
        (2, 34)
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
        // UnopNot = "!" => ActionFn(23);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action23::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant17(__nt), __end));
        (1, 35)
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
        // VarWithType = Mut, Id, ":", Type => ActionFn(10);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant16(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant11(__symbols);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action10::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (4, 36)
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
        // VarWithType? = VarWithType => ActionFn(62);
        let __sym0 = __pop_Variant4(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action62::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant18(__nt), __end));
        (1, 37)
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
        // VarWithType? =  => ActionFn(63);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action63::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant18(__nt), __end));
        (0, 37)
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
        // __Program = Program => ActionFn(0);
        let __sym0 = __pop_Variant13(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action0::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant13(__nt), __end));
        (1, 38)
    }
}
pub use self::__parse__Stmt::StmtParser;
#[cfg_attr(rustfmt, rustfmt_skip)]
mod __intern_token {
    #![allow(unused_imports)]
    use crate::value_language::ast::*;
    use crate::value_language::typing::Type;
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
            ("^(\\->)", false),
            ("^(/)", false),
            ("^(:)", false),
            ("^(;)", false),
            ("^(=)", false),
            ("^(==)", false),
            ("^(bool)", false),
            ("^(false)", false),
            ("^(fn)", false),
            ("^(i32)", false),
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
    (_, __1, _): (usize, VarWithType, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, ParsedExpr, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __3, _): (usize, usize, usize),
) -> ParsedStmt
{
    astf.let_stmt(__0, __1, __2, __3)
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
    (_, __1, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, Vec<VarWithType>, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __3, _): (usize, Type, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __4, _): (usize, alloc::vec::Vec<ParsedStmt>, usize),
    (_, __5, _): (usize, ParsedExpr, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __6, _): (usize, usize, usize),
) -> ParsedStmt
{
    astf.function(__0, __1, __2, __3, __4, __5, __6)
}

#[allow(unused_variables)]
fn __action10<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, bool, usize),
    (_, __1, _): (usize, String, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __2, _): (usize, Type, usize),
) -> VarWithType
{
    (__0, __1, __2)
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
) -> Type
{
    Type::I32
}

#[allow(unused_variables)]
fn __action46<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> Type
{
    Type::Bool
}

#[allow(unused_variables)]
fn __action47<
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
fn __action48<
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
fn __action49<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, mut v, _): (usize, alloc::vec::Vec<VarWithType>, usize),
    (_, e, _): (usize, core::option::Option<VarWithType>, usize),
) -> Vec<VarWithType>
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
fn __action51<
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
fn __action52<
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
fn __action53<
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
fn __action54<
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
fn __action55<
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
fn __action56<
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
fn __action57<
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
fn __action58<
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
fn __action59<
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
fn __action60<
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
fn __action61<
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
fn __action62<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, VarWithType, usize),
) -> core::option::Option<VarWithType>
{
    Some(__0)
}

#[allow(unused_variables)]
fn __action63<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> core::option::Option<VarWithType>
{
    None
}

#[allow(unused_variables)]
fn __action64<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> alloc::vec::Vec<VarWithType>
{
    alloc::vec![]
}

#[allow(unused_variables)]
fn __action65<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<VarWithType>, usize),
) -> alloc::vec::Vec<VarWithType>
{
    v
}

#[allow(unused_variables)]
fn __action66<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, VarWithType, usize),
    (_, _, _): (usize, &'input str, usize),
) -> VarWithType
{
    __0
}

#[allow(unused_variables)]
fn __action67<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, VarWithType, usize),
) -> alloc::vec::Vec<VarWithType>
{
    alloc::vec![__0]
}

#[allow(unused_variables)]
fn __action68<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<VarWithType>, usize),
    (_, e, _): (usize, VarWithType, usize),
) -> alloc::vec::Vec<VarWithType>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
fn __action69<
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
fn __action70<
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
fn __action71<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
) -> bool
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action47(
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
fn __action72<
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
    let __temp0 = __action48(
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
fn __action73<
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
    let __temp0 = __action61(
        astf,
        input,
        __0,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action69(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action74<
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
    let __temp0 = __action61(
        astf,
        input,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action70(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action75<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, core::option::Option<ParsedExpr>, usize),
) -> Vec<ParsedExpr>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action59(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action50(
        astf,
        input,
        __temp0,
        __0,
    )
}

#[allow(unused_variables)]
fn __action76<
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
    let __temp0 = __action60(
        astf,
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action50(
        astf,
        input,
        __temp0,
        __1,
    )
}

#[allow(unused_variables)]
fn __action77<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, VarWithType, usize),
    __1: (usize, &'input str, usize),
) -> alloc::vec::Vec<VarWithType>
{
    let __start0 = __0.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action66(
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
fn __action78<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<VarWithType>, usize),
    __1: (usize, VarWithType, usize),
    __2: (usize, &'input str, usize),
) -> alloc::vec::Vec<VarWithType>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action66(
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
fn __action79<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, core::option::Option<VarWithType>, usize),
) -> Vec<VarWithType>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action64(
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
fn __action80<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<VarWithType>, usize),
    __1: (usize, core::option::Option<VarWithType>, usize),
) -> Vec<VarWithType>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action65(
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
    let __temp0 = __action52(
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
    let __temp0 = __action52(
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
    let __temp0 = __action52(
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
fn __action84<
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
    let __temp0 = __action52(
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
fn __action85<
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
    let __temp0 = __action52(
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
fn __action86<
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
    let __temp0 = __action52(
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
fn __action87<
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
    let __temp0 = __action52(
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
fn __action88<
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
    let __temp0 = __action52(
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
fn __action89<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, VarWithType, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, ParsedExpr, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, usize, usize),
) -> ParsedStmt
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action52(
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
fn __action90<
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
    let __temp0 = __action52(
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
fn __action91<
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
    let __temp0 = __action52(
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
fn __action92<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<VarWithType>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, Type, usize),
    __7: (usize, &'input str, usize),
    __8: (usize, alloc::vec::Vec<ParsedStmt>, usize),
    __9: (usize, ParsedExpr, usize),
    __10: (usize, &'input str, usize),
    __11: (usize, usize, usize),
) -> ParsedStmt
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action52(
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
    let __temp0 = __action52(
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
fn __action94<
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
    let __temp0 = __action52(
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
fn __action95<
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
    let __temp0 = __action52(
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
fn __action96<
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
    let __temp0 = __action52(
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
fn __action97<
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
    let __temp0 = __action52(
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
fn __action98<
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
    let __temp0 = __action52(
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
fn __action99<
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
    let __temp0 = __action52(
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
    let __temp0 = __action51(
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
    let __temp0 = __action51(
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
    let __temp0 = __action51(
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
    let __temp0 = __action51(
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
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action104<
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
    let __temp0 = __action51(
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
    let __temp0 = __action51(
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
fn __action106<
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
    let __temp0 = __action51(
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
    __1: (usize, &'input str, usize),
    __2: (usize, ParsedExpr, usize),
    __3: (usize, &'input str, usize),
    __4: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __4.2.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action51(
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
        __4,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action108<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, VarWithType, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, ParsedExpr, usize),
    __4: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __4.2.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action51(
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
fn __action109<
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
    let __temp0 = __action51(
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
        __3,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action110<
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
    let __temp0 = __action51(
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
    __3: (usize, Vec<VarWithType>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, Type, usize),
    __7: (usize, &'input str, usize),
    __8: (usize, alloc::vec::Vec<ParsedStmt>, usize),
    __9: (usize, ParsedExpr, usize),
    __10: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __10.2.clone();
    let __end0 = __10.2.clone();
    let __temp0 = __action51(
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
    let __temp0 = __action51(
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
    let __temp0 = __action51(
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
    let __temp0 = __action51(
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
    let __temp0 = __action51(
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
    let __temp0 = __action51(
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
    let __temp0 = __action51(
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
    let __temp0 = __action51(
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
    let __temp0 = __action57(
        astf,
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action75(
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
    let __temp0 = __action58(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action75(
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
    let __temp0 = __action57(
        astf,
        input,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action76(
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
    let __temp0 = __action58(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action76(
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
    __lookbehind: &usize,
    __lookahead: &usize,
) -> ParsedProgram
{
    let __start0 = __lookbehind.clone();
    let __end0 = __lookahead.clone();
    let __temp0 = __action53(
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
fn __action124<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<ParsedStmt>, usize),
) -> ParsedProgram
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action54(
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
fn __action125<
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
    let __temp0 = __action53(
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
fn __action126<
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
    let __temp0 = __action54(
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
fn __action127<
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
    let __temp0 = __action53(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action106(
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
fn __action128<
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
    let __temp0 = __action54(
        astf,
        input,
        __3,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action106(
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
fn __action129<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, Vec<VarWithType>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, Type, usize),
    __7: (usize, &'input str, usize),
    __8: (usize, ParsedExpr, usize),
    __9: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __7.2.clone();
    let __end0 = __8.0.clone();
    let __temp0 = __action53(
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
        __temp0,
        __8,
        __9,
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
    __3: (usize, Vec<VarWithType>, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, Type, usize),
    __7: (usize, &'input str, usize),
    __8: (usize, alloc::vec::Vec<ParsedStmt>, usize),
    __9: (usize, ParsedExpr, usize),
    __10: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __8.0.clone();
    let __end0 = __8.2.clone();
    let __temp0 = __action54(
        astf,
        input,
        __8,
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
        __temp0,
        __9,
        __10,
    )
}

#[allow(unused_variables)]
fn __action131<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, VarWithType, usize),
) -> Vec<VarWithType>
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action62(
        astf,
        input,
        __0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action79(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action132<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> Vec<VarWithType>
{
    let __start0 = __lookbehind.clone();
    let __end0 = __lookahead.clone();
    let __temp0 = __action63(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action79(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action133<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<VarWithType>, usize),
    __1: (usize, VarWithType, usize),
) -> Vec<VarWithType>
{
    let __start0 = __1.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action62(
        astf,
        input,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action80(
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
    __0: (usize, alloc::vec::Vec<VarWithType>, usize),
) -> Vec<VarWithType>
{
    let __start0 = __0.2.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action63(
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
        __temp0,
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

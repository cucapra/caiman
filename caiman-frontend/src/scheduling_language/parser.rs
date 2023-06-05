// auto-generated: "lalrpop 0.19.12"
// sha3: 25106a02d3bfc5b5813e1fb336da67f5465871917df22cb3e07f8cb13eb3dc36
use crate::scheduling_language::ast::*;
use crate::scheduling_language::ast_factory::ASTFactory;
use super::schedulable::{SubExpr, FullExpr};
#[allow(unused_extern_crates)]
extern crate lalrpop_util as __lalrpop_util;
#[allow(unused_imports)]
use self::__lalrpop_util::state_machine as __state_machine;
extern crate core;
extern crate alloc;

#[cfg_attr(rustfmt, rustfmt_skip)]
mod __parse__Program {
    #![allow(non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens, clippy::all)]

    use crate::scheduling_language::ast::*;
    use crate::scheduling_language::ast_factory::ASTFactory;
    use super::super::schedulable::{SubExpr, FullExpr};
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
        Variant1(SubExpr),
        Variant2(alloc::vec::Vec<SubExpr>),
        Variant3(usize),
        Variant4(FullExpr),
        Variant5(String),
        Variant6(ParsedProgram),
        Variant7(ParsedStmt),
        Variant8(alloc::vec::Vec<ParsedStmt>),
    }
    const __ACTION: &[i8] = &[
        // State 0
        0, 0, 0, 0, 0, 0, 0, 0, 8,
        // State 1
        4, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 2
        0, 0, 0, 0, 0, 0, 0, 0, 8,
        // State 3
        0, 0, 13, 14, 15, 16, 17, 0, 0,
        // State 4
        0, 0, 13, 14, 15, 16, 17, 0, 0,
        // State 5
        0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 6
        0, 0, 0, 0, 0, 0, 0, 0, -17,
        // State 7
        -10, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 8
        5, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 9
        0, 0, 0, 0, 0, 0, 0, 0, -18,
        // State 10
        0, 20, 0, 0, 0, 0, 0, 0, 0,
        // State 11
        -4, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 12
        0, -8, 0, 0, 0, 0, 0, 0, 0,
        // State 13
        -21, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 14
        -19, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 15
        -20, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 16
        0, -9, 0, 0, 0, 0, 0, 0, 0,
        // State 17
        0, 21, 0, 0, 0, 0, 0, 0, 0,
        // State 18
        -5, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 19
        0, 0, 0, 0, 0, 0, 0, 0, -13,
        // State 20
        0, 0, 0, 0, 0, 0, 0, 0, -14,
    ];
    fn __action(state: i8, integer: usize) -> i8 {
        __ACTION[(state as usize) * 9 + integer]
    }
    const __EOF_ACTION: &[i8] = &[
        // State 0
        -11,
        // State 1
        0,
        // State 2
        -12,
        // State 3
        0,
        // State 4
        0,
        // State 5
        -22,
        // State 6
        -17,
        // State 7
        0,
        // State 8
        0,
        // State 9
        -18,
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
        -13,
        // State 20
        -14,
    ];
    fn __goto(state: i8, nt: usize) -> i8 {
        match nt {
            2 => 8,
            5 => match state {
                4 => 17,
                _ => 10,
            },
            6 => 1,
            7 => 5,
            8 => match state {
                2 => 9,
                _ => 6,
            },
            10 => 2,
            11 => match state {
                4 => 18,
                _ => 11,
            },
            _ => 0,
        }
    }
    fn __expected_tokens(__state: i8) -> alloc::vec::Vec<alloc::string::String> {
        const __TERMINAL: &[&str] = &[
            r###"".""###,
            r###"";""###,
            r###""IfComplete""###,
            r###""IfFalse""###,
            r###""IfGuard""###,
            r###""IfTrue""###,
            r###""Prim""###,
            r###""Var""###,
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
            __action(state, 9 - 1)
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
            Token(3, _) if true => Some(0),
            Token(4, _) if true => Some(1),
            Token(5, _) if true => Some(2),
            Token(6, _) if true => Some(3),
            Token(7, _) if true => Some(4),
            Token(8, _) if true => Some(5),
            Token(9, _) if true => Some(6),
            Token(10, _) if true => Some(7),
            Token(1, _) if true => Some(8),
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
            0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 => match __token {
                Token(3, __tok0) | Token(4, __tok0) | Token(5, __tok0) | Token(6, __tok0) | Token(7, __tok0) | Token(8, __tok0) | Token(9, __tok0) | Token(10, __tok0) | Token(1, __tok0) if true => __Symbol::Variant0(__tok0),
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
                // __Program = Program => ActionFn(0);
                let __sym0 = __pop_Variant6(__symbols);
                let __start = __sym0.0.clone();
                let __end = __sym0.2.clone();
                let __nt = super::__action0::<>(astf, input, __sym0);
                return Some(Ok(__nt));
            }
            22 => {
                __reduce22(astf, input, __lookahead_start, __symbols, core::marker::PhantomData::<(&())>)
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
    fn __pop_Variant4<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, FullExpr, usize)
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
    ) -> (usize, ParsedProgram, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant6(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant7<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, ParsedStmt, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant7(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant5<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, String, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant5(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant1<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, SubExpr, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant1(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant8<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, alloc::vec::Vec<ParsedStmt>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant8(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant2<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, alloc::vec::Vec<SubExpr>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant2(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant3<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, usize, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant3(__v), __r)) => (__l, __v, __r),
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
        // ("." <SubExpr>) = ".", SubExpr => ActionFn(13);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action13::<>(astf, input, __sym0, __sym1);
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
        // ("." <SubExpr>)* =  => ActionFn(11);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action11::<>(astf, input, &__start, &__end);
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
        // ("." <SubExpr>)* = ("." <SubExpr>)+ => ActionFn(12);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action12::<>(astf, input, __sym0);
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
        // ("." <SubExpr>)+ = ".", SubExpr => ActionFn(21);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action21::<>(astf, input, __sym0, __sym1);
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
        // ("." <SubExpr>)+ = ("." <SubExpr>)+, ".", SubExpr => ActionFn(22);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant1(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action22::<>(astf, input, __sym0, __sym1, __sym2);
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
        // @L =  => ActionFn(14);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action14::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (0, 3)
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
        // @R =  => ActionFn(10);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action10::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
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
        // FullExpr = "IfComplete" => ActionFn(7);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action7::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (1, 5)
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
        // FullExpr = "Prim" => ActionFn(8);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action8::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (1, 5)
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
        // Id = r#"[a-zA-Z][a-zA-Z0-9_]*"# => ActionFn(9);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action9::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (1, 6)
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
        // Program =  => ActionFn(29);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action29::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (0, 7)
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
        // Program = Stmt+ => ActionFn(30);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action30::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (1, 7)
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
        // Stmt = Id, ".", FullExpr, ";" => ActionFn(27);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant4(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action27::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (4, 8)
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
        // Stmt = Id, ("." <SubExpr>)+, ".", FullExpr, ";" => ActionFn(28);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant4(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action28::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (5, 8)
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
        // Stmt* =  => ActionFn(15);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action15::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (0, 9)
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
        // Stmt* = Stmt+ => ActionFn(16);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action16::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (1, 9)
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
        // Stmt+ = Stmt => ActionFn(17);
        let __sym0 = __pop_Variant7(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action17::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
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
        // Stmt+ = Stmt+, Stmt => ActionFn(18);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action18::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (2, 10)
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
        // SubExpr = "IfGuard" => ActionFn(4);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action4::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
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
        // SubExpr = "IfTrue" => ActionFn(5);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action5::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (1, 11)
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
        // SubExpr = "IfFalse" => ActionFn(6);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action6::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (1, 11)
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
        // __Stmt = Stmt => ActionFn(1);
        let __sym0 = __pop_Variant7(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action1::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (1, 13)
    }
}
pub use self::__parse__Program::ProgramParser;

#[cfg_attr(rustfmt, rustfmt_skip)]
mod __parse__Stmt {
    #![allow(non_snake_case, non_camel_case_types, unused_mut, unused_variables, unused_imports, unused_parens, clippy::all)]

    use crate::scheduling_language::ast::*;
    use crate::scheduling_language::ast_factory::ASTFactory;
    use super::super::schedulable::{SubExpr, FullExpr};
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
        Variant1(SubExpr),
        Variant2(alloc::vec::Vec<SubExpr>),
        Variant3(usize),
        Variant4(FullExpr),
        Variant5(String),
        Variant6(ParsedProgram),
        Variant7(ParsedStmt),
        Variant8(alloc::vec::Vec<ParsedStmt>),
    }
    const __ACTION: &[i8] = &[
        // State 0
        0, 0, 0, 0, 0, 0, 0, 0, 6,
        // State 1
        3, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 2
        0, 0, 10, 11, 12, 13, 14, 0, 0,
        // State 3
        0, 0, 10, 11, 12, 13, 14, 0, 0,
        // State 4
        0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 5
        -10, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 6
        4, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 7
        0, 17, 0, 0, 0, 0, 0, 0, 0,
        // State 8
        -4, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 9
        0, -8, 0, 0, 0, 0, 0, 0, 0,
        // State 10
        -21, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 11
        -19, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 12
        -20, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 13
        0, -9, 0, 0, 0, 0, 0, 0, 0,
        // State 14
        0, 18, 0, 0, 0, 0, 0, 0, 0,
        // State 15
        -5, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 16
        0, 0, 0, 0, 0, 0, 0, 0, 0,
        // State 17
        0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    fn __action(state: i8, integer: usize) -> i8 {
        __ACTION[(state as usize) * 9 + integer]
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
        -23,
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
        -13,
        // State 17
        -14,
    ];
    fn __goto(state: i8, nt: usize) -> i8 {
        match nt {
            2 => 6,
            5 => match state {
                3 => 14,
                _ => 7,
            },
            6 => 1,
            8 => 4,
            11 => match state {
                3 => 15,
                _ => 8,
            },
            _ => 0,
        }
    }
    fn __expected_tokens(__state: i8) -> alloc::vec::Vec<alloc::string::String> {
        const __TERMINAL: &[&str] = &[
            r###"".""###,
            r###"";""###,
            r###""IfComplete""###,
            r###""IfFalse""###,
            r###""IfGuard""###,
            r###""IfTrue""###,
            r###""Prim""###,
            r###""Var""###,
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
            __action(state, 9 - 1)
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
            Token(3, _) if true => Some(0),
            Token(4, _) if true => Some(1),
            Token(5, _) if true => Some(2),
            Token(6, _) if true => Some(3),
            Token(7, _) if true => Some(4),
            Token(8, _) if true => Some(5),
            Token(9, _) if true => Some(6),
            Token(10, _) if true => Some(7),
            Token(1, _) if true => Some(8),
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
            0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 => match __token {
                Token(3, __tok0) | Token(4, __tok0) | Token(5, __tok0) | Token(6, __tok0) | Token(7, __tok0) | Token(8, __tok0) | Token(9, __tok0) | Token(10, __tok0) | Token(1, __tok0) if true => __Symbol::Variant0(__tok0),
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
                // __Stmt = Stmt => ActionFn(1);
                let __sym0 = __pop_Variant7(__symbols);
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
    fn __pop_Variant4<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, FullExpr, usize)
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
    ) -> (usize, ParsedProgram, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant6(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant7<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, ParsedStmt, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant7(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant5<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, String, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant5(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant1<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, SubExpr, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant1(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant8<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, alloc::vec::Vec<ParsedStmt>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant8(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant2<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, alloc::vec::Vec<SubExpr>, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant2(__v), __r)) => (__l, __v, __r),
            _ => __symbol_type_mismatch()
        }
    }
    fn __pop_Variant3<
      'input,
    >(
        __symbols: &mut alloc::vec::Vec<(usize,__Symbol<'input>,usize)>
    ) -> (usize, usize, usize)
     {
        match __symbols.pop() {
            Some((__l, __Symbol::Variant3(__v), __r)) => (__l, __v, __r),
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
        // ("." <SubExpr>) = ".", SubExpr => ActionFn(13);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action13::<>(astf, input, __sym0, __sym1);
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
        // ("." <SubExpr>)* =  => ActionFn(11);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action11::<>(astf, input, &__start, &__end);
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
        // ("." <SubExpr>)* = ("." <SubExpr>)+ => ActionFn(12);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action12::<>(astf, input, __sym0);
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
        // ("." <SubExpr>)+ = ".", SubExpr => ActionFn(21);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant1(__symbols);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action21::<>(astf, input, __sym0, __sym1);
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
        // ("." <SubExpr>)+ = ("." <SubExpr>)+, ".", SubExpr => ActionFn(22);
        assert!(__symbols.len() >= 3);
        let __sym2 = __pop_Variant1(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant2(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym2.2.clone();
        let __nt = super::__action22::<>(astf, input, __sym0, __sym1, __sym2);
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
        // @L =  => ActionFn(14);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action14::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
        (0, 3)
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
        // @R =  => ActionFn(10);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action10::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant3(__nt), __end));
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
        // FullExpr = "IfComplete" => ActionFn(7);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action7::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (1, 5)
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
        // FullExpr = "Prim" => ActionFn(8);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action8::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant4(__nt), __end));
        (1, 5)
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
        // Id = r#"[a-zA-Z][a-zA-Z0-9_]*"# => ActionFn(9);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action9::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant5(__nt), __end));
        (1, 6)
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
        // Program =  => ActionFn(29);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action29::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (0, 7)
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
        // Program = Stmt+ => ActionFn(30);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action30::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (1, 7)
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
        // Stmt = Id, ".", FullExpr, ";" => ActionFn(27);
        assert!(__symbols.len() >= 4);
        let __sym3 = __pop_Variant0(__symbols);
        let __sym2 = __pop_Variant4(__symbols);
        let __sym1 = __pop_Variant0(__symbols);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym3.2.clone();
        let __nt = super::__action27::<>(astf, input, __sym0, __sym1, __sym2, __sym3);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (4, 8)
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
        // Stmt = Id, ("." <SubExpr>)+, ".", FullExpr, ";" => ActionFn(28);
        assert!(__symbols.len() >= 5);
        let __sym4 = __pop_Variant0(__symbols);
        let __sym3 = __pop_Variant4(__symbols);
        let __sym2 = __pop_Variant0(__symbols);
        let __sym1 = __pop_Variant2(__symbols);
        let __sym0 = __pop_Variant5(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym4.2.clone();
        let __nt = super::__action28::<>(astf, input, __sym0, __sym1, __sym2, __sym3, __sym4);
        __symbols.push((__start, __Symbol::Variant7(__nt), __end));
        (5, 8)
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
        // Stmt* =  => ActionFn(15);
        let __start = __lookahead_start.cloned().or_else(|| __symbols.last().map(|s| s.2.clone())).unwrap_or_default();
        let __end = __start.clone();
        let __nt = super::__action15::<>(astf, input, &__start, &__end);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (0, 9)
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
        // Stmt* = Stmt+ => ActionFn(16);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action16::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (1, 9)
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
        // Stmt+ = Stmt => ActionFn(17);
        let __sym0 = __pop_Variant7(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action17::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
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
        // Stmt+ = Stmt+, Stmt => ActionFn(18);
        assert!(__symbols.len() >= 2);
        let __sym1 = __pop_Variant7(__symbols);
        let __sym0 = __pop_Variant8(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym1.2.clone();
        let __nt = super::__action18::<>(astf, input, __sym0, __sym1);
        __symbols.push((__start, __Symbol::Variant8(__nt), __end));
        (2, 10)
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
        // SubExpr = "IfGuard" => ActionFn(4);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action4::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
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
        // SubExpr = "IfTrue" => ActionFn(5);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action5::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (1, 11)
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
        // SubExpr = "IfFalse" => ActionFn(6);
        let __sym0 = __pop_Variant0(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action6::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant1(__nt), __end));
        (1, 11)
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
        // __Program = Program => ActionFn(0);
        let __sym0 = __pop_Variant6(__symbols);
        let __start = __sym0.0.clone();
        let __end = __sym0.2.clone();
        let __nt = super::__action0::<>(astf, input, __sym0);
        __symbols.push((__start, __Symbol::Variant6(__nt), __end));
        (1, 12)
    }
}
pub use self::__parse__Stmt::StmtParser;
#[cfg_attr(rustfmt, rustfmt_skip)]
mod __intern_token {
    #![allow(unused_imports)]
    use crate::scheduling_language::ast::*;
    use crate::scheduling_language::ast_factory::ASTFactory;
    use super::super::schedulable::{SubExpr, FullExpr};
    #[allow(unused_extern_crates)]
    extern crate lalrpop_util as __lalrpop_util;
    #[allow(unused_imports)]
    use self::__lalrpop_util::state_machine as __state_machine;
    extern crate core;
    extern crate alloc;
    pub fn new_builder() -> __lalrpop_util::lexer::MatcherBuilder {
        let __strs: &[(&str, bool)] = &[
            ("^(//[\0-\t\u{b}-\u{c}\u{e}-\u{10ffff}]*[\n\r]*)", true),
            ("^([A-Za-z][0-9A-Z_a-z]*)", false),
            ("^([\t-\r \u{85}\u{a0}\u{1680}\u{2000}-\u{200a}\u{2028}-\u{2029}\u{202f}\u{205f}\u{3000}]*)", true),
            ("^(\\.)", false),
            ("^(;)", false),
            ("^(IfComplete)", false),
            ("^(IfFalse)", false),
            ("^(IfGuard)", false),
            ("^(IfTrue)", false),
            ("^(Prim)", false),
            ("^(Var)", false),
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
    (_, __1, _): (usize, String, usize),
    (_, __2, _): (usize, alloc::vec::Vec<SubExpr>, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __3, _): (usize, FullExpr, usize),
    (_, _, _): (usize, &'input str, usize),
    (_, __4, _): (usize, usize, usize),
) -> ParsedStmt
{
    astf.expr(__0, __1, __2, __3, __4)
}

#[allow(unused_variables)]
fn __action4<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> SubExpr
{
    SubExpr::IfGuard
}

#[allow(unused_variables)]
fn __action5<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> SubExpr
{
    SubExpr::IfTrue
}

#[allow(unused_variables)]
fn __action6<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> SubExpr
{
    SubExpr::IfFalse
}

#[allow(unused_variables)]
fn __action7<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> FullExpr
{
    FullExpr::IfComplete
}

#[allow(unused_variables)]
fn __action8<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, &'input str, usize),
) -> FullExpr
{
    FullExpr::Primitive
}

#[allow(unused_variables)]
fn __action9<
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
fn __action10<
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
fn __action11<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __lookbehind: &usize,
    __lookahead: &usize,
) -> alloc::vec::Vec<SubExpr>
{
    alloc::vec![]
}

#[allow(unused_variables)]
fn __action12<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<SubExpr>, usize),
) -> alloc::vec::Vec<SubExpr>
{
    v
}

#[allow(unused_variables)]
fn __action13<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, _, _): (usize, &'input str, usize),
    (_, __0, _): (usize, SubExpr, usize),
) -> SubExpr
{
    __0
}

#[allow(unused_variables)]
fn __action14<
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
fn __action15<
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
fn __action16<
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
fn __action17<
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
fn __action18<
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
fn __action19<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, __0, _): (usize, SubExpr, usize),
) -> alloc::vec::Vec<SubExpr>
{
    alloc::vec![__0]
}

#[allow(unused_variables)]
fn __action20<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    (_, v, _): (usize, alloc::vec::Vec<SubExpr>, usize),
    (_, e, _): (usize, SubExpr, usize),
) -> alloc::vec::Vec<SubExpr>
{
    { let mut v = v; v.push(e); v }
}

#[allow(unused_variables)]
fn __action21<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, &'input str, usize),
    __1: (usize, SubExpr, usize),
) -> alloc::vec::Vec<SubExpr>
{
    let __start0 = __0.0.clone();
    let __end0 = __1.2.clone();
    let __temp0 = __action13(
        astf,
        input,
        __0,
        __1,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action19(
        astf,
        input,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action22<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<SubExpr>, usize),
    __1: (usize, &'input str, usize),
    __2: (usize, SubExpr, usize),
) -> alloc::vec::Vec<SubExpr>
{
    let __start0 = __1.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action13(
        astf,
        input,
        __1,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action20(
        astf,
        input,
        __0,
        __temp0,
    )
}

#[allow(unused_variables)]
fn __action23<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, usize, usize),
    __1: (usize, String, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, FullExpr, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, usize, usize),
) -> ParsedStmt
{
    let __start0 = __1.2.clone();
    let __end0 = __2.0.clone();
    let __temp0 = __action11(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action3(
        astf,
        input,
        __0,
        __1,
        __temp0,
        __2,
        __3,
        __4,
        __5,
    )
}

#[allow(unused_variables)]
fn __action24<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, usize, usize),
    __1: (usize, String, usize),
    __2: (usize, alloc::vec::Vec<SubExpr>, usize),
    __3: (usize, &'input str, usize),
    __4: (usize, FullExpr, usize),
    __5: (usize, &'input str, usize),
    __6: (usize, usize, usize),
) -> ParsedStmt
{
    let __start0 = __2.0.clone();
    let __end0 = __2.2.clone();
    let __temp0 = __action12(
        astf,
        input,
        __2,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action3(
        astf,
        input,
        __0,
        __1,
        __temp0,
        __3,
        __4,
        __5,
        __6,
    )
}

#[allow(unused_variables)]
fn __action25<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
    __1: (usize, &'input str, usize),
    __2: (usize, FullExpr, usize),
    __3: (usize, &'input str, usize),
    __4: (usize, usize, usize),
) -> ParsedStmt
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action14(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action23(
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
fn __action26<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
    __1: (usize, alloc::vec::Vec<SubExpr>, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, FullExpr, usize),
    __4: (usize, &'input str, usize),
    __5: (usize, usize, usize),
) -> ParsedStmt
{
    let __start0 = __0.0.clone();
    let __end0 = __0.0.clone();
    let __temp0 = __action14(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action24(
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
fn __action27<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
    __1: (usize, &'input str, usize),
    __2: (usize, FullExpr, usize),
    __3: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __3.2.clone();
    let __end0 = __3.2.clone();
    let __temp0 = __action10(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action25(
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
fn __action28<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, String, usize),
    __1: (usize, alloc::vec::Vec<SubExpr>, usize),
    __2: (usize, &'input str, usize),
    __3: (usize, FullExpr, usize),
    __4: (usize, &'input str, usize),
) -> ParsedStmt
{
    let __start0 = __4.2.clone();
    let __end0 = __4.2.clone();
    let __temp0 = __action10(
        astf,
        input,
        &__start0,
        &__end0,
    );
    let __temp0 = (__start0, __temp0, __end0);
    __action26(
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
fn __action29<
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
    let __temp0 = __action15(
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
fn __action30<
    'input,
>(
    astf: &ASTFactory,
    input: &'input str,
    __0: (usize, alloc::vec::Vec<ParsedStmt>, usize),
) -> ParsedProgram
{
    let __start0 = __0.0.clone();
    let __end0 = __0.2.clone();
    let __temp0 = __action16(
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

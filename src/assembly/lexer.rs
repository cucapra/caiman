use chumsky::{prelude::*, error::SimpleReason};
use core::fmt;
use std::convert::TryInto;

pub type Span = std::ops::Range<usize>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    // Symbols
    LPAREN,
    RPAREN,
    LBRACKET,
    RBRACKET,
    LCURLY,
    RCURLY,
    ARROW,
    DASH,
    COMMA,
    NEWLINE,
    ASSIGN,

    // Keywords
    TYPES,
    EXTERNAL_CPU,
    EXTERNAL_GPU,
    VALUE,
    SCHEDULE,
    TIMELINE,

    // Types
    I32,

    // Place
    LOCAL,
    CPU,
    GPU,

    // Tags

    NONE,
    OPERATION,
    INPUT,
    OUTPUT,
    FUNCTION_INPUT,
    FUNCTION_OUTPUT,
    HALT,

    // Operations
    PHI,
    RETURN,
    ALLOC,
    DO,

    // Vars
    Id(String),
    Number(usize),
    Var(String),
    TypeName(String),
    FnName(String),
    FunctionLoc(String, usize)
}

fn ident_map(ident: String, span : Span) -> 
Result<Token, Simple<char>> {
    match ident.as_str() {
        "types" => Ok(Token::TYPES),
        "external_cpu" => Ok(Token::EXTERNAL_CPU),
        "external_gpu" => Ok(Token::EXTERNAL_GPU),
        "value" => Ok(Token::VALUE),
        "schedule" => Ok(Token::SCHEDULE),
        "timeline" => Ok(Token::TIMELINE),
        "i32" => Ok(Token::I32),
        "local" => Ok(Token::LOCAL),
        "cpu" => Ok(Token::CPU),
        "gpu" => Ok(Token::GPU),
        "none" => Ok(Token::NONE),
        "operation" => Ok(Token::OPERATION),
        "input" => Ok(Token::INPUT),
        "output" => Ok(Token::OUTPUT),
        "function_input" => Ok(Token::FUNCTION_INPUT),
        "function_output" => Ok(Token::FUNCTION_OUTPUT),
        "halt" => Ok(Token::HALT),
        "phi" => Ok(Token::PHI),
        "return" => Ok(Token::RETURN),
        "alloc" => Ok(Token::ALLOC),
        "do" => Ok(Token::DO),
        _ => Err(Simple::custom(span,format!("Unknown keyword {}", ident)))
    }
}

fn symbol_map(s : String, span : Span) -> Result<Token, Simple<char>> {
    match s.as_str() {
        "(" => Ok(Token::LPAREN),
        ")" => Ok(Token::RPAREN),
        "[" => Ok(Token::LBRACKET),
        "]" => Ok(Token::RBRACKET),
        "{" => Ok(Token::LCURLY),
        "}" => Ok(Token::RCURLY),
        "->" => Ok(Token::ARROW),
        "-" => Ok(Token::DASH),
        "," => Ok(Token::COMMA),
        "=" => Ok(Token::ASSIGN),
        _ => Err(Simple::custom(span, format!("unknown symbol {}", s)))
    }
}

// Boring old lexer stuff
// https://github.com/zesterer/chumsky/blob/master/examples/nano_rust.rs is basis
pub fn lexer() -> impl Parser<char, Vec<(Token, Span)>, 
Error = Simple<char>> + Clone {
    let integer = text::int(10).map(
        |s : String| Token::Number(s.parse().unwrap()));

    let ident = text::ident::<_, Simple<char>>().try_map(ident_map);

    let symbols = "()[]{}->,=";
    let symbol = one_of::<_, _, Simple<char>>(symbols)
        .repeated()
        .collect::<String>()
        .try_map(symbol_map);

    let newline = text::newline().map(|_| Token::NEWLINE);

    let comment = just("//").then(take_until(just('\n'))).padded();

    let token = integer
        .or(ident)
        .or(symbol)
        .or(newline);

    let whitespace = just(" ").or(just("\t"));

    token
        .map_with_span(|tok, span| (tok, span))
        .padded_by(comment.repeated())
        .padded_by(whitespace.repeated())
        .repeated()
        .then_ignore(end())
}
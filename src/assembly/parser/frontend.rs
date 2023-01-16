use crate::assembly::lexer;
use crate::ir;
use chumsky::prelude::*;

pub fn parser() -> impl Parser<lexer::Token, 
ir::Program, Error = Simple<lexer::Token>> + Clone {
    use lexer::Token;

    let newline = just(Token::NEWLINE).or(just(Token::COMMENT));

    newline
        .clone()
        .repeated()
        .ignore_then(command_parser().separated_by(newline.clone().repeated().at_least(1)))
        .then_ignore(newline.repeated())
        .then_ignore(end())
}
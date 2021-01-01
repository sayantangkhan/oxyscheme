//! Module to succesively parse a stream of `Token`s into `Expression`s and then `Program`.

use crate::lexer::Token;
use crate::lexer::TokenWithPosition;
use std::iter::Peekable;

use crate::{lexer::LispNum, CompilerError};

#[derive(Debug, PartialEq)]
enum Datum {
    Boolean(bool),
    Number(LispNum),
    Character(char),
    String(String),
    Identifier(String),
    List(Vec<Datum>),
    DottedPair(Vec<Datum>, Box<Datum>),
    Quote(Box<Datum>),
    BackQuote(Box<Datum>),
    Comma(Box<Datum>),
    CommaAt(Box<Datum>),
    Vector(Vec<Datum>),
}

fn parse_into_datum<I>(token_stream: &mut Peekable<I>) -> Result<Datum, CompilerError>
where
    I: Iterator<Item = Result<TokenWithPosition, CompilerError>>,
{
    match token_stream.peek() {
        Some(Ok(TokenWithPosition {
            token,
            line,
            column,
        })) => match token {
            Token::Boolean(_) => parse_simple_datum(token_stream),
            Token::String(_) => parse_simple_datum(token_stream),
            Token::Character(_) => parse_simple_datum(token_stream),
            Token::Number(_) => parse_simple_datum(token_stream),
            Token::Identifier(_) => parse_simple_datum(token_stream),
            Token::Whitespace => {
                token_stream.next();
                parse_into_datum(token_stream)
            }
            Token::Comment => {
                token_stream.next();
                parse_into_datum(token_stream)
            }
            Token::Punctuator(p) if p == "#(" => parse_vector(token_stream),
            _ => Err(CompilerError::UnexpectedToken(*line, *column)),
        },

        Some(Err(_)) => Err(token_stream.next().unwrap().unwrap_err()),

        None => Err(CompilerError::TokenStreamEnded),
    }
}

fn parse_simple_datum<I>(token_stream: &mut Peekable<I>) -> Result<Datum, CompilerError>
where
    I: Iterator<Item = Result<TokenWithPosition, CompilerError>>,
{
    let TokenWithPosition { token, .. } = token_stream.next().unwrap()?;
    match token {
        Token::Boolean(b) => Ok(Datum::Boolean(b)),
        Token::String(s) => Ok(Datum::String(s)),
        Token::Character(c) => Ok(Datum::Character(c)),
        Token::Number(l) => Ok(Datum::Number(l)),
        Token::Identifier(i) => Ok(Datum::Identifier(i)),
        _ => unreachable!(),
    }
}

fn parse_vector<I>(token_stream: &mut Peekable<I>) -> Result<Datum, CompilerError>
where
    I: Iterator<Item = Result<TokenWithPosition, CompilerError>>,
{
    let mut vector = Vec::new();

    // Consuming the "#("
    token_stream.next();

    loop {
        match token_stream.peek() {
            Some(Ok(token_with_position)) => {
                let token = &token_with_position.token;
                match token {
                    Token::Punctuator(p) if p == ")" => {
                        token_stream.next();
                        break;
                    }
                    _ => {
                        let datum = parse_into_datum(token_stream)?;
                        vector.push(datum);
                    }
                }
            }

            Some(Err(_)) => {
                return Err(token_stream.next().unwrap().unwrap_err());
            }

            None => {
                return Err(CompilerError::MissingCloseParen);
            }
        }
    }

    Ok(Datum::Vector(vector))
}

#[cfg(test)]
mod test {
    use super::{parse_into_datum, Datum};
    use crate::{lexer::TokenWithPosition, CompilerError, Token};

    #[test]
    fn parse_simple_datum_test() {
        let vec_of_res: Vec<Result<TokenWithPosition, CompilerError>> =
            vec![Ok(TokenWithPosition {
                token: Token::Boolean(true),
                line: 0,
                column: 0,
            })];
        let mut token_stream = vec_of_res.into_iter().peekable();
        assert_eq!(
            parse_into_datum(&mut token_stream).unwrap(),
            Datum::Boolean(true)
        );
    }

    #[test]
    fn parse_vector_test() {
        let vec_of_res: Vec<Result<TokenWithPosition, CompilerError>> = vec![
            Ok(TokenWithPosition {
                token: Token::Punctuator(String::from("#(")),
                line: 0,
                column: 0,
            }),
            Ok(TokenWithPosition {
                token: Token::Punctuator(String::from("#(")),
                line: 0,
                column: 0,
            }),
            Ok(TokenWithPosition {
                token: Token::Boolean(true),
                line: 0,
                column: 0,
            }),
            Ok(TokenWithPosition {
                token: Token::Punctuator(String::from(")")),
                line: 0,
                column: 0,
            }),
            Ok(TokenWithPosition {
                token: Token::Punctuator(String::from(")")),
                line: 0,
                column: 0,
            }),
        ];
        let mut token_stream = vec_of_res.into_iter().peekable();
        assert_eq!(
            parse_into_datum(&mut token_stream).unwrap(),
            Datum::Vector(vec![Datum::Vector(vec![Datum::Boolean(true)])])
        );
    }
}

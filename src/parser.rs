//! Module to succesively parse a stream of `Token`s into `Datum`s which is then further modified
//! by the `ast-wrangler`.

use crate::lexer::Token;
use crate::lexer::TokenWithPosition;
use std::iter::Peekable;

use crate::{lexer::LispNum, CompilerError};

/// An enum representing `Datum`, i.e. the nodes of the abstract syntax tree
#[derive(Debug, PartialEq)]
pub enum Datum {
    /// Represents a boolean
    Boolean(bool),
    /// Represents a `LispNum`
    Number(LispNum),
    /// Represents a character
    Character(char),
    /// Represents a string
    String(String),
    /// Represents an identifier
    Identifier(String),
    /// Represents a list without a dot
    List(Vec<Datum>),
    /// Represents a `cons` block, with a `car` and `cdr`. The `car` is represented by a list of
    /// `Datum`, and the `cdr` is just a single `Datum`.
    DottedPair(Vec<Datum>, Box<Datum>),
    /// Represents a quoted `Datum`
    Quote(Box<Datum>),
    /// Represents a backquoted `Datum`
    Backquote(Box<Datum>),
    /// Represents a unquoted `Datum`
    Unquote(Box<Datum>),
    /// Represents a spliced unquoted `Datum`
    UnquoteSplice(Box<Datum>),
    /// Represents a vector
    Vector(Vec<Datum>),
}

/// Parses a single `Datum` from the token stream
pub fn parse_datum<I>(token_stream: &mut Peekable<I>) -> Result<Datum, CompilerError>
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
                parse_datum(token_stream)
            }
            Token::Comment => {
                token_stream.next();
                parse_datum(token_stream)
            }
            Token::Punctuator(p) if p == "(" => parse_list(token_stream),
            Token::Punctuator(p) if p == "#(" => parse_vector(token_stream),
            Token::Punctuator(p) if p == "'" => parse_abbrev(token_stream),
            Token::Punctuator(p) if p == "`" => parse_abbrev(token_stream),
            Token::Punctuator(p) if p == "," => parse_abbrev(token_stream),
            Token::Punctuator(p) if p == ",@" => parse_abbrev(token_stream),
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
                        let datum = parse_datum(token_stream)?;
                        vector.push(datum);
                    }
                }
            }

            Some(Err(_)) => {
                return Err(token_stream.next().unwrap().unwrap_err());
            }

            None => {
                // Figure out a way to include the line and column number of the error
                return Err(CompilerError::MissingCloseParen);
            }
        }
    }

    Ok(Datum::Vector(vector))
}

fn parse_abbrev<I>(token_stream: &mut Peekable<I>) -> Result<Datum, CompilerError>
where
    I: Iterator<Item = Result<TokenWithPosition, CompilerError>>,
{
    let TokenWithPosition { token, .. } = token_stream.next().unwrap()?;
    let datum = parse_datum(token_stream)?;
    if let Token::Punctuator(s) = token {
        match s.as_str() {
            "'" => Ok(Datum::Quote(Box::new(datum))),
            "`" => Ok(Datum::Backquote(Box::new(datum))),
            "," => Ok(Datum::Unquote(Box::new(datum))),
            ",@" => Ok(Datum::UnquoteSplice(Box::new(datum))),
            _ => unreachable!(),
        }
    } else {
        unreachable!()
    }
}

fn parse_list<I>(token_stream: &mut Peekable<I>) -> Result<Datum, CompilerError>
where
    I: Iterator<Item = Result<TokenWithPosition, CompilerError>>,
{
    let mut car: Vec<Datum> = Vec::new();

    // Consuming the "("
    token_stream.next();

    loop {
        match token_stream.peek() {
            Some(Ok(token_with_position)) => {
                let token = &token_with_position.token;
                match token {
                    Token::Punctuator(p) if p == ")" => {
                        token_stream.next();
                        return Ok(Datum::List(car));
                    }
                    Token::Punctuator(p) if p == "." => {
                        return parse_cdr(token_stream, car);
                    }
                    _ => {
                        let next_datum = parse_datum(token_stream)?;
                        car.push(next_datum);
                    }
                }
            }
            Some(Err(_)) => {
                return Err(token_stream.next().unwrap().unwrap_err());
            }
            None => {
                // Figure out a way to include the line and column number of the error
                return Err(CompilerError::MissingCloseParen);
            }
        }
    }
}

fn parse_cdr<I>(token_stream: &mut Peekable<I>, car: Vec<Datum>) -> Result<Datum, CompilerError>
where
    I: Iterator<Item = Result<TokenWithPosition, CompilerError>>,
{
    token_stream.next();
    let cdr = parse_datum(token_stream)?;
    match token_stream.next() {
        Some(Ok(TokenWithPosition {
            token: Token::Punctuator(p),
            ..
        })) if p == ")" => Ok(Datum::DottedPair(car, Box::new(cdr))),
        _ => {
            // Figure out a way to include the line and column number of the error
            Err(CompilerError::MissingCloseParen)
        }
    }
}

#[cfg(test)]
mod test {
    use super::{parse_datum, Datum};
    use crate::{
        lexer::{Token, TokenWithPosition},
        CompilerError,
    };

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
            parse_datum(&mut token_stream).unwrap(),
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
            parse_datum(&mut token_stream).unwrap(),
            Datum::Vector(vec![Datum::Vector(vec![Datum::Boolean(true)])])
        );
    }

    #[test]
    fn parse_list_test() {
        let vec_of_res: Vec<Result<TokenWithPosition, CompilerError>> = vec![
            Ok(TokenWithPosition {
                token: Token::Punctuator(String::from("(")),
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
            parse_datum(&mut token_stream).unwrap(),
            Datum::List(vec![Datum::Vector(vec![Datum::Boolean(true)])])
        );
    }

    #[test]
    fn parse_dotted_pair_test() {
        let vec_of_res: Vec<Result<TokenWithPosition, CompilerError>> = vec![
            Ok(TokenWithPosition {
                token: Token::Punctuator(String::from("(")),
                line: 0,
                column: 0,
            }),
            Ok(TokenWithPosition {
                token: Token::Identifier(String::from("a")),
                line: 0,
                column: 0,
            }),
            Ok(TokenWithPosition {
                token: Token::Punctuator(String::from(".")),
                line: 0,
                column: 0,
            }),
            Ok(TokenWithPosition {
                token: Token::Identifier(String::from("a")),
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
        let car = vec![Datum::Identifier(String::from("a"))];
        let cdr = Box::new(Datum::Identifier(String::from("a")));
        let pair = Datum::DottedPair(car, cdr);
        assert_eq!(parse_datum(&mut token_stream).unwrap(), pair);
    }

    #[test]
    fn parse_abbrev_test() {
        let vec_of_res: Vec<Result<TokenWithPosition, CompilerError>> = vec![
            Ok(TokenWithPosition {
                token: Token::Punctuator("'".to_string()),
                line: 0,
                column: 0,
            }),
            Ok(TokenWithPosition {
                token: Token::Boolean(true),
                line: 0,
                column: 1,
            }),
        ];
        let mut token_stream = vec_of_res.into_iter().peekable();
        assert_eq!(
            parse_datum(&mut token_stream).unwrap(),
            Datum::Quote(Box::new(Datum::Boolean(true)))
        );
    }
}

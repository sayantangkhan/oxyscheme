//! Module to lex the input stream and return a stream of tokens
use nom::{
    branch::alt,
    bytes::complete::{escaped_transform, is_not, tag},
    character::complete::{anychar, digit0, digit1, one_of, satisfy},
    combinator::{map, opt, peek, recognize, value},
    error::ErrorKind,
    multi::many0,
    sequence::tuple,
    IResult,
};

use nom::error::Error as NomErrorStruct;
use nom::Err::Error as NomErrorEnum;

/// Type alias for the common return type for the lexers
type LexResult<'a> = IResult<&'a str, Token<'a>, NomErrorStruct<&'a str>>;

/// Terminal token types for the lexer
///
/// The variants of `Token` wrap around the corresponding Rust types in the case of `String`,
/// `Character`, and `Boolean`. `Number` wraps around `LispNum`, which can either be an `f32`
/// or an `i32`. `Identifier` and `Punctuator` wrap around slices from the input, to avoid
/// unnecessary heap copying and heap allocation. In particular, this means that the token cannot
/// be dropped before the input string.
#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    /// Wraps a string
    String(String),
    /// Wraps a character
    Character(char),
    /// Wraps a boolean
    Boolean(bool),
    /// Wraps a number
    Number(LispNum),
    /// Wraps an identifier in the form of a string slice
    Identifier(&'a str),
    /// Wraps a punctuator in the form of a string slice
    Punctuator(&'a str),
}

/// Internal representation of numeric types in Scheme
///
/// `LispNum` is an enum wrapping around Rust's `i32` and `f32` types; the only two numeric types
/// we are currently implementing for the Scheme compiler target. More variants will be added in
/// the future.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LispNum {
    /// Wraps an `i32`
    Integer(i32),
    /// Wraps an `f32`
    Float(f32),
}

/// Iterator of `Token`s that maintains state
#[derive(Debug)]
pub struct TokenStream<'a> {
    /// The leftover input. May become a private field in the future.
    pub input_slice: &'a str,
}

impl<'a> TokenStream<'a> {
    /// Creates a new `TokenStream` from a string slice
    pub fn new(input: &'a str) -> TokenStream<'a> {
        TokenStream { input_slice: input }
    }

    /// Checks whether any more the leftover input is whitespace
    pub fn is_empty(&self) -> bool {
        match self.input_slice.trim() {
            "" => true,
            _ => false,
        }
    }
}

impl<'a> Iterator for TokenStream<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut parser = alt((
            parse_string,
            parse_boolean,
            parse_character,
            parse_identifier,
            parse_number,
            parse_punctuator,
        ));
        if let Ok((leftover, parsed)) = parser(self.input_slice) {
            self.input_slice = leftover;
            Some(parsed)
        } else {
            None
        }
    }
}

fn parse_string(input: &str) -> LexResult {
    let (input, _) = tag("\"")(input)?;
    let (leftover, parsed) = escaped_transform(
        is_not("\\\""),
        '\\',
        alt((
            value("\\", tag("\\")),
            value("\"", tag("\"")),
            value("\n", tag("n")),
        )),
    )(input)?;
    let (input, _) = tag("\"")(leftover)?;
    Ok((input, Token::String(parsed)))
}

fn parse_boolean(input: &str) -> LexResult {
    let (input, _) = tag("#")(input)?;
    let (leftover, parsed) = one_of("tf")(input)?;
    match parsed {
        't' => Ok((leftover, Token::Boolean(true))),
        'f' => Ok((leftover, Token::Boolean(false))),
        _ => Err(NomErrorEnum(NomErrorStruct::new(input, ErrorKind::OneOf))),
    }
}

fn peek_delimiter(input: &str) -> IResult<&str, ()> {
    let whitespace = one_of(" \n\t");
    let delimiter = alt((whitespace, one_of("()\";")));
    map(peek(delimiter), |_: char| ())(input)
}

fn parse_character(input: &str) -> LexResult {
    let (input, _) = tag("#\\")(input)?;
    let space_parser = map(tag("space"), |_| ' ');
    let newline_parser = map(tag("newline"), |_| '\n');
    let (leftover, parsed) = alt((space_parser, newline_parser, anychar))(input)?;
    peek_delimiter(leftover)?;
    Ok((leftover, Token::Character(parsed)))
}

fn non_peculiar(input: &str) -> IResult<&str, &str> {
    let special_initial = one_of("!$%&*/:<=>?^_~");
    let letter = satisfy(|c| c.is_alphabetic());
    let initial = alt((letter, special_initial));
    let digit = satisfy(|c| c.is_numeric());
    let special_subsequent = one_of("+-.@");
    let subsequent = alt((initial, digit, special_subsequent));

    // The repeated code is to get around the compiler's move semantics.
    let special_initial = one_of("!$%&*/:<=>?^_~");
    let letter = satisfy(|c| c.is_alphabetic());
    let initial = alt((letter, special_initial));

    recognize(tuple((initial, many0(subsequent))))(input)
}

fn parse_identifier(input: &str) -> LexResult {
    let peculiar_identifier = alt((tag("+"), tag("-"), tag("...")));
    let (leftover, parsed) = alt((non_peculiar, peculiar_identifier))(input)?;
    peek_delimiter(leftover)?;
    Ok((leftover, Token::Identifier(parsed)))
}

fn parse_number(input: &str) -> LexResult {
    let integer_parser = tuple((opt(one_of("+-")), digit1));
    let float_parser =
        tuple::<_, _, (_, ErrorKind), _>((opt(one_of("+-")), digit0, tag("."), digit1));
    // Note that one needs to annotate the tuple function in this case because the compilier
    // is unable to infer the return type.
    if let Ok((l, p)) = recognize(float_parser)(input) {
        if let Ok(num) = p.parse() {
            Ok((l, Token::Number(LispNum::Float(num))))
        } else {
            Err(NomErrorEnum(NomErrorStruct::new(l, ErrorKind::TooLarge)))
        }
    } else {
        let (l, p) = recognize(integer_parser)(input)?;
        if let Ok(num) = p.parse() {
            Ok((l, Token::Number(LispNum::Integer(num))))
        } else {
            Err(NomErrorEnum(NomErrorStruct::new(l, ErrorKind::TooLarge)))
        }
    }
}

fn parse_punctuator(input: &str) -> LexResult {
    alt((
        tag("("),
        tag(")"),
        tag("#("),
        tag("'"),
        tag("`"),
        tag(",@"),
        tag(","),
        tag("."),
    ))(input)
    .map(|(l, p)| (l, Token::Punctuator(p)))
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn parse_string_test() {
        assert_eq!(
            parse_string(r#""string""#),
            Ok(("", Token::String(String::from("string"))))
        );
        assert_eq!(
            parse_string(r#""st\"ring""#),
            Ok(("", Token::String(String::from("st\"ring"))))
        );
        assert_eq!(
            parse_string(r#""fail"#),
            Err(NomErrorEnum(NomErrorStruct::new("", ErrorKind::Tag)))
        );
        assert_eq!(
            parse_string(r#""new\nline""#),
            Ok(("", Token::String(String::from("new\nline"))))
        );
        assert_eq!(
            parse_string(r#"blah"string""#),
            Err(NomErrorEnum(NomErrorStruct::new(
                "blah\"string\"",
                ErrorKind::Tag
            )))
        );
    }

    #[test]
    fn parse_boolean_test() {
        assert_eq!(parse_boolean("#t"), Ok(("", Token::Boolean(true))));
        assert_eq!(parse_boolean("#f"), Ok(("", Token::Boolean(false))));
        assert_eq!(
            parse_boolean("#m"),
            Err(NomErrorEnum(NomErrorStruct::new("m", ErrorKind::OneOf)))
        );
    }

    #[test]
    fn parse_character_test() {
        assert_eq!(parse_character("#\\n\n"), Ok(("\n", Token::Character('n'))));
        assert_eq!(
            parse_character("#\\space\n"),
            Ok(("\n", Token::Character(' ')))
        );
        assert_eq!(
            parse_character("#\\newline\n"),
            Ok(("\n", Token::Character('\n')))
        );
    }

    #[test]
    fn non_peculiar_identifier_test() {
        assert_eq!(non_peculiar("a"), Ok(("", "a")));
        assert_eq!(non_peculiar("a+"), Ok(("", "a+")));
        assert_eq!(non_peculiar("&a+"), Ok(("", "&a+")));
        assert_eq!(
            non_peculiar("+&a+"),
            Err(NomErrorEnum(NomErrorStruct::new("+&a+", ErrorKind::OneOf)))
        );
    }

    #[test]
    fn parse_identifier_test() {
        assert_eq!(
            parse_identifier("...\n"),
            Ok(("\n", Token::Identifier("...")))
        );
        assert_eq!(
            parse_identifier("var\n"),
            Ok(("\n", Token::Identifier("var")))
        );
        assert_eq!(
            parse_identifier("var "),
            Ok((" ", Token::Identifier("var")))
        );
        assert_eq!(
            parse_identifier("var)"),
            Ok((")", Token::Identifier("var")))
        );
        assert_eq!(
            parse_identifier("var;"),
            Ok((";", Token::Identifier("var")))
        );
        assert_eq!(
            parse_identifier("var\""),
            Ok(("\"", Token::Identifier("var")))
        );
        assert_eq!(
            parse_identifier("he++o "),
            Ok((" ", Token::Identifier("he++o")))
        );
        assert_eq!(
            parse_identifier("hel.o "),
            Ok((" ", Token::Identifier("hel.o")))
        );
        assert_eq!(
            parse_identifier("..."),
            Err(NomErrorEnum(NomErrorStruct::new("", ErrorKind::OneOf)))
        );
        assert_eq!(
            parse_identifier("asdf,"),
            Err(NomErrorEnum(NomErrorStruct::new(",", ErrorKind::OneOf)))
        );
    }

    #[test]
    fn parse_number_test() {
        assert_eq!(
            parse_number("+3.14;"),
            Ok((";", Token::Number(LispNum::Float(3.14))))
        );
        assert_eq!(
            parse_number("-3.14;"),
            Ok((";", Token::Number(LispNum::Float(-3.14))))
        );
        assert_eq!(
            parse_number("3.14;"),
            Ok((";", Token::Number(LispNum::Float(3.14))))
        );
        assert_eq!(
            parse_number(".14;"),
            Ok((";", Token::Number(LispNum::Float(0.14))))
        );
        assert_eq!(
            parse_number("1;"),
            Ok((";", Token::Number(LispNum::Integer(1))))
        );
        assert_eq!(
            parse_number("-1;"),
            Ok((";", Token::Number(LispNum::Integer(-1))))
        );
        assert_eq!(
            parse_number("-1;"),
            Ok((";", Token::Number(LispNum::Integer(-1))))
        );
        assert_eq!(
            parse_number("4294967296;"),
            Err(NomErrorEnum(NomErrorStruct::new(";", ErrorKind::TooLarge)))
        );
    }

    #[test]
    fn parse_punctuator_test() {
        assert_eq!(parse_punctuator(",3"), Ok(("3", Token::Punctuator(","))));
        assert_eq!(parse_punctuator(",@"), Ok(("", Token::Punctuator(",@"))));
    }
}
use nom::branch::alt;
use nom::bytes::complete::{escaped_transform, is_not, tag};
use nom::character::complete::{anychar, one_of};
use nom::error::ErrorKind;
use nom::IResult;
use nom::{
    character::complete::satisfy,
    combinator::{map, peek, value},
    multi::many0,
};

enum Token {
    String(String),
    Character(char),
    Boolean(bool),
    Number(LispNum),
    Identifier(String),
    Punctuator(String),
}

enum LispNum {
    Integer(i32),
    Float(f32),
}

fn parse_string(input: &str) -> IResult<&str, String> {
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
    Ok((input, parsed))
}

fn parse_boolean(input: &str) -> IResult<&str, bool> {
    let (input, _) = tag("#")(input)?;
    let (leftover, parsed) = one_of("tf")(input)?;
    match parsed {
        't' => Ok((leftover, true)),
        'f' => Ok((leftover, false)),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            ErrorKind::OneOf,
        ))),
    }
}

fn peek_delimiter(input: &str) -> IResult<&str, ()> {
    let whitespace = one_of(" \n\t");
    let delimiter = alt((whitespace, one_of("()\";")));
    map(peek(delimiter), |_: char| ())(input)
}

fn parse_character(input: &str) -> IResult<&str, char> {
    let (input, _) = tag("#\\")(input)?;
    let space_parser = map(tag("space"), |_| ' ');
    let newline_parser = map(tag("newline"), |_| '\n');
    let (leftover, parsed) = alt((space_parser, newline_parser, anychar))(input)?;
    peek_delimiter(leftover)?;
    Ok((leftover, parsed))
}

fn non_peculiar(input: &str) -> IResult<&str, String> {
    let special_initial = one_of("!$%&*/:<=>?^_~");
    let letter = satisfy(|c| c.is_alphabetic());
    let mut initial = alt((letter, special_initial));
    let (leftover, initial_parsed_char) = initial(input)?;

    let digit = satisfy(|c| c.is_numeric());
    let special_subsequent = one_of("+-.@");
    let (leftover, mut vec_subsequent) =
        many0(alt((initial, digit, special_subsequent)))(leftover)?;

    vec_subsequent.insert(0, initial_parsed_char);

    Ok((leftover, vec_subsequent.iter().collect()))
}

fn parse_identifier(input: &str) -> IResult<&str, String> {
    let peculiar_identifier = map(alt((tag("+"), tag("-"), tag("..."))), |s: &str| {
        String::from(s)
    });
    let (leftover, parsed) = alt((non_peculiar, peculiar_identifier))(input)?;
    peek_delimiter(leftover)?;
    Ok((leftover, parsed))
}

#[test]
fn parse_string_test() {
    assert_eq!(parse_string("\"string\""), Ok(("", String::from("string"))));
    assert_eq!(
        parse_string("\"st\\\"ring\""),
        Ok(("", String::from("st\"ring")))
    );
    assert_eq!(
        parse_string("\"fail"),
        Err(nom::Err::Error(nom::error::Error::new("", ErrorKind::Tag)))
    );
    assert_eq!(
        parse_string("\"new\\nline\""),
        Ok(("", String::from("new\nline")))
    );
    assert_eq!(
        parse_string("blah\"string\""),
        Err(nom::Err::Error(nom::error::Error::new(
            "blah\"string\"",
            ErrorKind::Tag
        )))
    );
}

#[test]
fn parse_boolean_test() {
    assert_eq!(parse_boolean("#t"), Ok(("", true)));
    assert_eq!(parse_boolean("#f"), Ok(("", false)));
    assert_eq!(
        parse_boolean("#m"),
        Err(nom::Err::Error(nom::error::Error::new(
            "m",
            ErrorKind::OneOf
        )))
    );
}

#[test]
fn parse_character_test() {
    assert_eq!(parse_character("#\\n\n"), Ok(("\n", 'n')));
    assert_eq!(parse_character("#\\space\n"), Ok(("\n", ' ')));
    assert_eq!(parse_character("#\\newline\n"), Ok(("\n", '\n')));
}

#[test]
fn non_peculiar_identifier_test() {
    assert_eq!(non_peculiar("a"), Ok(("", String::from("a"))));
    assert_eq!(non_peculiar("a+"), Ok(("", String::from("a+"))));
    assert_eq!(non_peculiar("&a+"), Ok(("", String::from("&a+"))));
    assert_eq!(
        non_peculiar("+&a+"),
        Err(nom::Err::Error(nom::error::Error::new(
            "+&a+",
            ErrorKind::OneOf
        )))
    );
}

#[test]
fn parse_identifier_test() {
    assert_eq!(parse_identifier("...\n"), Ok(("\n", String::from("..."))));
    assert_eq!(parse_identifier("var\n"), Ok(("\n", String::from("var"))));
    assert_eq!(
        parse_identifier("..."),
        Err(nom::Err::Error(nom::error::Error::new(
            "",
            ErrorKind::OneOf
        )))
    );
    assert_eq!(
        parse_identifier("asdf,"),
        Err(nom::Err::Error(nom::error::Error::new(
            ",",
            ErrorKind::OneOf
        )))
    );
}

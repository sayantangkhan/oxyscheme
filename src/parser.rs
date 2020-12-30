//! Module to succesively parse a stream of `Token`s into `Expression`s and then `Program`.

use crate::lexer::LispNum;

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

// struct DatumStream {
//     token_stream: TokenStream,
// }

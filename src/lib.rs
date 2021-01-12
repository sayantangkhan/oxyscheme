//! Library for lexing and parsing a reasonable subset of
//! [R5RS](https://schemers.org/Documents/Standards/R5RS/r5rs.pdf) Scheme.
//!
//! ## Usage
//! TO ADD

#![warn(missing_docs, unused_variables, rust_2018_idioms)]

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod reader;

use thiserror::Error;

/// The toplevel error type for the crate
#[derive(Error, Debug)]
pub enum CompilerError {
    /// Indicates a lexing error
    ///
    /// `LexError` wraps around a `String` and a `usize`. The first `usize` is the line number in the input,
    /// the second `usize` is the column number, and the `String` is a copy of the leftover unlexed input from the line.
    #[error("Lex error at line {1}, column {2}, near \"{0}\" while lexing input")]
    LexError(String, usize, usize),

    /// Error variant handling the token stream ending too early
    #[error("Token stream ended unexpectedly")]
    TokenStreamEnded,

    /// Error variant handling unexpected tokens
    #[error("Unexpected token encountered at line {0}, column {1} while parsing input")]
    UnexpectedToken(usize, usize),

    /// Error variant handling unclosed lists or vectors
    #[error("Missing close paren at unknown position")]
    MissingCloseParen,

    /// Indicates an IO error
    ///
    /// Usually happens if the source files cannot be opened
    #[error("I/O error")]
    IOError(#[from] std::io::Error),
}

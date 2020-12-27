//! Library for lexing and parsing a reasonable subset of
//! [R5RS](https://schemers.org/Documents/Standards/R5RS/r5rs.pdf) Scheme.
//!
//! ## Usage
//! TO ADD

#![warn(missing_docs, unused_variables)]

pub mod lexer;
mod parser;

use thiserror::Error;

/// The toplevel error type for the crate
#[derive(Error, Debug)]
pub enum CompilerError {
    /// Indicates a lexing error
    ///
    /// `LexError` wraps around a `String` and a `usize`. The `usize` is the line number in the input,
    /// and the `String` is a copy of the leftover unlexed input from the line.
    #[error("Error at line {1}, near \"{0}\" while lexing input")]
    LexError(String, usize),
}

// pub fn create_token_stream

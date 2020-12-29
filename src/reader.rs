//! Handles reading files, and annotating tokens with line and column numbers
use crate::lexer::*;

/// Wrapper around `Token` that keeps track of line and column
#[derive(Debug)]
pub struct TokenWithPosition {
    token: Token,
    line: usize,
    column: usize,
}

/// Iterator of `Token`s that maintains state
#[derive(Debug)]
pub struct TokenStream<'a> {
    /// The leftover input. May become a private field in the future.
    pub input_slice: &'a str,
    /// Line number in input
    pub line_number: usize,
    /// Cursor position in input_slice
    pub cursor_position: usize,
}

impl<'a> TokenStream<'a> {
    /// Creates a new `TokenStream` from a string slice
    pub fn new(input: &'a str, line_number: usize) -> TokenStream<'a> {
        TokenStream {
            input_slice: input,
            line_number,
            cursor_position: 0,
        }
    }

    /// Checks whether any more input left
    pub fn is_empty(&self) -> bool {
        match self.input_slice {
            "" => true,
            _ => false,
        }
    }
}

impl<'a> Iterator for TokenStream<'a> {
    type Item = TokenWithPosition;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok((leftover, parsed)) = lex_input(self.input_slice) {
            let len_leftover = self.input_slice.len();
            self.input_slice = leftover;
            let token_with_position = TokenWithPosition {
                token: parsed,
                line: self.line_number,
                column: self.cursor_position,
            };
            self.cursor_position += len_leftover - self.input_slice.len();
            Some(token_with_position)
        } else {
            None
        }
    }
}

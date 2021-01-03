//! Handles reading files, and annotating tokens with line and column numbers
use crate::lexer::*;
use crate::parser::{parse_datum, Datum};
use crate::*;
use anyhow::Result;
use std::{
    fs::File,
    io::{BufRead, BufReader, Lines},
    iter::{Enumerate, Peekable},
    path::PathBuf,
};

/// `FileLexer` can be turned into an iterator of `Ok(TokenWithPosition)` and `Err(_)`
///
/// An instance of `FileLexer` can be created using the `FileLexer::new` method, which
/// takes a `&str` input for the filename, and returns `Result<FileLexer, CompilerError>`,
/// the `Err` case showing up when the given file cannot be opened. `FileLexer` implements
/// the `IntoIterator` trait, which means one can get a list of `TokenWithPosition`s using
/// a `for` loop over a `FileLexer`. More specifically, the associated iterator `Item`
/// is `Result<TokenWithPosition, CompilerError>`. On the first instance of encountering
/// a lexing error, the iterator outputs the corresponding error, and then stops. The idiomatic
/// way of turning a `FileLexer` into `Result<Vec<TokenWithPosition>, CompilerError>` is the
/// following.
///
/// ```
/// # use oxyscheme::CompilerError;
/// # use oxyscheme::reader::FileLexer;
/// # use oxyscheme::lexer::TokenWithPosition;
/// # use std::path::Path;
/// # let filename = &Path::new(env!("CARGO_MANIFEST_DIR"))
/// #                .join("inputs/hello-world.scm")
/// #                .into_os_string()
/// #                .into_string()
/// #                .unwrap();
/// let file_lexer = FileLexer::new(filename).unwrap();
/// let vec_of_tokens_res: Result<Vec<TokenWithPosition>, CompilerError> = file_lexer.into_iter().collect();
/// ```
pub struct FileLexer {
    file: File,
}

impl FileLexer {
    /// Creates a `FileLexer` from a filename. May return `Err` if the file cannot be opened.
    pub fn new(filename: &str) -> Result<Self, CompilerError> {
        Ok(FileLexer {
            file: File::open(PathBuf::from(filename))?,
        })
    }
}

impl IntoIterator for FileLexer {
    type Item = Result<TokenWithPosition, CompilerError>;
    type IntoIter = FileLexerIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let line_enumerator = BufReader::new(self.file).lines().enumerate();
        let input_string = String::from("");
        FileLexerIntoIter {
            line_enumerator,
            input_string,
            cursor_position: 0,
            line_number: 0,
            encountered_error: false,
        }
    }
}

/// The associated Iterator type for FileLexer
pub struct FileLexerIntoIter {
    line_enumerator: Enumerate<Lines<BufReader<File>>>,
    input_string: String,
    cursor_position: usize,
    line_number: usize,
    encountered_error: bool,
}

impl Iterator for FileLexerIntoIter {
    type Item = Result<TokenWithPosition, CompilerError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.encountered_error {
            return None;
        }

        while self.input_string.len() <= self.cursor_position {
            if let Some((index, line_res)) = self.line_enumerator.next() {
                match line_res {
                    Ok(line) => {
                        self.input_string = line;
                        self.cursor_position = 0;
                        self.line_number = index + 1;
                    }
                    Err(e) => {
                        self.encountered_error = true;
                        return Some(Err(CompilerError::IOError(e)));
                    }
                }
            } else {
                return None;
            }
        }

        match lex_input(&self.input_string[self.cursor_position..]) {
            Ok((leftover, parsed)) => {
                let token_with_position = TokenWithPosition {
                    token: parsed,
                    line: self.line_number,
                    column: self.cursor_position,
                };
                self.cursor_position = self.input_string.len() - leftover.len();

                Some(Ok(token_with_position))
            }
            Err(_) => {
                self.encountered_error = true;
                Some(Err(CompilerError::LexError(
                    String::from(&self.input_string[self.cursor_position..]),
                    self.line_number,
                    self.cursor_position,
                )))
            }
        }
    }
}

/// Iterator adapter that transforms a `TokenStream` to a stream of `Datum`
///
/// An instance of `DatumStream` can be created by first creating a `TokenStream`,
/// and then turning that into an iterator, which is then wrapped in a `DatumStream`.
///
/// ```
/// # use oxyscheme::CompilerError;
/// # use oxyscheme::reader::{FileLexer, DatumIterator};
/// # use oxyscheme::parser::Datum;
/// # use oxyscheme::lexer::TokenWithPosition;
/// # use std::path::Path;
/// # let filename = &Path::new(env!("CARGO_MANIFEST_DIR"))
/// #                .join("inputs/hello-world.scm")
/// #                .into_os_string()
/// #                .into_string()
/// #                .unwrap();
/// let file_lexer = FileLexer::new(filename).unwrap();
/// let token_stream = file_lexer.into_iter();
/// let datum_stream = DatumIterator::new(token_stream);
/// let vec_of_datums_res: Result<Vec<Datum>, CompilerError> = datum_stream.collect();
/// ```
pub struct DatumIterator<I>
where
    I: Iterator<Item = Result<TokenWithPosition, CompilerError>>,
{
    token_stream: Peekable<I>,
    encountered_error: bool,
}

impl<I> DatumIterator<I>
where
    I: Iterator<Item = Result<TokenWithPosition, CompilerError>>,
{
    /// Creates a `DatumStream` from a `TokenStream`
    pub fn new(token_stream: I) -> Self {
        DatumIterator {
            token_stream: token_stream.peekable(),
            encountered_error: false,
        }
    }
}

impl<I> Iterator for DatumIterator<I>
where
    I: Iterator<Item = Result<TokenWithPosition, CompilerError>>,
{
    type Item = Result<Datum, CompilerError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.encountered_error {
            return None;
        }

        if self.token_stream.peek().is_none() {
            return None;
        }

        let datum_res = parse_datum(&mut self.token_stream);
        if datum_res.is_err() {
            self.encountered_error = true;
        }
        Some(datum_res)
    }
}

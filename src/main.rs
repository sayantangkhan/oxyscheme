use anyhow::Result;
use lexer::TokenStream;
use oxyscheme::*;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() -> Result<()> {
    let filename = env::args().nth(1).unwrap();
    for (index, line) in BufReader::new(File::open(filename)?).lines().enumerate() {
        let line = line?;
        let mut token_stream = TokenStream::new(&line);
        for token in token_stream.by_ref() {
            println!("{:?}", token);
        }
        if !token_stream.is_empty() {
            return Err(
                CompilerError::LexError(String::from(token_stream.input_slice), index + 1).into(),
            );
        };
    }
    Ok(())
}
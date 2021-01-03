use anyhow::Result;
use oxyscheme::*;
use reader::{DatumIterator, FileLexer};
use std::env;

fn main() -> Result<()> {
    let filename = env::args().nth(1).unwrap();

    let file_lexer = FileLexer::new(&filename)?;
    let token_stream = file_lexer.into_iter();
    let datum_stream = DatumIterator::new(token_stream);
    for datum_res in datum_stream {
        let datum = datum_res?;
        println!("{:#?}", datum);
    }
    Ok(())
}

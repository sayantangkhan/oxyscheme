use anyhow::Result;
use oxyscheme::*;
use reader::FileLexer;
use std::env;

fn main() -> Result<()> {
    let filename = env::args().nth(1).unwrap();

    let file_lexer = FileLexer::new(&filename)?;
    for token_res in file_lexer {
        let token = token_res?;
        println!("{:?}", token);
    }
    Ok(())
}

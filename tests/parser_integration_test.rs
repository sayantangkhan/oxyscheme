use oxyscheme::*;
use std::fs;
use std::path::Path;

#[test]
fn lexer_accepts_valid_input() {
    let good_directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("inputs/good-inputs/");

    for file_res in fs::read_dir(&good_directory).unwrap() {
        let file = file_res.unwrap().path();
        let file_lexer = reader::FileLexer::new(file.to_str().unwrap()).unwrap();
        let vec_of_tokens_res: Result<Vec<lexer::TokenWithPosition>, CompilerError> =
            file_lexer.into_iter().collect();
        assert!(vec_of_tokens_res.is_ok());
    }
}

#[test]
fn lexer_rejects_invalid_input() {
    let bad_directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("inputs/bad-lexer-inputs/");

    for file_res in fs::read_dir(&bad_directory).unwrap() {
        let file = file_res.unwrap().path();
        let file_lexer = reader::FileLexer::new(file.to_str().unwrap()).unwrap();
        let vec_of_tokens_res: Result<Vec<lexer::TokenWithPosition>, CompilerError> =
            file_lexer.into_iter().collect();
        assert!(vec_of_tokens_res.is_err());
    }
}

#[test]
fn parser_accepts_valid_input() {
    let good_directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("inputs/good-inputs/");

    for file_res in fs::read_dir(&good_directory).unwrap() {
        let file = file_res.unwrap().path();
        let file_lexer = reader::FileLexer::new(file.to_str().unwrap()).unwrap();
        let token_stream = file_lexer.into_iter();
        let datum_stream = reader::DatumIterator::new(token_stream);
        let vec_of_datums_res: Result<Vec<parser::Datum>, CompilerError> = datum_stream.collect();
        assert!(vec_of_datums_res.is_ok());
    }
}

#[test]
fn parser_rejects_invalid_input() {
    let bad_directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("inputs/bad-parser-inputs/");

    for file_res in fs::read_dir(&bad_directory).unwrap() {
        let file = file_res.unwrap().path();
        let file_lexer = reader::FileLexer::new(file.to_str().unwrap()).unwrap();
        let token_stream = file_lexer.into_iter();
        let datum_stream = reader::DatumIterator::new(token_stream);
        let vec_of_datums_res: Result<Vec<parser::Datum>, CompilerError> = datum_stream.collect();
        assert!(vec_of_datums_res.is_err());
    }
}

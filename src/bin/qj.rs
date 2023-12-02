use clap::Parser;
use json_parser::parser::{ParseError, Value};
use json_parser::tokenizer::Tokenizer;
use std::{fs, process};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    json_file: String,
}

fn main() -> Result<(), ParseError> {
    let cli = Cli::parse();
    let file = fs::read_to_string(cli.json_file).unwrap_or_else(|err| {
        eprintln!("Error reading file: {}", err);
        process::exit(1)
    });
    let mut tokenizer = Tokenizer::new(file);
    let value = Value::parse(&mut tokenizer)?;
    eprintln!("{:#?}", value);
    Ok(())
}

use clap::Parser;
use json_parser::tokenizer::Tokenizer;
use std::{fs, process};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    json_file: String,
}

fn main() {
    let cli = Cli::parse();
    let file = fs::read_to_string(cli.json_file).unwrap_or_else(|err| {
        eprintln!("Error reading file: {}", err);
        process::exit(1)
    });
    let tokenizer = Tokenizer::new(file);
    for token in tokenizer {
        eprintln!("{:#?}", token);
    }
}

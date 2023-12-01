use clap::Parser;
use json_parser::tokenizer::Lexer;
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
    let lexer = Lexer::new(file);
    for token in lexer {
        eprintln!("{:#?}", token);
    }
}

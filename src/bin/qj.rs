use clap::Parser;
use json_parser::{parser::Value, tokenizer::Tokenizer};
use std::time::Instant;
use std::{fs, process};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    json_file: String,
}

fn main() {
    let start = Instant::now();
    let cli = Cli::parse();
    let file = fs::read_to_string(cli.json_file).unwrap_or_else(|err| {
        eprintln!("Error reading file: {}", err);
        process::exit(1)
    });
    let mut tokenizer = Tokenizer::new(file);
    let value = Value::parse(&mut tokenizer).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });
    println!("Parsed in {:?}", start.elapsed());
    let start_stringify = Instant::now();
    let stringified = value.stringified().create();
    println!("{}", stringified);
    println!("Stringified in {:?}", start_stringify.elapsed());
    println!("Took {:?}", start.elapsed());
}

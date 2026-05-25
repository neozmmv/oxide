mod lexer;
mod parser;
mod ast;
mod codegen;

use logos::Logos;
use lexer::Token;
use chumsky::prelude::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: oxide <file.ox>");
        std::process::exit(1);
    }

    let source = std::fs::read_to_string(&args[1]).unwrap_or_else(|e| {
        eprintln!("Error reading file '{}': {}", args[1], e);
        std::process::exit(1);
    });

    // lex
    let tokens: Vec<Token> = Token::lexer(&source)
        .filter_map(|t| t.ok())
        .collect();

    // parse
    let result = parser::parser().parse(tokens.as_slice());

    match result {
        Ok(ast) => println!("{:#?}", ast),
        Err(errs) => {
            if let Some(err) = errs.first() {
                eprintln!("Parse error: {:?}", err);
            }
            std::process::exit(1);
        }
    }
}
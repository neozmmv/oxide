mod lexer;
mod parser;
mod ast;
mod codegen;
mod typechecker;

use typechecker::TypeChecker;
use logos::Logos;
use lexer::Token;
use chumsky::prelude::*;
use codegen::Codegen;
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: oxide <file.ox>");
        std::process::exit(1);
    }

    let source_path = &args[1];
    let source = std::fs::read_to_string(source_path).unwrap_or_else(|e| {
        eprintln!("Error reading file '{}': {}", source_path, e);
        std::process::exit(1);
    });

    // lex
    let tokens: Vec<Token> = Token::lexer(&source)
        .filter_map(|t| t.ok())
        .collect();

    let tokens: Vec<Token> = Token::lexer(&source)
        .filter_map(|t| {
            match t {
                Ok(tok) => Some(tok),
                Err(e) => {
                    eprintln!("Lex error: {:?}", e);
                    None
                }
            }
        })
        .collect();

    println!("Tokens: {:#?}", tokens);

    // parse
    let program = match parser::parser().parse(tokens) {
        Ok(ast) => ast,
        Err(errs) => {
            for err in errs {
                eprintln!("Parse error: {:?}", err);
            }
            std::process::exit(1);
        }
    };

    println!("{:#?}", program);

    let mut checker = TypeChecker::new();
    checker.check_program(&program);
    if !checker.errors.is_empty() {
        for err in &checker.errors {
            eprintln!("Type error: {}", err);
        }
        std::process::exit(1);
    }

    // codegen
    let mut codegen = Codegen::new();
    let c_code = codegen.generate(&program);

    // write .c file
    let c_path = source_path.replace(".ox", ".c");
    std::fs::write(&c_path, &c_code).unwrap_or_else(|e| {
        eprintln!("Error writing C file: {}", e);
        std::process::exit(1);
    });

    // output binary name
    let out_path = source_path.replace(".ox", "");

    // call gcc
    let status = Command::new("gcc")
        .args([&c_path, "-o", &out_path, "-lm"])
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Error calling gcc: {}", e);
            std::process::exit(1);
        });

    if !status.success() {
        eprintln!("Compilation failed.");
        std::process::exit(1);
    }

    println!("Compiled successfully -> {}", out_path);
}
mod lexer;
mod parser;
mod ast;
use lexer::Lexer;
use parser::Parser;

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

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    match parser.parse_program() {
        Ok(program) => println!("{:#?}", program),
        Err(e) => eprintln!("Parse error: {}", e),
    }
}
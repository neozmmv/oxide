pub mod token;
pub use token::Token;
use logos::Logos;

pub fn tokenize(source: &str) -> Vec<Result<Token, ()>> {
    Token::lexer(source).collect()
}
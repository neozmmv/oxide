pub mod token;
use token::Token;
pub struct Lexer {
    input: Vec<char>,
    position: usize,
}
impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }
    fn current(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }
    fn peek(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }
    fn advance(&mut self) -> Option<char> {
        let ch = self.current();
        self.position += 1;
        ch
    }
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
    fn skip_comment(&mut self) {
        // skip until end of line
        while let Some(ch) = self.current() {
            self.advance();
            if ch == '\n' {
                break;
            }
        }
    }
    fn skip_block_comment(&mut self) {
        // skip until */
        while let Some(ch) = self.current() {
            self.advance();
            if ch == '*' {
                if let Some('/') = self.current() {
                    self.advance();
                    break;
                }
            }
        }
    }
    fn read_string(&mut self) -> Token {
        self.advance(); // skip opening "
        let mut s = String::new();
        while let Some(ch) = self.current() {
            if ch == '"' {
                self.advance(); // skip closing "
                break;
            }
            s.push(ch);
            self.advance();
        }
        Token::StringLiteral(s)
    }
    fn read_number(&mut self) -> Token {
        let mut s = String::new();
        while let Some(ch) = self.current() {
            if ch.is_ascii_digit() || ch == '.' {
                s.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        if s.contains('.') {
            Token::Float(s.parse().unwrap())
        } else {
            Token::Int(s.parse().unwrap())
        }
    }
    fn read_ident(&mut self) -> Token {
        let mut s = String::new();
        while let Some(ch) = self.current() {
            if ch.is_alphanumeric() || ch == '_' {
                s.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        // check if it's a keyword
        match s.as_str() {
            "int"        => Token::TypeInt,
            "float"      => Token::TypeFloat,
            "string"     => Token::TypeString,
            "bool"       => Token::TypeBool,
            "const"      => Token::Const,
            "void"       => Token::Void,
            "never"      => Token::Never,
            "return"     => Token::Return,
            "if"         => Token::If,
            "else"       => Token::Else,
            "while"      => Token::While,
            "for"        => Token::For,
            "struct"     => Token::Struct,
            "interface"  => Token::Interface,
            "implements" => Token::Implements,
            "true"       => Token::Bool(true),
            "false"      => Token::Bool(false),
            "null"       => Token::Null,
            _            => Token::Ident(s),
        }
    }
    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        let ch = self.current()?;
        let token = match ch {
            '(' => { self.advance(); Token::LParen },
            ')' => { self.advance(); Token::RParen },
            '{' => { self.advance(); Token::LBrace },
            '}' => { self.advance(); Token::RBrace },
            '[' => { self.advance(); Token::LBracket },
            ']' => { self.advance(); Token::RBracket },
            ';' => { self.advance(); Token::Semicolon },
            ',' => { self.advance(); Token::Comma },
            '%' => { self.advance(); Token::Percent },
            '+' => {
                self.advance();
                match self.current() {
                    Some('+') => { self.advance(); Token::PlusPlus }
                    Some('=') => { self.advance(); Token::PlusAssign }
                    _ => Token::Plus
                }
            },
            '-' => {
                self.advance();
                match self.current() {
                    Some('-') => { self.advance(); Token::MinusMinus }
                    Some('=') => { self.advance(); Token::MinusAssign }
                    _ => Token::Minus
                }
            },
            '*' => {
                self.advance();
                match self.current() {
                    Some('*') => { self.advance(); Token::StarStar }
                    _ => Token::Star
                }
            },
            '/' => {
                self.advance();
                match self.current() {
                    Some('/') => { self.advance(); self.skip_comment(); return self.next_token() }
                    Some('*') => { self.advance(); self.skip_block_comment(); return self.next_token() }
                    _ => Token::Slash
                }
            },
            '=' => {
                self.advance();
                match self.current() {
                    Some('=') => { self.advance(); Token::Eq }
                    _ => Token::Assign
                }
            },
            ':' => {
                self.advance();
                match self.current() {
                    Some('=') => { self.advance(); Token::ColonAssign }
                    _ => Token::Colon
                }
            },
            '!' => {
                self.advance();
                match self.current() {
                    Some('=') => { self.advance(); Token::Neq }
                    _ => Token::Not
                }
            },
            '<' => {
                self.advance();
                match self.current() {
                    Some('=') => { self.advance(); Token::Lte }
                    _ => Token::Lt
                }
            },
            '>' => {
                self.advance();
                match self.current() {
                    Some('=') => { self.advance(); Token::Gte }
                    _ => Token::Gt
                }
            },
            '&' => {
                self.advance();
                match self.current() {
                    Some('&') => { self.advance(); Token::And }
                    _ => Token::Ampersand
                }
            },
            '|' => {
                self.advance();
                match self.current() {
                    Some('|') => { self.advance(); Token::Or }
                    _ => return None
                }
            },
            '?' => {
                self.advance();
                match self.current() {
                    Some('?') => { self.advance(); Token::QuestionQuestion }
                    _ => Token::Question
                }
            },
            '.' => {
                self.advance();
                match self.current() {
                    Some('.') => {
                        self.advance();
                        match self.current() {
                            Some('=') => { self.advance(); Token::DotDotEq }
                            _ => Token::DotDot
                        }
                    }
                    _ => Token::Dot
                }
            },
            '"' => self.read_string(),
            c if c.is_ascii_digit() => self.read_number(),
            c if c.is_alphabetic() || c == '_' => self.read_ident(),
            _ => return None,
        };
        Some(token)
    }
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = vec![];
        while let Some(tok) = self.next_token() {
            tokens.push(tok);
        }
        tokens
    }
}
use crate::lexer::token::Token;
use crate::ast::*;

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position + 1)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.position);
        self.position += 1;
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        match self.current() {
            Some(tok) if tok == expected => {
                self.advance();
                Ok(())
            }
            Some(tok) => Err(format!("Expected {:?}, got {:?}", expected, tok)),
            None => Err(format!("Expected {:?}, got EOF", expected)),
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut items = vec![];
        while self.current().is_some() {
            let item = self.parse_top_level()?;
            items.push(item);
        }
        Ok(Program { items })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        let ty = match self.current() {
            Some(Token::TypeInt)    => { self.advance(); Type::Int }
            Some(Token::TypeFloat)  => { self.advance(); Type::Float }
            Some(Token::TypeString) => { self.advance(); Type::String }
            Some(Token::TypeBool)   => { self.advance(); Type::Bool }
            Some(Token::Void)       => { self.advance(); Type::Void }
            Some(Token::Never)      => { self.advance(); Type::Never }
            Some(tok) => return Err(format!("Expected type, got {:?}", tok)),
            None => return Err("Expected type, got EOF".to_string()),
        };

        // checa se é nullable: string?
        if let Some(Token::Question) = self.current() {
            self.advance();
            return Ok(Type::Nullable(Box::new(ty)));
        }

        Ok(ty)
    }

    fn parse_top_level(&mut self) -> Result<TopLevel, String> {
        match self.current() {
            Some(Token::Struct)    => self.parse_struct(),
            Some(Token::Interface) => self.parse_interface(),
            // método com receiver: void (&Person p) setAge(...)
            Some(Token::LParen)    => self.parse_method_decl(),
            // função normal: void main()
            _                      => self.parse_function(),
        }
    }

    fn parse_function(&mut self) -> Result<TopLevel, String> {
        let return_ty = self.parse_type()?;

        let name = match self.current() {
            Some(Token::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            Some(tok) => return Err(format!("Expected function name, got {:?}", tok)),
            None => return Err("Expected function name, got EOF".to_string()),
        };

        self.expect(&Token::LParen)?;
        let params = self.parse_params()?;
        self.expect(&Token::RParen)?;
        self.expect(&Token::LBrace)?;
        let body = self.parse_block()?;
        self.expect(&Token::RBrace)?;

        Ok(TopLevel::Function { return_ty, name, params, body })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, String> {
        let mut params = vec![];

        // sem parâmetros
        if let Some(Token::RParen) = self.current() {
            return Ok(params);
        }

        loop {
            let by_const_ref = if let Some(Token::Ampersand) = self.current() {
                self.advance();
                if let Some(Token::Const) = self.current() {
                    self.advance();
                    true
                } else {
                    false
                }
            } else {
                false
            };

            let by_ref = if !by_const_ref {
                if let Some(Token::Ampersand) = self.current() {
                    self.advance();
                    true
                } else {
                    false
                }
            } else {
                false
            };

            let ty = self.parse_type()?;

            let name = match self.current() {
                Some(Token::Ident(n)) => {
                    let n = n.clone();
                    self.advance();
                    n
                }
                Some(tok) => return Err(format!("Expected param name, got {:?}", tok)),
                None => return Err("Expected param name, got EOF".to_string()),
            };

            params.push(Param { ty, name, by_ref, by_const_ref });

            match self.current() {
                Some(Token::Comma) => { self.advance(); }
                _ => break,
            }
        }

        Ok(params)
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = vec![];
        while let Some(tok) = self.current() {
            if tok == &Token::RBrace {
                break;
            }
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
        }
        Ok(stmts)
    }

    fn parse_struct(&mut self) -> Result<TopLevel, String> {
        todo!("parse_struct not implemented yet")
    }

    fn parse_interface(&mut self) -> Result<TopLevel, String> {
        todo!("parse_interface not implemented yet")
    }

    fn parse_method_decl(&mut self) -> Result<TopLevel, String> {
        todo!("parse_method_decl not implemented yet")
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        todo!("parse_stmt not implemented yet")
    }
}
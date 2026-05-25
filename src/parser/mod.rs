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
            Some(Token::LParen)    => self.parse_method_decl(),
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
        match self.current() {
            Some(Token::Const) => self.parse_var_decl(true),
            Some(Token::TypeInt)
            | Some(Token::TypeFloat)
            | Some(Token::TypeString)
            | Some(Token::TypeBool) => self.parse_var_decl(false),
            Some(Token::If)       => self.parse_if(),
            Some(Token::While)    => self.parse_while(),
            Some(Token::For)      => self.parse_for(),
            Some(Token::Return)   => self.parse_return(),
            Some(Token::Ident(_)) => self.parse_ident_stmt(),
            Some(tok) => Err(format!("Unexpected token in statement: {:?}", tok)),
            None => Err("Unexpected EOF in statement".to_string()),
        }
    }

    fn parse_var_decl(&mut self, is_const: bool) -> Result<Stmt, String> {
        if is_const {
            self.advance();
        }

        let ty = match self.current() {
            Some(Token::TypeInt)
            | Some(Token::TypeFloat)
            | Some(Token::TypeString)
            | Some(Token::TypeBool) => Some(self.parse_type()?),
            _ => None,
        };

        let name = match self.current() {
            Some(Token::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            Some(tok) => return Err(format!("Expected variable name, got {:?}", tok)),
            None => return Err("Expected variable name, got EOF".to_string()),
        };

        match self.current() {
            Some(Token::Assign) | Some(Token::ColonAssign) => { self.advance(); }
            Some(tok) => return Err(format!("Expected = or :=, got {:?}", tok)),
            None => return Err("Expected = or :=, got EOF".to_string()),
        }

        let value = self.parse_expr()?;
        self.expect(&Token::Semicolon)?;

        Ok(Stmt::VarDecl { ty, name, value, is_const })
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.advance(); // consome 'if'
        self.expect(&Token::LParen)?;
        let condition = self.parse_expr()?;
        self.expect(&Token::RParen)?;
        self.expect(&Token::LBrace)?;
        let then_block = self.parse_block()?;
        self.expect(&Token::RBrace)?;

        let else_block = if let Some(Token::Else) = self.current() {
            self.advance();
            if let Some(Token::If) = self.current() {
                // else if
                let else_if = self.parse_if()?;
                Some(vec![else_if])
            } else {
                // else
                self.expect(&Token::LBrace)?;
                let block = self.parse_block()?;
                self.expect(&Token::RBrace)?;
                Some(block)
            }
        } else {
            None
        };

        Ok(Stmt::If { condition, then_block, else_block })
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.advance(); // consome 'while'
        self.expect(&Token::LParen)?;
        let condition = self.parse_expr()?;
        self.expect(&Token::RParen)?;
        self.expect(&Token::LBrace)?;
        let body = self.parse_block()?;
        self.expect(&Token::RBrace)?;

        Ok(Stmt::While { condition, body })
    }

    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.advance(); // consome 'for'
        self.expect(&Token::LParen)?;

        // checa se é range: for(int i = 0 .. n)
        let is_range = match (self.current(), self.peek()) {
            (Some(Token::TypeInt), _)
            | (Some(Token::TypeFloat), _) => {
                // olha adiante pra ver se tem ..
                true
            }
            _ => false,
        };

        if is_range {
            let ty = Some(self.parse_type()?);
            let name = match self.current() {
                Some(Token::Ident(n)) => {
                    let n = n.clone();
                    self.advance();
                    n
                }
                Some(tok) => return Err(format!("Expected variable name, got {:?}", tok)),
                None => return Err("Expected variable name, got EOF".to_string()),
            };
            self.expect(&Token::Assign)?;
            let range = self.parse_expr()?;
            self.expect(&Token::RParen)?;
            self.expect(&Token::LBrace)?;
            let body = self.parse_block()?;
            self.expect(&Token::RBrace)?;
            Ok(Stmt::ForRange { ty, name, range, body })
        } else {
            // for clássico: for(i := 0; i < n; i++)
            let init = Box::new(self.parse_stmt()?);
            let condition = self.parse_expr()?;
            self.expect(&Token::Semicolon)?;
            let step = Box::new(self.parse_ident_stmt()?);
            self.expect(&Token::RParen)?;
            self.expect(&Token::LBrace)?;
            let body = self.parse_block()?;
            self.expect(&Token::RBrace)?;
            Ok(Stmt::For { init, condition, step, body })
        }
    }

    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.advance(); // consome 'return'

        if let Some(Token::Semicolon) = self.current() {
            self.advance();
            return Ok(Stmt::Return(None));
        }

        let expr = self.parse_expr()?;
        self.expect(&Token::Semicolon)?;
        Ok(Stmt::Return(Some(expr)))
    }

    fn parse_ident_stmt(&mut self) -> Result<Stmt, String> {
        let name = match self.current() {
            Some(Token::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => unreachable!(),
        };

        match self.current() {
            // x := 5;
            Some(Token::ColonAssign) => {
                self.advance();
                let value = self.parse_expr()?;
                self.expect(&Token::Semicolon)?;
                Ok(Stmt::VarDecl { ty: None, name, value, is_const: false })
            }
            // x = 5;
            Some(Token::Assign) => {
                self.advance();
                let value = self.parse_expr()?;
                self.expect(&Token::Semicolon)?;
                Ok(Stmt::Assign { name, value })
            }
            // x++;
            Some(Token::PlusPlus) => {
                self.advance();
                self.expect(&Token::Semicolon)?;
                Ok(Stmt::Expr(Expr::UnaryOp {
                    op: UnaryOp::PostIncrement,
                    expr: Box::new(Expr::Ident(name)),
                }))
            }
            // x--;
            Some(Token::MinusMinus) => {
                self.advance();
                self.expect(&Token::Semicolon)?;
                Ok(Stmt::Expr(Expr::UnaryOp {
                    op: UnaryOp::PostDecrement,
                    expr: Box::new(Expr::Ident(name)),
                }))
            }
            // x += 5;
            Some(Token::PlusAssign) => {
                self.advance();
                let value = self.parse_expr()?;
                self.expect(&Token::Semicolon)?;
                Ok(Stmt::Assign {
                    name: name.clone(),
                    value: Expr::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Ident(name)),
                        right: Box::new(value),
                    },
                })
            }
            // x -= 5;
            Some(Token::MinusAssign) => {
                self.advance();
                let value = self.parse_expr()?;
                self.expect(&Token::Semicolon)?;
                Ok(Stmt::Assign {
                    name: name.clone(),
                    value: Expr::BinaryOp {
                        op: BinaryOp::Sub,
                        left: Box::new(Expr::Ident(name)),
                        right: Box::new(value),
                    },
                })
            }
            // foo(...);
            Some(Token::LParen) => {
                self.advance();
                let args = self.parse_args()?;
                self.expect(&Token::RParen)?;
                self.expect(&Token::Semicolon)?;
                Ok(Stmt::Expr(Expr::Call { name, args }))
            }
            Some(tok) => Err(format!("Unexpected token in statement: {:?}", tok)),
            None => Err("Unexpected EOF in statement".to_string()),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_binary(0)
    }

    fn parse_binary(&mut self, min_precedence: u8) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;

        loop {
            let prec = match self.current() {
                Some(Token::Or)               => 1,
                Some(Token::And)              => 2,
                Some(Token::Eq)
                | Some(Token::Neq)            => 3,
                Some(Token::Lt)
                | Some(Token::Gt)
                | Some(Token::Lte)
                | Some(Token::Gte)            => 4,
                Some(Token::DotDot)
                | Some(Token::DotDotEq)       => 5,
                Some(Token::QuestionQuestion) => 6,
                Some(Token::Plus)
                | Some(Token::Minus)          => 7,
                Some(Token::Star)
                | Some(Token::Slash)
                | Some(Token::Percent)        => 8,
                Some(Token::StarStar)         => 9,
                _ => break,
            };

            if prec < min_precedence {
                break;
            }

            let op = match self.current() {
                Some(Token::Plus)             => BinaryOp::Add,
                Some(Token::Minus)            => BinaryOp::Sub,
                Some(Token::Star)             => BinaryOp::Mul,
                Some(Token::Slash)            => BinaryOp::Div,
                Some(Token::Percent)          => BinaryOp::Mod,
                Some(Token::StarStar)         => BinaryOp::Pow,
                Some(Token::Eq)               => BinaryOp::Eq,
                Some(Token::Neq)              => BinaryOp::Neq,
                Some(Token::Lt)               => BinaryOp::Lt,
                Some(Token::Gt)               => BinaryOp::Gt,
                Some(Token::Lte)              => BinaryOp::Lte,
                Some(Token::Gte)              => BinaryOp::Gte,
                Some(Token::And)              => BinaryOp::And,
                Some(Token::Or)               => BinaryOp::Or,
                Some(Token::DotDot)           => BinaryOp::Range,
                Some(Token::DotDotEq)         => BinaryOp::RangeInclusive,
                Some(Token::QuestionQuestion) => BinaryOp::NullCoalesce,
                _ => break,
            };

            self.advance();
            let right = self.parse_binary(prec + 1)?;
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.current() {
            Some(Token::Not) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp { op: UnaryOp::Not, expr: Box::new(expr) })
            }
            Some(Token::Minus) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp { op: UnaryOp::Neg, expr: Box::new(expr) })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.current() {
            Some(Token::Int(n)) => {
                let n = *n;
                self.advance();
                Ok(Expr::Literal(Literal::Int(n)))
            }
            Some(Token::Float(f)) => {
                let f = *f;
                self.advance();
                Ok(Expr::Literal(Literal::Float(f)))
            }
            Some(Token::StringLiteral(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Literal(Literal::String(s)))
            }
            Some(Token::Bool(b)) => {
                let b = *b;
                self.advance();
                Ok(Expr::Literal(Literal::Bool(b)))
            }
            Some(Token::Null) => {
                self.advance();
                Ok(Expr::Literal(Literal::Null))
            }
            Some(Token::Ident(_)) => self.parse_ident_expr(),
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Some(tok) => Err(format!("Unexpected token in expression: {:?}", tok)),
            None => Err("Unexpected EOF in expression".to_string()),
        }
    }

    fn parse_ident_expr(&mut self) -> Result<Expr, String> {
        let name = match self.current() {
            Some(Token::Ident(n)) => {
                let n = n.clone();
                self.advance();
                n
            }
            _ => unreachable!(),
        };

        match self.current() {
            Some(Token::LParen) => {
                self.advance();
                let args = self.parse_args()?;
                self.expect(&Token::RParen)?;
                Ok(Expr::Call { name, args })
            }
            Some(Token::Dot) => {
                self.advance();
                let field = match self.current() {
                    Some(Token::Ident(f)) => {
                        let f = f.clone();
                        self.advance();
                        f
                    }
                    Some(tok) => return Err(format!("Expected field name, got {:?}", tok)),
                    None => return Err("Expected field name, got EOF".to_string()),
                };
                if let Some(Token::LParen) = self.current() {
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect(&Token::RParen)?;
                    Ok(Expr::MethodCall {
                        object: Box::new(Expr::Ident(name)),
                        method: field,
                        args,
                    })
                } else {
                    Ok(Expr::FieldAccess {
                        object: Box::new(Expr::Ident(name)),
                        field,
                    })
                }
            }
            Some(Token::LBracket) => {
                self.advance();
                let index = self.parse_expr()?;
                self.expect(&Token::RBracket)?;
                Ok(Expr::Index {
                    array: Box::new(Expr::Ident(name)),
                    index: Box::new(index),
                })
            }
            _ => Ok(Expr::Ident(name)),
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = vec![];

        if let Some(Token::RParen) = self.current() {
            return Ok(args);
        }

        loop {
            args.push(self.parse_expr()?);
            match self.current() {
                Some(Token::Comma) => { self.advance(); }
                _ => break,
            }
        }

        Ok(args)
    }
}
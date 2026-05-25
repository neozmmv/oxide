use chumsky::prelude::*;
use crate::lexer::Token;
use crate::ast::*;

pub fn parser() -> impl Parser<Token, Program, Error = Simple<Token>> {
    // type parser
    let ty = select! {
        Token::TypeInt    => Type::Int,
        Token::TypeFloat  => Type::Float,
        Token::TypeString => Type::String,
        Token::TypeBool   => Type::Bool,
        Token::TypeVoid   => Type::Void,
        Token::TypeNever  => Type::Never,
    }
    .then(just(Token::Question).or_not())
    .map(|(ty, q)| {
        if q.is_some() {
            Type::Nullable(Box::new(ty))
        } else {
            ty
        }
    });

    // literal parser
    let literal = select! {
        Token::Int(n)           => Literal::Int(n),
        Token::Float(f)         => Literal::Float(f),
        Token::StringLiteral(s) => Literal::String(s),
        Token::True             => Literal::Bool(true),
        Token::False            => Literal::Bool(false),
        Token::Null             => Literal::Null,
    };

    // identifier parser
    let ident = select! {
        Token::Ident(s) => s,
    };

    // expression parser (recursive)
    let expr = recursive(|expr| {
        let primary = literal.map(Expr::Literal)
            .or(ident.clone().then(
                just(Token::LParen)
                    .ignore_then(
                        expr.clone()
                            .separated_by(just(Token::Comma))
                            .collect::<Vec<_>>()
                    )
                    .then_ignore(just(Token::RParen))
                    .or_not()
            ).map(|(name, args)| {
                if let Some(args) = args {
                    Expr::Call { name, args }
                } else {
                    Expr::Ident(name)
                }
            }))
            .or(just(Token::LParen)
                .ignore_then(expr.clone())
                .then_ignore(just(Token::RParen)));

        // field access and method calls: foo.bar or foo.bar(...)
        let primary = primary.then(
            just(Token::Dot)
                .ignore_then(ident.clone())
                .then(
                    just(Token::LParen)
                        .ignore_then(
                            expr.clone()
                                .separated_by(just(Token::Comma))
                                .collect::<Vec<_>>()
                        )
                        .then_ignore(just(Token::RParen))
                        .or_not()
                )
                .repeated()
        ).foldl(|obj, (field, args)| {
            if let Some(args) = args {
                Expr::MethodCall { object: Box::new(obj), method: field, args }
            } else {
                Expr::FieldAccess { object: Box::new(obj), field }
            }
        });

        // post-increment and post-decrement: x++, x--
        let primary = primary.then(
            just(Token::PlusPlus).to(UnaryOp::PostIncrement)
                .or(just(Token::MinusMinus).to(UnaryOp::PostDecrement))
                .or_not()
        ).map(|(expr, op)| {
            if let Some(op) = op {
                Expr::UnaryOp { op, expr: Box::new(expr) }
            } else {
                expr
            }
        });

        // unary operators
        let unary = just(Token::Not)
            .to(UnaryOp::Not)
            .or(just(Token::Minus).to(UnaryOp::Neg))
            .repeated()
            .then(primary)
            .foldr(|op, expr| Expr::UnaryOp { op, expr: Box::new(expr) });

        // binary operators by precedence
        let factor = unary.clone().then(
            just(Token::StarStar).to(BinaryOp::Pow)
                .then(unary)
                .repeated()
        ).foldl(|left, (op, right)| Expr::BinaryOp {
            op, left: Box::new(left), right: Box::new(right)
        });

        let term = factor.clone().then(
            just(Token::Star).to(BinaryOp::Mul)
                .or(just(Token::Slash).to(BinaryOp::Div))
                .or(just(Token::Percent).to(BinaryOp::Mod))
                .then(factor)
                .repeated()
        ).foldl(|left, (op, right)| Expr::BinaryOp {
            op, left: Box::new(left), right: Box::new(right)
        });

        let sum = term.clone().then(
            just(Token::Plus).to(BinaryOp::Add)
                .or(just(Token::Minus).to(BinaryOp::Sub))
                .then(term)
                .repeated()
        ).foldl(|left, (op, right)| Expr::BinaryOp {
            op, left: Box::new(left), right: Box::new(right)
        });

        let comparison = sum.clone().then(
            just(Token::EqualEqual).to(BinaryOp::Eq)
                .or(just(Token::NotEqual).to(BinaryOp::Neq))
                .or(just(Token::Less).to(BinaryOp::Lt))
                .or(just(Token::Greater).to(BinaryOp::Gt))
                .or(just(Token::LessEqual).to(BinaryOp::Lte))
                .or(just(Token::GreaterEqual).to(BinaryOp::Gte))
                .then(sum)
                .repeated()
        ).foldl(|left, (op, right)| Expr::BinaryOp {
            op, left: Box::new(left), right: Box::new(right)
        });

        let logic = comparison.clone().then(
            just(Token::And).to(BinaryOp::And)
                .or(just(Token::Or).to(BinaryOp::Or))
                .then(comparison)
                .repeated()
        ).foldl(|left, (op, right)| Expr::BinaryOp {
            op, left: Box::new(left), right: Box::new(right)
        });

        let null_coalesce = logic.clone().then(
            just(Token::QuestionQuestion).to(BinaryOp::NullCoalesce)
                .then(logic)
                .repeated()
        ).foldl(|left, (op, right)| Expr::BinaryOp {
            op, left: Box::new(left), right: Box::new(right)
        });

        null_coalesce
    });

    // statement parser
    let stmt = recursive(|stmt| {
        let var_decl = ty.clone()
            .then(ident.clone())
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .map(|((ty, name), value)| Stmt::VarDecl {
                ty: Some(ty),
                name,
                value,
                is_const: false,
            });

        let const_decl = just(Token::Const)
            .ignore_then(ty.clone().or_not())
            .then(ident.clone())
            .then_ignore(just(Token::Assign).or(just(Token::ColonAssign)))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .map(|((ty, name), value)| Stmt::VarDecl {
                ty,
                name,
                value,
                is_const: true,
            });

        let infer_decl = ident.clone()
            .then_ignore(just(Token::ColonAssign))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .map(|(name, value)| Stmt::VarDecl {
                ty: None,
                name,
                value,
                is_const: false,
            });

        let assign = ident.clone()
            .then_ignore(just(Token::Assign))
            .then(expr.clone())
            .then_ignore(just(Token::Semicolon))
            .map(|(name, value)| Stmt::Assign { name, value });

        let block = stmt.clone()
            .repeated()
            .collect::<Vec<_>>()
            .delimited_by(just(Token::LBrace), just(Token::RBrace));

        let if_stmt = recursive(|if_stmt| {
            just(Token::If)
                .ignore_then(expr.clone().delimited_by(just(Token::LParen), just(Token::RParen)))
                .then(block.clone())
                .then(
                    just(Token::Else)
                        .ignore_then(
                            if_stmt.map(|s| vec![s])
                                .or(block.clone())
                        )
                        .or_not()
                )
                .map(|((condition, then_block), else_block)| Stmt::If {
                    condition,
                    then_block,
                    else_block,
                })
        });

        let while_stmt = just(Token::While)
            .ignore_then(expr.clone().delimited_by(just(Token::LParen), just(Token::RParen)))
            .then(block.clone())
            .map(|(condition, body)| Stmt::While { condition, body });

        let return_stmt = just(Token::Return)
            .ignore_then(expr.clone().or_not())
            .then_ignore(just(Token::Semicolon))
            .map(Stmt::Return);

        let expr_stmt = expr.clone()
            .then_ignore(just(Token::Semicolon))
            .map(Stmt::Expr);

        var_decl
            .or(const_decl)
            .or(infer_decl)
            .or(assign)
            .or(if_stmt)
            .or(while_stmt)
            .or(return_stmt)
            .or(expr_stmt)
    });

    // function parser
    let block = stmt.clone()
        .repeated()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::LBrace), just(Token::RBrace));

    let param = just(Token::Ampersand)
        .then(just(Token::Const).or_not())
        .or_not()
        .then(ty.clone())
        .then(ident.clone())
        .map(|((amp, ty), name)| {
            let (by_ref, by_const_ref) = match amp {
                Some((_, None))    => (true, false),
                Some((_, Some(_))) => (false, true),
                None               => (false, false),
            };
            Param { ty, name, by_ref, by_const_ref }
        });

    let function = ty.clone()
        .then(ident.clone())
        .then(
            param.separated_by(just(Token::Comma))
                .collect::<Vec<_>>()
                .delimited_by(just(Token::LParen), just(Token::RParen))
        )
        .then(block)
        .map(|(((return_ty, name), params), body)| {
            TopLevel::Function { return_ty, name, params, body }
        });

    function
        .repeated()
        .collect::<Vec<_>>()
        .map(|items| Program { items })
}
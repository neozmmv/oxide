use std::collections::HashMap;
use crate::ast::*;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    UndeclaredVariable(String),
    TypeMismatch { expected: Type, found: Type },
    NullAssignedToNonNullable(String),
    ReturnTypeMismatch { expected: Type, found: Type },
    UndeclaredFunction(String),
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TypeError::UndeclaredVariable(name) =>
                write!(f, "undeclared variable '{}'", name),
            TypeError::TypeMismatch { expected, found } =>
                write!(f, "type mismatch: expected '{:?}', found '{:?}'", expected, found),
            TypeError::NullAssignedToNonNullable(name) =>
                write!(f, "cannot assign null to non-nullable variable '{}'", name),
            TypeError::ReturnTypeMismatch { expected, found } =>
                write!(f, "return type mismatch: expected '{:?}', found '{:?}'", expected, found),
            TypeError::UndeclaredFunction(name) =>
                write!(f, "undeclared function '{}'", name),
        }
    }
}

pub struct TypeChecker {
    // stack of scopes, each scope maps variable name to type
    scopes: Vec<HashMap<String, Type>>,
    // maps function name to (return type, param types)
    functions: HashMap<String, (Type, Vec<Type>)>,
    current_return_ty: Option<Type>,
    pub errors: Vec<TypeError>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            current_return_ty: None,
            errors: vec![],
        }
    }

    // scope management
    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    fn lookup(&self, name: &str) -> Option<&Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    pub fn check_program(&mut self, program: &Program) {
        // first pass: register all functions
        for item in &program.items {
            if let TopLevel::Function { return_ty, name, params, .. } = item {
                let param_types = params.iter().map(|p| p.ty.clone()).collect();
                self.functions.insert(name.clone(), (return_ty.clone(), param_types));
            }
        }

        // second pass: check bodies
        for item in &program.items {
            self.check_top_level(item);
        }
    }

    fn check_top_level(&mut self, item: &TopLevel) {
        match item {
            TopLevel::Function { return_ty, params, body, .. } => {
                self.push_scope();
                self.current_return_ty = Some(return_ty.clone());

                // declare params in scope
                for param in params {
                    self.declare(&param.name, param.ty.clone());
                }

                for stmt in body {
                    self.check_stmt(stmt);
                }

                self.current_return_ty = None;
                self.pop_scope();
            }
            _ => {} // structs and interfaces handled later
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { ty, name, value, .. } => {
                let value_ty = self.check_expr(value);

                let declared_ty = match ty {
                    Some(t) => t.clone(),
                    None => value_ty.clone(), // inferred
                };

                // check null assignment to non-nullable
                if value_ty == Type::Null && !matches!(declared_ty, Type::Nullable(_)) {
                    self.errors.push(TypeError::NullAssignedToNonNullable(name.clone()));
                }

                self.declare(name, declared_ty);
            }
            Stmt::Assign { name, value } => {
                let value_ty = self.check_expr(value);
                match self.lookup(name) {
                    Some(expected) => {
                        let expected = expected.clone();
                        if !self.types_compatible(&expected, &value_ty) {
                            self.errors.push(TypeError::TypeMismatch {
                                expected,
                                found: value_ty,
                            });
                        }
                    }
                    None => self.errors.push(TypeError::UndeclaredVariable(name.clone())),
                }
            }
            Stmt::Return(expr) => {
                let ret_ty = match expr {
                    Some(e) => self.check_expr(e),
                    None => Type::Void,
                };
                if let Some(expected) = &self.current_return_ty.clone() {
                    if !self.types_compatible(expected, &ret_ty) {
                        self.errors.push(TypeError::ReturnTypeMismatch {
                            expected: expected.clone(),
                            found: ret_ty,
                        });
                    }
                }
            }
            Stmt::If { condition, then_block, else_block } => {
                self.check_expr(condition);
                self.push_scope();
                for s in then_block { self.check_stmt(s); }
                self.pop_scope();
                if let Some(else_stmts) = else_block {
                    self.push_scope();
                    for s in else_stmts { self.check_stmt(s); }
                    self.pop_scope();
                }
            }
            Stmt::While { condition, body } => {
                self.check_expr(condition);
                self.push_scope();
                for s in body { self.check_stmt(s); }
                self.pop_scope();
            }
            Stmt::For { init, condition, step, body } => {
                self.push_scope();
                self.check_stmt(init);
                self.check_expr(condition);
                self.check_stmt(step);
                for s in body { self.check_stmt(s); }
                self.pop_scope();
            }
            Stmt::ForRange { ty, name, range, body } => {
                self.push_scope();
                let range_ty = ty.clone().unwrap_or(Type::Int);
                self.declare(name, range_ty);
                self.check_expr(range);
                for s in body { self.check_stmt(s); }
                self.pop_scope();
            }
            Stmt::Expr(expr) => { self.check_expr(expr); }
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Int(_)    => Type::Int,
                Literal::Float(_)  => Type::Float,
                Literal::String(_) => Type::String,
                Literal::Bool(_)   => Type::Bool,
                Literal::Null      => Type::Null,
            },
            Expr::Ident(name) => {
                match self.lookup(name) {
                    Some(ty) => ty.clone(),
                    None => {
                        self.errors.push(TypeError::UndeclaredVariable(name.clone()));
                        Type::Int // fallback
                    }
                }
            }
            Expr::BinaryOp { op, left, right } => {
                let l = self.check_expr(left);
                let r = self.check_expr(right);
                match op {
                    BinaryOp::Add | BinaryOp::Sub
                    | BinaryOp::Mul | BinaryOp::Div
                    | BinaryOp::Mod | BinaryOp::Pow => {
                        // float if either side is float
                        if l == Type::Float || r == Type::Float {
                            Type::Float
                        } else {
                            Type::Int
                        }
                    }
                    BinaryOp::Eq | BinaryOp::Neq
                    | BinaryOp::Lt | BinaryOp::Gt
                    | BinaryOp::Lte | BinaryOp::Gte
                    | BinaryOp::And | BinaryOp::Or => Type::Bool,
                    BinaryOp::NullCoalesce => l,
                    BinaryOp::Range | BinaryOp::RangeInclusive => Type::Int,
                }
            }
            Expr::UnaryOp { op, expr } => {
                let ty = self.check_expr(expr);
                match op {
                    UnaryOp::Not => Type::Bool,
                    UnaryOp::Neg => ty,
                    UnaryOp::PostIncrement | UnaryOp::PostDecrement => ty,
                }
            }
            Expr::Call { name, args } => {
                for arg in args { self.check_expr(arg); }
                match self.functions.get(name) {
                    Some((ret_ty, _)) => ret_ty.clone(),
                    None => match name.as_str() {
                        // built-in functions
                        "println" | "printf" => Type::Void,
                        "sprintf" => Type::String,
                        _ => {
                            self.errors.push(TypeError::UndeclaredFunction(name.clone()));
                            Type::Void
                        }
                    }
                }
            }
            Expr::FieldAccess { object, .. } => {
                self.check_expr(object);
                Type::Int // placeholder until structs are implemented
            }
            Expr::MethodCall { object, args, .. } => {
                self.check_expr(object);
                for arg in args { self.check_expr(arg); }
                Type::Void // placeholder until structs are implemented
            }
            Expr::Index { array, index } => {
                self.check_expr(array);
                self.check_expr(index);
                Type::Int // placeholder until arrays are implemented
            }
        }
    }

    fn types_compatible(&self, expected: &Type, found: &Type) -> bool {
        if expected == found { return true; }
        // nullable accepts null
        if matches!(expected, Type::Nullable(_)) && found == &Type::Null { return true; }
        // nullable<T> accepts T
        if let Type::Nullable(inner) = expected {
            if inner.as_ref() == found { return true; }
        }
        false
    }
}
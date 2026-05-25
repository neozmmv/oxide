use std::fmt;

#[derive(Debug, Clone, PartialEq)]

pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Void,
    Never,
    Null,
    Nullable(Box<Type>), // string?
    Array(Box<Type>),    // int[]
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Int            => write!(f, "int"),
            Type::Float          => write!(f, "float"),
            Type::String         => write!(f, "string"),
            Type::Bool           => write!(f, "bool"),
            Type::Void           => write!(f, "void"),
            Type::Never          => write!(f, "never"),
            Type::Null           => write!(f, "null"),
            Type::Nullable(inner) => write!(f, "{}?", inner),
            Type::Array(inner)   => write!(f, "{}[]", inner),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod, Pow, // aritméticos
    Eq, Neq, Lt, Gt, Lte, Gte,   // comparação
    And, Or,                       // lógicos
    Range,                         // ..
    RangeInclusive,                // ..=
    NullCoalesce,                  // ??
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Not,       // !
    Neg,       // -x
    PostIncrement, // x++
    PostDecrement, // x--
}

#[derive(Debug, Clone)]
pub enum StringPart {
    Literal(String),
    Expr(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Ident(String),
    BinaryOp {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    StringInterp(Vec<StringPart>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    VarDecl {
        ty: Option<Type>,   // None = inferido com :=
        name: String,
        value: Expr,
        is_const: bool,
    },
    Assign {
        name: String,
        value: Expr,
    },
    If {
        condition: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    For {
        init: Box<Stmt>,
        condition: Expr,
        step: Box<Stmt>,
        body: Vec<Stmt>,
    },
    ForRange {
        ty: Option<Type>,
        name: String,
        range: Expr,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub struct Param {
    pub ty: Type,
    pub name: String,
    pub by_ref: bool,       // &
    pub by_const_ref: bool, // &const
}

#[derive(Debug, Clone)]
pub enum TopLevel {
    Function {
        return_ty: Type,
        name: String,
        params: Vec<Param>,
        body: Vec<Stmt>,
    },
    StructDecl {
        name: String,
        fields: Vec<(Type, String)>,
        implements: Option<String>,
    },
    InterfaceDecl {
        name: String,
        fields: Vec<(Type, String)>,
        methods: Vec<(Type, String, Vec<Param>)>,
    },
    MethodDecl {
        receiver_ty: String,
        receiver_name: String,
        by_ref: bool,
        return_ty: Type,
        name: String,
        params: Vec<Param>,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<TopLevel>,
}
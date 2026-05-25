#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // literals
    Int(i64),
    Float(f64),
    StringLiteral(String),
    Bool(bool),
    Null,

    // identifier (variable names, function names, etc)
    Ident(String),

    // types
    TypeInt,
    TypeFloat,
    TypeString,
    TypeBool,

    // keywords
    Const,
    Void,
    Never,
    Return,
    If,
    Else,
    While,
    For,
    Struct,
    Interface,
    Implements,

    // operators
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Percent,     // %
    StarStar,    // **
    Assign,      // =
    ColonAssign, // :=
    PlusPlus,    // ++
    MinusMinus,  // --
    PlusAssign,  // +=
    MinusAssign, // -=

    // comparison
    Eq,  // ==
    Neq, // !=
    Lt,  // 
    Gt,  // >
    Lte, // <=
    Gte, // >=

    // logical
    And, // &&
    Or,  // ||
    Not, // !

    // range
    DotDot,      // ..
    DotDotEq,    // ..=

    // null coalescing
    QuestionQuestion, // ??

    // punctuation
    LParen,    // (
    RParen,    // )
    LBrace,    // {
    RBrace,    // }
    LBracket,  // [
    RBracket,  // ]
    Semicolon, // ;
    Colon,     // :
    Comma,     // ,
    Dot,       // .
    Ampersand, // &
    Question,  // ?

    // special
    EOF,
}
use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")] // skips whitespace automatically
#[logos(skip(r"//[^\n]*", allow_greedy = true))]
#[logos(skip r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/")]
pub enum Token {
    // types
    #[token("int")] TypeInt,
    #[token("float")] TypeFloat,
    #[token("string")] TypeString,
    #[token("bool")] TypeBool,

    // modifiers
    #[token("public")] Public,
    #[token("private")] Private,
    #[token("static")] Static,
    #[token("const")] Const,

    // functions
    #[token("void")] TypeVoid,
    #[token("never")] TypeNever,
    #[token("return")] Return,

    // control flow
    #[token("if")] If,
    #[token("else")] Else,
    #[token("while")] While,
    #[token("for")] For,

    // structs, interfaces...
    #[token("struct")] Struct,
    #[token("interface")] Interface,
    #[token("implements")] Implements,

    // reserved keywords
    #[token("true")] True,
    #[token("false")] False,
    #[token("null")] Null,

    // operators
    #[token("=")] Assign,
    #[token("+")] Plus,
    #[token("-")] Minus,
    #[token("*")] Star,
    #[token("**")] StarStar,
    #[token("/")] Slash,
    #[token("==")] EqualEqual,
    #[token("!=")] NotEqual,
    #[token("<")] Less,
    #[token(">")] Greater,
    #[token("<=")] LessEqual,
    #[token(">=")] GreaterEqual,
    #[token("&&")] And,
    #[token("||")] Or,
    #[token("!")] Not,
    #[token(",")] Comma,
    #[token(".")] Dot,
    #[token("..")] DotDot,
    #[token("..=")] DotDotEq,
    #[token("++")] PlusPlus,
    #[token("--")] MinusMinus,
    #[token("%")] Percent,
    #[token("+=")] PlusAssign,
    #[token("-=")] MinusAssign,
    #[token("*=")] StarAssign,
    #[token("/=")] SlashAssign,
    #[token(":=")] ColonAssign,
    #[token("?")] Question,
    #[token("??")] QuestionQuestion,
    #[token("&")] Ampersand,
    
    // delimiters
    #[token(";")] Semicolon,
    #[token("(")] LParen,
    #[token(")")] RParen,
    #[token("{")] LBrace,
    #[token("}")] RBrace,
    #[token("[")] LBracket,
    #[token("]")] RBracket,


    // regex for identifiers and numbers

    #[regex("[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Int(i64),

    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),


    #[regex(r#""[^"$]*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    StringLiteral(String),

    // strings with interpolation: captured raw with quotes
#[regex(r#""[^"]*\$\{[^"]*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    StringInterp(String),
}

impl Eq for Token {}

impl std::hash::Hash for Token {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Token::Int(n)            => n.hash(state),
            Token::Float(f)          => f.to_bits().hash(state),
            Token::Ident(s)          => s.hash(state),
            Token::StringLiteral(s)  => s.hash(state),
            _                        => {}
        }
    }
}
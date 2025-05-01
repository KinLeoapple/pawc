// src/token.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    IntLiteral(i32),
    FloatLiteral(f64),
    LongLiteral(i64),
    StringLiteral(String),
    CharLiteral(char),
    BoolLiteral(bool),

    // Identifiers and keywords
    Identifier(String),
    Type(String),
    Keyword(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqEq,
    NotEq,
    Lt,
    Gt,
    Le,
    Ge,
    AndAnd,
    OrOr,
    Not,
    Assign,
    LeftArrow,
    Range,    // '..'

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket, // '['
    RBracket, // ']'
    Comma,
    Colon,
    Dot,      // '.'

    // Other
    Comment(String),
    Eof,
    Error(String),
}

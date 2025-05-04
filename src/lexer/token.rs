// src/lexer/token.rs

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    IntLiteral(i32),
    FloatLiteral(f32),
    DoubleLiteral(f64),
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
    Range,    // ".."
    Question,

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Dot,

    Comment(String),
    Eof,
    Error(String),
}

/// 带源位置信息的 Token
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, column: usize) -> Self {
        Token { kind, line, column }
    }

    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }
    pub fn line(&self) -> usize { self.line }
    pub fn column(&self) -> usize { self.column }
}
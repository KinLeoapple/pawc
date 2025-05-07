// src/lexer/token.rs

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind<'a> {
    // Literals
    IntLiteral(i32),
    FloatLiteral(f32),
    DoubleLiteral(f64),
    LongLiteral(i64),
    StringLiteral(&'a str),
    CharLiteral(char),
    BoolLiteral(bool),
    
    // Identifiers and keywords
    Identifier(&'a str),
    Type(&'a str),
    Keyword(&'a str),

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

    Comment(&'a str),
    Eof,
    Error(String),
}

/// 带源位置信息的 Token
#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub line: usize,
    pub column: usize,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind<'a>, line: usize, column: usize) -> Self {
        Token { kind, line, column }
    }

    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }
    pub fn line(&self) -> usize { self.line }
    pub fn column(&self) -> usize { self.column }
}
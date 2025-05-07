// src/ast/expr.rs

use crate::ast::method::Method;

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind<'a> {
    LiteralInt(i32),
    LiteralLong(i64),
    LiteralFloat(f32),
    LiteralDouble(f64),
    LiteralString(&'a str),
    LiteralChar(char),
    LiteralBool(bool),
    LiteralNopaw,

    Var(&'a str),

    UnaryOp {
        op: &'a str,
        expr: Box<Expr<'a>>,
    },

    BinaryOp {
        op: BinaryOp,
        left: Box<Expr<'a>>,
        right: Box<Expr<'a>>,
    },

    Call {
        name: &'a str,
        args: Vec<Expr<'a>>,
    },

    MethodCall {
        receiver: Box<Expr<'a>>,
        method: Method,
        args: Vec<Expr<'a>>,
    },

    Cast {
        expr: Box<Expr<'a>>,
        ty: &'a str,
    },

    ArrayLiteral(Vec<Expr<'a>>),
    Index {
        array: Box<Expr<'a>>,
        index: Box<Expr<'a>>,
    },
    FieldAccess {
        expr: Box<Expr<'a>>,
        field: &'a str,
    },
    RecordInit {
        name: &'a str,
        fields: Vec<(&'a str, Expr<'a>)>,
    },
    Await {
        expr: Box<Expr<'a>>,
    },
}

/// 带位置的表达式
#[derive(Debug, Clone, PartialEq)]
pub struct Expr<'a> {
    pub kind: ExprKind<'a>,
    pub line: usize,
    pub col: usize,
}

impl<'a> Expr<'a> {
    /// 构造带位置的表达式
    pub fn new(kind: ExprKind<'a>, line: usize, col: usize) -> Self {
        Expr { kind, line, col }
    }
}

/// 二元运算符枚举
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    As,
}

// src/ast/expr.rs

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    LiteralInt(i32),
    LiteralLong(i64),
    LiteralFloat(f32),
    LiteralDouble(f64),
    LiteralString(String),
    LiteralChar(char),
    LiteralBool(bool),
    LiteralNopaw,

    Var(String),

    UnaryOp {
        op: String,
        expr: Box<Expr>,
    },

    BinaryOp {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    Call {
        name: String,
        args: Vec<Expr>,
    },

    MethodCall {
        receiver: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },

    Cast {
        expr: Box<Expr>,
        ty: String,
    },

    ArrayLiteral(Vec<Expr>),
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    FieldAccess {
        expr: Box<Expr>,
        field: String,
    },
    RecordInit {
        name: String,
        fields: Vec<(String, Expr)>,
    },
    Await {
        expr: Box<Expr>,
    },
}

/// 带位置的表达式
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub line: usize,
    pub col: usize,
}

impl Expr {
    /// 构造带位置的表达式
    pub fn new(kind: ExprKind, line: usize, col: usize) -> Self {
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

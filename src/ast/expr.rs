// src/ast/expr.rs

/// 二元运算符
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// 抽象语法树中的表达式节点，直接携带行列
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub line: usize,  // 行号
    pub col: usize,   // 列号
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    LiteralInt(i32),
    LiteralLong(i64),
    LiteralFloat(f64),
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
    Cast {
        expr: Box<Expr>,
        ty: String,
    },

    /// 数组字面量：[e1, e2, e3]
    ArrayLiteral(Vec<Expr>),
    /// 索引操作：arr[idx]
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    /// 属性访问（可用于数组长度等）：obj.prop
    Property {
        object: Box<Expr>,
        name: String,
    },
}
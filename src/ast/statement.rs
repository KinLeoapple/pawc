// src/ast/statement.rs

use crate::ast::{Expr, Param};

/// 语句种类
#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    Let {
        name: String,
        ty: String,
        value: Expr,
    },
    Say(Expr),
    Assign {
        name: String,
        value: Expr,
    },
    Ask {
        name: String,
        ty: String,
        prompt: String,
    },
    AskPrompt(String),
    Return(Option<Expr>),
    Break,
    Continue,
    Expr(Expr),

    If {
        condition: Expr,
        body: Vec<Statement>,
        else_branch: Option<Box<Statement>>,
    },

    LoopForever(Vec<Statement>),

    LoopWhile {
        condition: Expr,
        body: Vec<Statement>,
    },

    LoopRange {
        var: String,
        start: Expr,
        end: Expr,
        body: Vec<Statement>,
    },

    FunDecl {
        name: String,
        params: Vec<Param>,
        return_type: Option<String>,
        body: Vec<Statement>,
    },

    Block(Vec<Statement>),
}

/// 语句
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub kind: StatementKind,
    pub line: usize,
}

impl Statement {
    pub fn new(kind: StatementKind) -> Self {
        Statement { kind, line: 0 }
    }
}

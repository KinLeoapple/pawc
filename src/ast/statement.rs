// src/ast/statement.rs

use crate::ast::expr::Expr;
use crate::ast::method::MethodSig;
use crate::ast::param::Param;

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
    LoopArray {
        var: String,
        array: Expr,
        body: Vec<Statement>,
    },

    FunDecl {
        receiver: Option<String>,
        name: String,
        params: Vec<Param>,
        is_async: bool,
        return_type: Option<String>,
        body: Vec<Statement>,
    },
    Block(Vec<Statement>),

    Throw(Expr),
    TryCatchFinally {
        body: Vec<Statement>,
        err_name: String,
        handler: Vec<Statement>,
        finally: Vec<Statement>,
    },

    Import {
        module: Vec<String>,
        alias: String,
    },
    InterfaceDecl {
        name: String,
        methods: Vec<MethodSig>,
    },

    RecordDecl {
        name: String,
        fields: Vec<Param>,
        impls: Vec<String>,
    },
}

/// 带位置的语句
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub kind: StatementKind,
    pub line: usize,
    pub col: usize,
}

impl Statement {
    pub fn new(kind: StatementKind, line: usize, col: usize) -> Self {
        Statement { kind, line, col }
    }
}

// src/ast/statement.rs

use crate::ast::expr::Expr;
use crate::ast::param::Param;

/// 语句种类
#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind<'a> {
    Let {
        name: &'a str,
        ty: &'a str,
        value: Expr<'a>,
    },
    Say(Expr<'a>),
    Assign {
        name: &'a str,
        value: Expr<'a>,
    },
    Ask {
        name: &'a str,
        ty: String,
        prompt: &'a str,
    },
    AskPrompt(&'a str),
    Return(Option<Expr<'a>>),
    Break,
    Continue,
    Expr(Expr<'a>),

    If {
        condition: Expr<'a>,
        body: Vec<Statement<'a>>,
        else_branch: Option<Box<Statement<'a>>>,
    },
    LoopForever(Vec<Statement<'a>>),
    LoopWhile {
        condition: Expr<'a>,
        body: Vec<Statement<'a>>,
    },
    LoopRange {
        var: &'a str,
        start: Expr<'a>,
        end: Expr<'a>,
        body: Vec<Statement<'a>>,
    },
    LoopArray {
        var: &'a str,
        array: Expr<'a>,
        body: Vec<Statement<'a>>,
    },

    FunDecl {
        name: &'a str,
        params: Vec<Param>,
        is_async: bool,
        return_type: Option<&'a str>,
        body: Vec<Statement<'a>>,
    },
    Block(Vec<Statement<'a>>),

    Throw(Expr<'a>),
    TryCatchFinally {
        body: Vec<Statement<'a>>,
        err_name: &'a str,
        handler: Vec<Statement<'a>>,
        finally: Vec<Statement<'a>>,
    },

    Import {
        module: Vec<String>,
        alias: &'a str,
    },
    RecordDecl {
        name: &'a str,
        fields: Vec<Param>,
    },
}

/// 带位置的语句
#[derive(Debug, Clone, PartialEq)]
pub struct Statement<'a> {
    pub kind: StatementKind<'a>,
    pub line: usize,
    pub col: usize,
}

impl<'a> Statement<'a> {
    pub fn new(kind: StatementKind<'a>, line: usize, col: usize) -> Self {
        Statement { kind, line, col }
    }
}

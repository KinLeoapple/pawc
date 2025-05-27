use pest_derive::Parser;

use crate::ast::ast::*;

#[derive(Parser)]
#[grammar = "src/grammar.pest"]
pub struct PawScriptParser;

// --- AST 构建错误类型 ---
#[derive(Debug, Clone, PartialEq)]
pub struct AstBuilderError(pub String);

impl std::fmt::Display for AstBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AST Builder Error: {}", self.0)
    }
}

impl std::error::Error for AstBuilderError {}
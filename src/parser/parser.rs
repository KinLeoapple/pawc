use crate::ast::ast::TopLevelItem;
use crate::parser::builder::build_toplevel_items::build_toplevel_items;
use pest::iterators::Pairs;
use pest_derive::Parser;

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

pub fn parse<'a>(pairs: Pairs<'a, Rule>) -> Result<Vec<TopLevelItem<'a>>, AstBuilderError> {
    let mut items = Vec::new();

    for pair in pairs {
        items.extend(build_toplevel_items(pair)?);
    }

    Ok(items)
}

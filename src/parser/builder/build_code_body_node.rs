use pest::iterators::Pair;
use crate::ast::ast::StatementNode;
use crate::parser::builder::build_statement_node::build_statement_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_code_body_node<'a>(pair: Pair<'a, Rule>) -> Result<Vec<StatementNode<'a>>, AstBuilderError> {
    // pair: code_body
    let mut statements = Vec::new();
    for stmt_pair in pair.into_inner() {
        // 只解析 statement
        if stmt_pair.as_rule() == Rule::statement {
            let stmt = build_statement_node(stmt_pair)?;
            statements.push(stmt);
        }
    }
    Ok(statements)
}
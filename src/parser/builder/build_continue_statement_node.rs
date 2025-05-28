use pest::iterators::Pair;
use crate::ast::ast::StatementNode;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_continue_statement_node<'a>(pair: Pair<'a, Rule>) -> Result<StatementNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    Ok(StatementNode::Continue { line, col })
}
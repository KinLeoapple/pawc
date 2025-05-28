use pest::iterators::Pair;
use crate::ast::ast::StatementNode;
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_return_statement_node<'a>(pair: Pair<'a, Rule>) -> Result<StatementNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner();

    // 可有可无 expression
    let expr = if let Some(expr_pair) = inner.next() {
        Some(build_expression_node(expr_pair)?)
    } else {
        None
    };

    Ok(StatementNode::Return {
        expr,
        line,
        col,
    })
}
use pest::iterators::Pair;
use crate::ast::ast::StatementNode;
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_return_statement_node<'a>(pair: Pair<'a, Rule>) -> Result<StatementNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner(); // 子项可能是空或一个 expression

    let expr_opt = inner.next()
        .map(|p| build_expression_node(p))
        .transpose()?; // None => return without expression

    Ok(StatementNode::Return {
        expr: expr_opt,
        line,
        col,
    })
}
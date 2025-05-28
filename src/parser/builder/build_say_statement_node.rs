use pest::iterators::Pair;
use crate::ast::ast::StatementNode;
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_say_statement_node<'a>(pair: Pair<'a, Rule>) -> Result<StatementNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner();

    // 跳过 KEYWORD_SAY，直接取 expression
    let expr_pair = inner.next().ok_or_else(|| AstBuilderError("say_statement: missing expression".into()))?;
    let expr = build_expression_node(expr_pair)?;

    Ok(StatementNode::Say {
        expr,
        line,
        col,
    })
}
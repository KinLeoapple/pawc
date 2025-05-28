use pest::iterators::Pair;
use crate::ast::ast::ExpressionNode;
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_ask_expression_node<'a>(pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    let mut inner = pair.into_inner();
    let _ask_kw = inner
        .next()
        .ok_or_else(|| AstBuilderError("ask_expression: missing 'ask' keyword".into()))?;
    let expr_pair = inner
        .next()
        .ok_or_else(|| AstBuilderError("ask_expression: missing prompt expression".into()))?;
    let prompt = build_expression_node(expr_pair)?;
    Ok(ExpressionNode::Ask { prompt: Box::new(prompt) })
}
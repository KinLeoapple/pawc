use pest::iterators::Pair;
use crate::ast::ast::{ExpressionNode, LiteralNode};
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_boolean_literal_node<'a>(pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    let val = match pair.as_str() {
        "true" => true,
        "false" => false,
        _ => return Err(AstBuilderError(format!("Invalid boolean literal: {}", pair.as_str()))),
    };
    Ok(ExpressionNode::Literal(LiteralNode::Bool(val)))
}
use pest::iterators::Pair;
use crate::ast::ast::{ExpressionNode, LiteralNode};
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_integer_literal_node<'a>(pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    let s = pair.as_str();
    match s.parse::<i64>() {
        Ok(val) => Ok(ExpressionNode::Literal(LiteralNode::Int(val))),
        Err(_) => Err(AstBuilderError(format!("Invalid integer literal: {}", s))),
    }
}

use pest::iterators::Pair;
use crate::ast::ast::{ExpressionNode, LiteralNode};
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_long_literal_node<'a>(pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    let s = pair.as_str();
    let numeric_part = &s[..s.len()-1];
    match numeric_part.parse::<i64>() {
        Ok(val) => Ok(ExpressionNode::Literal(LiteralNode::Long(val))),
        Err(_) => Err(AstBuilderError(format!("Invalid long literal: {}", s))),
    }
}
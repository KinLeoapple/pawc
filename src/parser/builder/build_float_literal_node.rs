use pest::iterators::Pair;
use crate::ast::ast::{ExpressionNode, LiteralNode};
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_float_literal_node<'a>(pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    let s = pair.as_str();
    let s = if s.ends_with('F') || s.ends_with('f') {
        &s[..s.len()-1]
    } else {
        s
    };
    match s.parse::<f32>() {
        Ok(val) => Ok(ExpressionNode::Literal(LiteralNode::Float(val))),
        Err(_) => Err(AstBuilderError(format!("Invalid float literal: {}", pair.as_str()))),
    }
}
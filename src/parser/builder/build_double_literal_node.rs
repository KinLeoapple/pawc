use pest::iterators::Pair;
use crate::ast::ast::{ExpressionNode, LiteralNode};
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_double_literal_node<'a>(pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    let s = pair.as_str();
    let s = if s.ends_with('D') || s.ends_with('d') {
        &s[..s.len()-1]
    } else {
        s
    };
    match s.parse::<f64>() {
        Ok(val) => Ok(ExpressionNode::Literal(LiteralNode::Double(val))),
        Err(_) => Err(AstBuilderError(format!("Invalid double literal: {}", pair.as_str()))),
    }
}
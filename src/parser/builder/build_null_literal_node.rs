use pest::iterators::Pair;
use crate::ast::ast::{ExpressionNode, LiteralNode};
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_null_literal_node<'a>(_pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    Ok(ExpressionNode::Literal(LiteralNode::Nopaw))
}
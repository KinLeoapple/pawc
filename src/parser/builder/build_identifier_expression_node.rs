use pest::iterators::Pair;
use crate::ast::ast::{ExpressionNode, IdentifierNode};
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_identifier_expression_node<'a>(pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    Ok(ExpressionNode::Identifier(IdentifierNode {
        name: pair.as_str(),
        line,
        col,
    }))
}
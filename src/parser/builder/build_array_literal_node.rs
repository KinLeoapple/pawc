use pest::iterators::Pair;
use crate::ast::ast::ExpressionNode;
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_array_literal_node<'a>(pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    let mut elements = Vec::new();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expression => {
                elements.push(build_expression_node(inner)?);
            }
            Rule::EOI => {}
            _ => {}
        }
    }
    Ok(ExpressionNode::ArrayLiteral(elements))
}
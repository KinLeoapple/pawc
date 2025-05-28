use pest::iterators::Pair;
use crate::ast::ast::{BinaryOp, ExpressionNode};
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_additive_expression_node<'a>(pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    let mut inner = pair.into_inner();
    let mut expr = build_expression_node(inner.next().unwrap())?;
    while let Some(op_pair) = inner.next() {
        let op_span = op_pair.as_span();
        let (line, col) = op_span.start_pos().line_col();
        let op = match op_pair.as_str() {
            "+" => BinaryOp::Add,
            "-" => BinaryOp::Sub,
            _   => return Err(AstBuilderError(format!("Unknown additive operator: {}", op_pair.as_str()))),
        };
        let right = build_expression_node(inner.next().unwrap())?;
        expr = ExpressionNode::BinaryOp {
            left: Box::new(expr),
            op,
            right: Box::new(right),
            line,
            col,
        };
    }
    Ok(expr)
}
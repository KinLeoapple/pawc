use pest::iterators::Pair;
use crate::ast::ast::{BinaryOp, ExpressionNode};
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_logical_and_expression_node<'a>(pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    let mut inner = pair.into_inner();
    // 第一个子项是左操作数
    let mut expr = build_expression_node(inner.next().unwrap())?;
    // 之后每一对（&&, equality_expression）
    while let Some(op_pair) = inner.next() {
        // op_pair 是 logical_and_operator
        let op_span = op_pair.as_span();
        let (line, col) = op_span.start_pos().line_col();
        let right = build_expression_node(inner.next().unwrap())?;
        expr = ExpressionNode::BinaryOp {
            left: Box::new(expr),
            op: BinaryOp::And,
            right: Box::new(right),
            line,
            col,
        };
    }
    Ok(expr)
}
use pest::iterators::Pair;
use crate::ast::ast::{BinaryOp, ExpressionNode};
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_logical_or_expression_node<'a>(pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    // pair: logical_or_expression
    let mut inner = pair.into_inner();
    // 第一个是左侧表达式
    let mut expr = build_expression_node(inner.next().unwrap())?;
    // 后续每一组（||, expr）
    while let Some(op_pair) = inner.next() {
        // op_pair 是 logical_or_operator
        let op_span = op_pair.as_span();
        let (line, col) = op_span.start_pos().line_col();
        // 下一个一定是 logical_and_expression
        let right_expr = build_expression_node(inner.next().unwrap())?;
        expr = ExpressionNode::BinaryOp {
            left: Box::new(expr),
            op: BinaryOp::Or,
            right: Box::new(right_expr),
            line,
            col,
        }
    }
    Ok(expr)
}
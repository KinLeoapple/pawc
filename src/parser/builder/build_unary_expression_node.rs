use pest::iterators::Pair;
use crate::ast::ast::{ExpressionNode, UnaryOp};
use crate::parser::builder::build_cast_expression_node::build_cast_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_unary_expression_node<'a>(pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();

    match first.as_rule() {
        Rule::unary_operator => {
            // 一元运算符
            let op_str = first.as_str();
            let (line, col) = first.as_span().start_pos().line_col();
            let op = match op_str {
                "-" => UnaryOp::Negate,
                "!" => UnaryOp::Not,
                _ => return Err(AstBuilderError(format!("Unknown unary operator: {}", op_str))),
            };
            let expr = build_unary_expression_node(inner.next().unwrap())?;
            Ok(ExpressionNode::UnaryOp {
                op,
                expr: Box::new(expr),
                line,
                col,
            })
        }
        _ => build_cast_expression_node(first),
    }
}
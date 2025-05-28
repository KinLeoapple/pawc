use pest::iterators::Pair;
use crate::ast::ast::{BinaryOp, ExpressionNode};
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::builder::build_type_name_node::build_type_name_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_cast_expression_node<'a>(pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    let mut inner = pair.into_inner();

    // 左侧：await_expression
    let left_pair = inner.next().unwrap();
    let expr = build_expression_node(left_pair)?;

    // 可选: as type_name
    if let Some(as_pair) = inner.next() {
        // as_pair: KEYWORD_AS
        let (line, col) = as_pair.as_span().start_pos().line_col();
        let type_pair = inner.next().ok_or_else(|| AstBuilderError("Missing type name after 'as'".into()))?;
        let type_name = build_type_name_node(type_pair)?;
        Ok(ExpressionNode::BinaryOp {
            left: Box::new(expr),
            op: BinaryOp::As,
            right: Box::new(ExpressionNode::TypeName(type_name)),
            line,
            col,
        })
    } else {
        Ok(expr)
    }
}
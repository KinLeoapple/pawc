use pest::iterators::Pair;
use crate::ast::ast::ExpressionNode;
use crate::parser::builder::build_postfix_expression_node::build_postfix_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_await_expression_node<'a>(pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();

    match first.as_rule() {
        Rule::KEYWORD_AWAIT => {
            // 结构是 KEYWORD_AWAIT ~ postfix_expression
            let expr_pair = inner.next().ok_or_else(|| AstBuilderError("await_expression: missing postfix_expression".into()))?;
            let (line, col) = first.as_span().start_pos().line_col();
            let expr = build_postfix_expression_node(expr_pair)?;
            Ok(ExpressionNode::Await {
                expr: Box::new(expr),
                line,
                col,
            })
        }
        // fallback: 直接是 postfix_expression
        Rule::postfix_expression => build_postfix_expression_node(first),
        _ => Err(AstBuilderError(format!("Unknown await_expression rule: {:?}", first.as_rule()))),
    }
}
use pest::iterators::Pair;
use crate::ast::ast::{ExpressionNode, LiteralNode, StringInterpolationNode, StringPartNode};
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_string_literal_node<'a>(pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    let pair_span = pair.as_span();
    let pair_offset = pair_span.start();
    let (line, col) = pair_span.start_pos().line_col();
    let original_str = pair_span.as_str(); // 拿到整个字符串的原始片段
    let mut parts = Vec::new();

    if let Some(content_pair) = pair.into_inner().find(|p| p.as_rule() == Rule::string_content) {
        let mut text_start = None;
        let mut text_end = None;

        for sub in content_pair.clone().into_inner() {
            match sub.as_rule() {
                Rule::string_interpolation => {
                    if let Some(start) = text_start {
                        let text = &original_str[start..text_end.unwrap()];
                        parts.push(StringPartNode::Text(text));
                        text_start = None;
                        text_end = None;
                    }

                    let expr_pair = sub.into_inner().next().unwrap();
                    let expr = build_expression_node(expr_pair)?;
                    parts.push(StringPartNode::Expr(expr));
                }
                _ => {
                    let span = sub.as_span();
                    let start = span.start() - pair_offset;
                    let end = span.end() - pair_offset;

                    if text_start.is_none() {
                        text_start = Some(start);
                    }
                    text_end = Some(end);
                }
            }
        }

        if let Some(start) = text_start {
            let text = &original_str[start..text_end.unwrap()];
            parts.push(StringPartNode::Text(text));
        }
    }

    Ok(ExpressionNode::Literal(LiteralNode::StringLiteral(
        StringInterpolationNode {
            parts,
            line,
            col,
        },
    )))
}
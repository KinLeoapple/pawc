use pest::iterators::Pair;
use crate::ast::ast::{ExpressionNode, LiteralNode, StringInterpolationNode, StringPartNode};
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_string_literal_node<'a>(pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut parts = Vec::new();
    let mut string_content = None;

    // string_literal = "\"" ~ string_content ~ "\""
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::string_content {
            string_content = Some(inner);
            break;
        }
    }

    if let Some(content_pair) = string_content {
        let mut last_end = content_pair.as_span().start();
        let orig_str = content_pair.as_span().as_str();
        for sub in content_pair.clone().into_inner() {
            let sub_span = sub.as_span();
            match sub.as_rule() {
                Rule::string_interpolation => {
                    // 之前的非插值文本
                    let start = sub.as_span().start();
                    let pre_text = &orig_str[(last_end - content_pair.as_span().start()) .. (start - content_pair.as_span().start())];
                    if !pre_text.is_empty() {
                        parts.push(StringPartNode::Text(pre_text));
                    }
                    // 插值
                    let expr_pair = sub.into_inner().next().unwrap();
                    let expr = build_expression_node(expr_pair)?;
                    parts.push(StringPartNode::Expr(expr));
                    last_end = sub_span.end();
                }
                _ => {}
            }
        }
        // 结尾剩余文本
        let tail_text = &orig_str[(last_end - content_pair.as_span().start())..];
        if !tail_text.is_empty() {
            parts.push(StringPartNode::Text(tail_text));
        }
    } else {
        // 纯空字符串
        parts.push(StringPartNode::Text(""));
    }

    Ok(ExpressionNode::Literal(LiteralNode::StringLiteral(StringInterpolationNode {
        parts,
        line,
        col,
    })))
}
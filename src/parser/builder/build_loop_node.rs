use pest::iterators::Pair;
use crate::ast::ast::{IdentifierNode, LoopNode, StatementNode};
use crate::parser::builder::build_code_body_node::build_code_body_node;
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_loop_node<'a>(pair: Pair<'a, Rule>) -> Result<StatementNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner();

    let variant_pair = inner.next().ok_or_else(|| AstBuilderError("loop_statement: missing variant".into()))?;
    match variant_pair.as_rule() {
        Rule::loop_for_in_variant => {
            let mut var_inner = variant_pair.into_inner();
            let id_pair = var_inner.next().ok_or_else(|| AstBuilderError("loop_for_in_variant: missing identifier".into()))?;
            let (id_line, id_col) = id_pair.as_span().start_pos().line_col();
            let var = IdentifierNode {
                name: id_pair.as_str(),
                line: id_line,
                col: id_col,
            };
            // KEYWORD_IN 已被 pest 吃掉
            let expr_pair = var_inner.next().ok_or_else(|| AstBuilderError("loop_for_in_variant: missing loop expr".into()))?;
            match expr_pair.as_rule() {
                Rule::loop_range_expression => {
                    let mut range_inner = expr_pair.into_inner();
                    let start_expr_pair = range_inner.next().ok_or_else(|| AstBuilderError("loop_range_expression: missing start".into()))?;
                    let end_expr_pair = range_inner.next().ok_or_else(|| AstBuilderError("loop_range_expression: missing end".into()))?;
                    let start = build_expression_node(start_expr_pair)?;
                    let end = build_expression_node(end_expr_pair)?;
                    let body_pair = var_inner.next().ok_or_else(|| AstBuilderError("loop_for_in_variant: missing body".into()))?;
                    let body = build_code_body_node(body_pair)?;
                    Ok(StatementNode::Loop(LoopNode::Range {
                        var,
                        start,
                        end,
                        body,
                        line,
                        col,
                    }))
                }
                Rule::loop_iterable_expression | Rule::expression => {
                    let iterable = build_expression_node(expr_pair)?;
                    let body_pair = var_inner.next().ok_or_else(|| AstBuilderError("loop_for_in_variant: missing body".into()))?;
                    let body = build_code_body_node(body_pair)?;
                    Ok(StatementNode::Loop(LoopNode::Iterable {
                        var,
                        iterable,
                        body,
                        line,
                        col,
                    }))
                }
                _ => Err(AstBuilderError("loop_for_in_variant: unknown expr type".into()))
            }
        }
        Rule::loop_conditional_variant => {
            let mut cond_inner = variant_pair.into_inner();
            let cond_pair = cond_inner.next().ok_or_else(|| AstBuilderError("loop_conditional_variant: missing cond".into()))?;
            let cond = build_expression_node(cond_pair)?;
            let body_pair = cond_inner.next().ok_or_else(|| AstBuilderError("loop_conditional_variant: missing body".into()))?;
            let body = build_code_body_node(body_pair)?;
            Ok(StatementNode::Loop(LoopNode::While {
                cond,
                body,
                line,
                col,
            }))
        }
        Rule::loop_infinite_variant => {
            let body_pair = variant_pair.into_inner().next().ok_or_else(|| AstBuilderError("loop_infinite_variant: missing body".into()))?;
            let body = build_code_body_node(body_pair)?;
            Ok(StatementNode::Loop(LoopNode::Infinite {
                body,
                line,
                col,
            }))
        }
        _ => Err(AstBuilderError(format!("Unknown loop variant: {:?}", variant_pair.as_rule())))
    }
}
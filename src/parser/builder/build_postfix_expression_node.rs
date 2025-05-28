use pest::iterators::Pair;
use crate::ast::ast::{ExpressionNode, IdentifierNode};
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::builder::build_primary_expression_node::build_primary_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_postfix_expression_node<'a>(pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    let mut inner = pair.into_inner();
    // 起点：primary_expression
    let mut expr = build_primary_expression_node(inner.next().unwrap())?;

    for postfix in inner {
        match postfix.as_rule() {
            Rule::member_access => {
                // 结构：'.' identifier
                let id_pair = postfix.into_inner().next().unwrap();
                let (line, col) = id_pair.as_span().start_pos().line_col();
                expr = ExpressionNode::MemberAccess {
                    target: Box::new(expr),
                    member: IdentifierNode {
                        name: id_pair.as_str(),
                        line,
                        col,
                    },
                    line,
                    col,
                };
            }
            Rule::array_index => {
                let idx_pair = postfix.into_inner().next().unwrap();
                let (line, col) = idx_pair.as_span().start_pos().line_col();
                let index_expr = build_expression_node(idx_pair)?;
                expr = ExpressionNode::ArrayAccess {
                    array: Box::new(expr),
                    index: Box::new(index_expr),
                    line,
                    col,
                };
            }
            Rule::function_call => {
                let (line, col) = postfix.as_span().start_pos().line_col();
                let arg_list_pair = postfix.into_inner().next();
                let mut args = Vec::new();
                if let Some(args_pair) = arg_list_pair {
                    for arg in args_pair.into_inner() {
                        if arg.as_rule() == Rule::expression {
                            args.push(build_expression_node(arg)?);
                        }
                    }
                }
                expr = ExpressionNode::FunctionCall {
                    callee: Box::new(expr),
                    args,
                    line,
                    col,
                };
            }
            Rule::length_access => {
                let (line, col) = postfix.as_span().start_pos().line_col();
                expr = ExpressionNode::LengthAccess {
                    target: Box::new(expr),
                    line,
                    col,
                };
            }
            _ => return Err(AstBuilderError(format!("Unknown postfix rule: {:?}", postfix.as_rule()))),
        }
    }
    Ok(expr)
}
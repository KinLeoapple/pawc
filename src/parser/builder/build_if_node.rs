use pest::iterators::Pair;
use crate::ast::ast::{IfNode, StatementNode};
use crate::parser::builder::build_code_body_node::build_code_body_node;
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_if_node<'a>(pair: Pair<'a, Rule>) -> Result<StatementNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner();

    // 主 if 条件
    let cond_pair = inner.next().ok_or_else(|| AstBuilderError("if_statement: missing condition".into()))?;
    let cond = build_expression_node(cond_pair)?;

    // 主 if 块
    let then_body_pair = inner.next().ok_or_else(|| AstBuilderError("if_statement: missing then body".into()))?;
    let then_block = build_code_body_node(then_body_pair)?;

    // 检查是否有 else/else if
    let mut else_block = None;
    let mut pending = inner.peek();

    if let Some(peek_pair) = pending {
        match peek_pair.as_rule() {
            Rule::KEYWORD_ELSE => {
                inner.next(); // consume ELSE

                // 判断是不是 else if
                if let Some(next_pair) = inner.peek() {
                    if next_pair.as_rule() == Rule::KEYWORD_IF {
                        // else if ... （这里只实现一层嵌套，如果想无限嵌套可递归）
                        inner.next(); // consume IF
                        // 解析 else if 分支
                        let else_if_cond_pair = inner.next().ok_or_else(|| AstBuilderError("if_statement: missing else-if condition".into()))?;
                        let else_if_cond = build_expression_node(else_if_cond_pair)?;
                        let else_if_body_pair = inner.next().ok_or_else(|| AstBuilderError("if_statement: missing else-if body".into()))?;
                        let else_if_block = build_code_body_node(else_if_body_pair)?;
                        // 用递归方式，把 else-if 分支作为 else_block 塞进 IfNode
                        let else_if_node = IfNode {
                            cond: else_if_cond,
                            then_block: else_if_block,
                            else_block: None, // 如果要支持 else if 链式递归可递归下去
                            line,
                            col,
                        };
                        else_block = Some(vec![StatementNode::If(else_if_node)]);
                    } else {
                        // 普通 else
                        let else_body_pair = inner.next().ok_or_else(|| AstBuilderError("if_statement: missing else body".into()))?;
                        let else_body = build_code_body_node(else_body_pair)?;
                        else_block = Some(else_body);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(StatementNode::If(IfNode {
        cond,
        then_block,
        else_block,
        line,
        col,
    }))
}
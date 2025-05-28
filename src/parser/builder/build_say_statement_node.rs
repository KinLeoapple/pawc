use crate::ast::ast::StatementNode;
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};
use pest::iterators::Pair;

pub fn build_say_statement_node<'a>(pair: Pair<'a, Rule>, ) -> Result<StatementNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner();

    // 先尝试取第一个子节点
    let first = inner
        .next()
        .ok_or_else(|| AstBuilderError("say_statement: missing parts".into()))?;

    // 如果它是 KEYWORD_SAY，就跳过，再取下一个作为真正的 expression
    let expr_pair = if first.as_rule() == Rule::KEYWORD_SAY {
        inner
            .next()
            .ok_or_else(|| AstBuilderError("say_statement: missing expression".into()))?
    } else {
        // 否则，就把它当成 expression
        first
    };

    // 构建表达式
    let expr = build_expression_node(expr_pair)?;

    Ok(StatementNode::Say {
        expr,
        line,
        col,
    })
}

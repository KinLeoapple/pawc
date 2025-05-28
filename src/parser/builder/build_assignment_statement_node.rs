use pest::iterators::Pair;
use crate::ast::ast::{IdentifierNode, StatementNode};
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_assignment_statement_node<'a>(pair: Pair<'a, Rule>) -> Result<StatementNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner();

    // 左侧变量名
    let id_pair = inner.next().ok_or_else(|| AstBuilderError("assignment_statement: missing identifier".into()))?;
    let (id_line, id_col) = id_pair.as_span().start_pos().line_col();
    let target = IdentifierNode {
        name: id_pair.as_str(),
        line: id_line,
        col: id_col,
    };

    // 右侧表达式
    let expr_pair = inner.next().ok_or_else(|| AstBuilderError("assignment_statement: missing expr".into()))?;
    let expr = build_expression_node(expr_pair)?;

    Ok(StatementNode::Assign {
        target,
        expr,
        line,
        col,
    })
}
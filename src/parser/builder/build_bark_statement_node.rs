use crate::ast::ast::StatementNode;
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};
use pest::iterators::Pair;

pub fn build_bark_statement_node<'a>(
    pair: Pair<'a, Rule>,
) -> Result<StatementNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner();
    
    let _bark_kw = inner
        .next()
        .ok_or_else(|| AstBuilderError("bark_statement: missing 'bark' keyword".into()))?;
    
    let expr_pair = inner
        .next()
        .ok_or_else(|| AstBuilderError("bark_statement: missing expression".into()))?;
    let expr = build_expression_node(expr_pair)?;

    Ok(StatementNode::Bark { expr, line, col })
}

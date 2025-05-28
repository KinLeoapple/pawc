use pest::iterators::Pair;
use crate::ast::ast::{ErrorHandlingNode, IdentifierNode, StatementNode};
use crate::parser::builder::build_code_body_node::build_code_body_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_error_handling_node<'a>(pair: Pair<'a, Rule>) -> Result<StatementNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner();

    // sniff_body
    let sniff_body_pair = inner.next().ok_or_else(|| AstBuilderError("error_handling_statement: missing sniff body".into()))?;
    let sniff_body = build_code_body_node(sniff_body_pair)?;

    // snatch_clauses
    let mut snatch_clauses = Vec::new();
    let mut lastly_body = None;

    for next in inner {
        match next.as_rule() {
            Rule::snatch_clause => {
                let mut snatch_inner = next.into_inner();
                // KEYWORD_SNATCH ~ "(" ~ identifier ~ ")" ~ code_body
                let id_pair = snatch_inner.next().ok_or_else(|| AstBuilderError("snatch_clause: missing identifier".into()))?;
                let (id_line, id_col) = id_pair.as_span().start_pos().line_col();
                let ident = IdentifierNode {
                    name: id_pair.as_str(),
                    line: id_line,
                    col: id_col,
                };
                let body_pair = snatch_inner.next().ok_or_else(|| AstBuilderError("snatch_clause: missing code body".into()))?;
                let body = build_code_body_node(body_pair)?;
                snatch_clauses.push((ident, body));
            }
            Rule::lastly_clause => {
                let mut lastly_inner = next.into_inner();
                let body_pair = lastly_inner.next().ok_or_else(|| AstBuilderError("lastly_clause: missing code body".into()))?;
                let body = build_code_body_node(body_pair)?;
                lastly_body = Some(body);
            }
            _ => {} // 忽略其它
        }
    }

    Ok(StatementNode::ErrorHandling(ErrorHandlingNode {
        sniff_body,
        snatch_clauses,
        lastly_body,
        line,
        col,
    }))
}
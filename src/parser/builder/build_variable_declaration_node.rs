use std::iter::Peekable;
use pest::iterators::{Pair, Pairs};
use crate::ast::ast::{IdentifierNode, StatementNode};
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::builder::build_type_name_node::build_type_name_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_variable_declaration_node<'a>(
    pair: Pair<'a, Rule>,
) -> Result<StatementNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut inner: Peekable<Pairs<'a, Rule>> = pair.into_inner().peekable();
    
    let first = inner
        .next()
        .ok_or_else(|| AstBuilderError("variable_declaration: missing 'let'".into()))?;
    if first.as_rule() != Rule::KEYWORD_LET {
        return Err(AstBuilderError("variable_declaration: expected 'let'".into()));
    }
    
    let name_pair = inner
        .next()
        .ok_or_else(|| AstBuilderError("variable_declaration: missing identifier".into()))?;
    let (name_line, name_col) = name_pair.as_span().start_pos().line_col();
    let name = IdentifierNode {
        name: name_pair.as_str(),
        line: name_line,
        col: name_col,
    };
    
    let (type_name, expr_pair) = {
        let next_pair = inner
            .next()
            .ok_or_else(|| AstBuilderError("variable_declaration: missing expr or type".into()))?;
        if next_pair.as_rule() == Rule::type_name {
            let tn = build_type_name_node(next_pair)?;
            let ep = inner
                .next()
                .ok_or_else(|| AstBuilderError("variable_declaration: missing expr".into()))?;
            (tn, ep)
        } else {
            return Err(AstBuilderError("variable_declaration: missing type annotation".into()));
        }
    };
    
    let expr = build_expression_node(expr_pair)?;

    Ok(StatementNode::Let {
        name,
        type_name,
        expr,
        line,
        col,
    })
}

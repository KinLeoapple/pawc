use pest::iterators::Pair;
use crate::ast::ast::{IdentifierNode, StatementNode};
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::builder::build_type_name_node::build_type_name_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_variable_declaration_node<'a>(pair: Pair<'a, Rule>) -> Result<StatementNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner();

    // identifier
    let name_pair = inner.next().ok_or_else(|| AstBuilderError("variable_declaration: missing identifier".into()))?;
    let (name_line, name_col) = name_pair.as_span().start_pos().line_col();
    let name = IdentifierNode {
        name: name_pair.as_str(),
        line: name_line,
        col: name_col,
    };

    // type_name
    let type_pair = inner.next().ok_or_else(|| AstBuilderError("variable_declaration: missing type".into()))?;
    let type_name = build_type_name_node(type_pair)?;

    // expression
    let expr_pair = inner.next().ok_or_else(|| AstBuilderError("variable_declaration: missing expr".into()))?;
    let expr = build_expression_node(expr_pair)?;

    Ok(StatementNode::Let {
        name,
        type_name,
        expr,
        line,
        col,
    })
}

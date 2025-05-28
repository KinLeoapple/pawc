use pest::iterators::Pair;
use crate::ast::ast::TypeNameNode;
use crate::parser::builder::build_core_type_name_node::build_core_type_name_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_type_name_node<'a>(pair: Pair<'a, Rule>) -> Result<TypeNameNode<'a>, AstBuilderError> {
    // pair: type_name
    let (start_line, start_col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner();

    // 1. 核心类型（simple 或 generic）
    let core_pair = inner.next().ok_or_else(|| AstBuilderError("type_name: missing core".into()))?;
    let core = build_core_type_name_node(core_pair)?;

    // 2. 可选类型 ?（optional_marker，可能没有）
    let is_optional = inner.peek().map(|p| p.as_rule() == Rule::optional_marker).unwrap_or(false);

    Ok(TypeNameNode {
        core,
        is_optional,
        line: start_line,
        col: start_col,
    })
}
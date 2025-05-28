use pest::iterators::Pair;
use crate::ast::ast::TypeNameNode;
use crate::parser::builder::build_core_type_name_node::build_core_type_name_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_type_name_node<'a>(pair: Pair<'a, Rule>) -> Result<TypeNameNode<'a>, AstBuilderError> {
    let span = pair.as_span();
    let (line, col) = span.start_pos().line_col();

    // type_name = { core_type ~ optional_marker? }
    let mut inner = pair.into_inner();
    // 一定有 core_type
    let core_pair = inner
        .next()
        .ok_or_else(|| AstBuilderError("type_name: missing core_type".into()))?;
    let core = build_core_type_name_node(core_pair)?;

    // 可选的 optional_marker ("?")
    let is_optional = match inner.next() {
        Some(opt) if opt.as_rule() == Rule::optional_marker => true,
        _ => false,
    };

    Ok(TypeNameNode {
        core,
        is_optional,
        line,
        col,
    })
}
use pest::iterators::Pair;
use crate::ast::ast::{IdentifierNode, ProtocolDefinitionNode};
use crate::parser::builder::build_protocol_method_signature_node::build_protocol_method_signature_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_protocol_definition_node<'a>(pair: Pair<'a, Rule>) -> Result<ProtocolDefinitionNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner();

    // 跳过 KEYWORD_TAIL
    let name_pair = inner.next().ok_or_else(|| AstBuilderError("protocol_definition: missing name".into()))?;
    let (name_line, name_col) = name_pair.as_span().start_pos().line_col();
    let name = IdentifierNode {
        name: name_pair.as_str(),
        line: name_line,
        col: name_col,
    };

    let mut methods = Vec::new();
    for method_pair in inner {
        if method_pair.as_rule() == Rule::protocol_method_signature {
            let sig = build_protocol_method_signature_node(method_pair)?;
            methods.push(sig);
        }
    }

    Ok(ProtocolDefinitionNode {
        name,
        methods,
        line,
        col,
    })
}
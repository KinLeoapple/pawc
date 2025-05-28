use pest::iterators::Pair;
use crate::ast::ast::{IdentifierNode, ProtocolDefinitionNode};
use crate::parser::builder::build_protocol_method_signature_node::build_protocol_method_signature_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_protocol_definition_node<'a>(pair: Pair<'a, Rule>) -> Result<ProtocolDefinitionNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner();

    // 1. 消耗 KEYWORD_TAIL
    let keyword_pair = inner.next().ok_or_else(|| AstBuilderError("protocol_definition: expected KEYWORD_TAIL".into()))?;
    if keyword_pair.as_rule() != Rule::KEYWORD_TAIL {
        return Err(AstBuilderError(format!(
            "protocol_definition: expected KEYWORD_TAIL, found {:?}",
            keyword_pair.as_rule()
        )));
    }

    // 2. 获取协议名称
    let name_pair = inner.next().ok_or_else(|| AstBuilderError("protocol_definition: missing name".into()))?;
    if name_pair.as_rule() != Rule::identifier {
        return Err(AstBuilderError(format!(
            "protocol_definition: expected identifier for name, found {:?}",
            name_pair.as_rule()
        )));
    }
    let (name_line, name_col) = name_pair.as_span().start_pos().line_col();
    let name = IdentifierNode {
        name: name_pair.as_str(),
        line: name_line,
        col: name_col,
    };

    let mut methods = Vec::new();
    // 迭代剩余的 inner pairs
    for method_pair_or_rbrace in inner {
        match method_pair_or_rbrace.as_rule() {
            Rule::protocol_method_signature => {
                let sig = build_protocol_method_signature_node(method_pair_or_rbrace)?;
                methods.push(sig);
            }
            _ => {
                // 可以选择性地对非预期的 rule 报错或记录日志
                return Err(AstBuilderError(format!("protocol_definition: unexpected rule in body: {:?}", method_pair_or_rbrace.as_rule())));
            }
        }
    }

    Ok(ProtocolDefinitionNode {
        name,
        methods,
        line,
        col,
    })
}
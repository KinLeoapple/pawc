use pest::iterators::Pair;
use crate::ast::ast::{CoreTypeNameNode, FunctionSignatureNode, IdentifierNode, TypeNameNode};
use crate::parser::builder::build_type_name_node::build_type_name_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_protocol_method_signature_node<'a>(pair: Pair<'a, Rule>) -> Result<FunctionSignatureNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner();

    // async?
    let mut is_async = false;
    let mut next = inner.next().ok_or_else(|| AstBuilderError("protocol_method_signature: missing FUN".into()))?;
    if next.as_rule() == Rule::KEYWORD_ASYNC {
        is_async = true;
        next = inner.next().ok_or_else(|| AstBuilderError("protocol_method_signature: missing FUN".into()))?;
    }
    // 跳过 FUN
    let name_pair = inner.next().ok_or_else(|| AstBuilderError("protocol_method_signature: missing name".into()))?;
    let (name_line, name_col) = name_pair.as_span().start_pos().line_col();
    let name = IdentifierNode {
        name: name_pair.as_str(),
        line: name_line,
        col: name_col,
    };

    // 跳过 (
    let mut params = Vec::new();
    let param_list_pair = inner.next().ok_or_else(|| AstBuilderError("protocol_method_signature: missing param_list".into()))?;
    for param_pair in param_list_pair.into_inner() {
        let mut param_inner = param_pair.into_inner();
        let id_pair = param_inner.next().ok_or_else(|| AstBuilderError("protocol_method_signature: param missing name".into()))?;
        let (id_line, id_col) = id_pair.as_span().start_pos().line_col();
        let param_id = IdentifierNode {
            name: id_pair.as_str(),
            line: id_line,
            col: id_col,
        };
        let type_pair = param_inner.next().ok_or_else(|| AstBuilderError("protocol_method_signature: param missing type".into()))?;
        let param_ty = build_type_name_node(type_pair)?;
        params.push((param_id, param_ty));
    }

    // 可选返回类型
    let mut return_type = None;
    if let Some(type_pair) = inner.next() {
        return_type = Some(build_type_name_node(type_pair)?);
    }
    let return_type = return_type.unwrap_or_else(|| TypeNameNode {
        core: CoreTypeNameNode::Simple(IdentifierNode {
            name: "Void",
            line,
            col,
        }),
        is_optional: false,
        line,
        col,
    });

    Ok(FunctionSignatureNode {
        is_async,
        name,
        params,
        return_type,
        line,
        col,
    })
}

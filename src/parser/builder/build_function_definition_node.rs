use pest::iterators::Pair;
use crate::ast::ast::{CoreTypeNameNode, FunctionDefinitionNode, IdentifierNode, TypeNameNode};
use crate::parser::builder::build_code_body_node::build_code_body_node;
use crate::parser::builder::build_type_name_node::build_type_name_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_function_definition_node<'a>(pair: Pair<'a, Rule>) -> Result<FunctionDefinitionNode<'a>, AstBuilderError> {
    let (start_line, start_col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner();

    // async?
    let mut is_async = false;
    if inner.peek().map(|p| p.as_rule() == Rule::KEYWORD_ASYNC).unwrap_or(false) {
        is_async = true;
        inner.next();
    }

    // fun
    if inner.peek().map(|p| p.as_rule() == Rule::KEYWORD_FUN).unwrap_or(false) {
        inner.next();
    }

    // name
    let name_pair = inner.next().ok_or_else(|| AstBuilderError("Function missing name".into()))?;
    let (name_line, name_col) = name_pair.as_span().start_pos().line_col();
    let name = IdentifierNode {
        name: name_pair.as_str(),
        line: name_line,
        col: name_col,
    };

    // params
    let params_pair = inner.next().ok_or_else(|| AstBuilderError("Function missing param list".into()))?;
    let mut params = Vec::new();
    for param_pair in params_pair.into_inner() {
        let mut param_inner = param_pair.into_inner();
        let id_pair = param_inner.next().ok_or_else(|| AstBuilderError("Function param missing name".into()))?;
        let type_pair = param_inner.next().ok_or_else(|| AstBuilderError("Function param missing type".into()))?;
        params.push((
            IdentifierNode {
                name: id_pair.as_str(),
                line: id_pair.as_span().start_pos().line_col().0,
                col: id_pair.as_span().start_pos().line_col().1,
            },
            build_type_name_node(type_pair)?
        ));
    }

    // 返回类型 (type_name, 可选)
    let mut return_type = None;
    if let Some(next) = inner.peek() {
        if next.as_rule() == Rule::type_name {
            let type_pair = inner.next().unwrap();
            return_type = Some(build_type_name_node(type_pair)?);
        }
    }
    let return_type = return_type.unwrap_or(TypeNameNode {
        core: CoreTypeNameNode::Simple(IdentifierNode { name: "Void", line: start_line, col: start_col }),
        is_optional: false,
        line: start_line,
        col: start_col,
    });

    // body
    let body_pair = inner.next().ok_or_else(|| AstBuilderError("Function missing body".into()))?;
    let body = build_code_body_node(body_pair)?;

    Ok(FunctionDefinitionNode {
        is_async,
        name,
        params,
        return_type,
        body,
        line: start_line,
        col: start_col,
    })
}

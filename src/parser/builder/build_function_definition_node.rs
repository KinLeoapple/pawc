use crate::ast::ast::{CoreTypeNameNode, FunctionDefinitionNode, IdentifierNode, TypeNameNode};
use crate::parser::builder::build_statement_node::build_statement_node;
use crate::parser::builder::build_type_name_node::build_type_name_node;
use crate::parser::parser::{AstBuilderError, Rule};
use pest::iterators::Pair;

pub fn build_function_definition_node<'a>(pair: Pair<'a, Rule>) -> Result<FunctionDefinitionNode<'a>, AstBuilderError> {
    let (start_line, start_col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner();

    // 是否 async
    let mut is_async = false;
    if inner.peek().map(|p| p.as_rule() == Rule::KEYWORD_ASYNC).unwrap_or(false) {
        is_async = true;
        inner.next();
    }

    // 消耗 fun
    inner.next();

    // 函数名
    let name_pair = inner.next().ok_or_else(|| AstBuilderError("Function missing name".into()))?;
    let (name_line, name_col) = name_pair.as_span().start_pos().line_col();
    let name = IdentifierNode {
        name: name_pair.as_str(),
        line: name_line,
        col: name_col,
    };

    // 参数列表
    let mut params = Vec::new();
    let param_list = inner.next().ok_or_else(|| AstBuilderError("Function missing parameter list".into()))?;
    for param in param_list.into_inner() {
        let mut p_inner = param.into_inner();
        let id_pair = p_inner.next().unwrap();
        let type_pair = p_inner.next().unwrap();
        let (line, col) = id_pair.as_span().start_pos().line_col();
        let id = IdentifierNode {
            name: id_pair.as_str(),
            line,
            col,
        };
        let ty = build_type_name_node(type_pair)?;
        params.push((id, ty));
    }

    // 返回类型（默认 Void）
    let mut return_type = TypeNameNode {
        core: CoreTypeNameNode::Simple(IdentifierNode {
            name: "Void",
            line: start_line,
            col: start_col,
        }),
        is_optional: false,
        line: start_line,
        col: start_col,
    };

    if inner.peek().map(|p| p.as_rule() == Rule::type_name).unwrap_or(false) {
        let type_pair = inner.next().unwrap();
        return_type = build_type_name_node(type_pair)?;
    }

    // 函数体
    let body_pair = inner.next().ok_or_else(|| AstBuilderError("Function missing body".into()))?;
    let mut body = vec![];
    for stmt in body_pair.into_inner() {
        body.push(build_statement_node(stmt)?);
    }

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

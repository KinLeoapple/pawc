use pest::iterators::Pair;
use crate::ast::ast::{CoreTypeNameNode, IdentifierNode, RecordDefinitionNode, TypeNameNode};
use crate::parser::builder::build_function_definition_node::build_function_definition_node;
use crate::parser::builder::build_type_name_node::build_type_name_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_record_definition_node<'a>(pair: Pair<'a, Rule>) -> Result<RecordDefinitionNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner();

    // 消耗 "record"
    let keyword_pair = inner.next().ok_or_else(|| AstBuilderError("record_definition: expected KEYWORD_RECORD".into()))?;
    if keyword_pair.as_rule() != Rule::KEYWORD_RECORD {
        return Err(AstBuilderError(format!(
            "record_definition: expected KEYWORD_RECORD, found {:?}",
            keyword_pair.as_rule()
        )));
    }

    // 记录名称
    let name_pair = inner.next().ok_or_else(|| AstBuilderError("record_definition: missing name".into()))?;
    if name_pair.as_rule() != Rule::identifier {
        return Err(AstBuilderError(format!(
            "record_definition: expected identifier for name, found {:?}",
            name_pair.as_rule()
        )));
    }
    let (name_line, name_col) = name_pair.as_span().start_pos().line_col();
    let name = IdentifierNode {
        name: name_pair.as_str(),
        line: name_line,
        col: name_col,
    };

    // 可能的 implements 子句
    let mut implements = vec![];
    let mut fields = vec![];
    let mut methods = vec![];

    let mut next = inner.next().ok_or_else(|| AstBuilderError("record_definition: expected implements or body".into()))?;
    if next.as_rule() == Rule::record_implements_clause {
        for protocol_id in next.into_inner() {
            let (line, col) = protocol_id.as_span().start_pos().line_col();
            let id = IdentifierNode {
                name: protocol_id.as_str(),
                line,
                col,
            };
            implements.push(id);
        }
        next = inner.next().ok_or_else(|| AstBuilderError("record_definition: expected body after implements".into()))?;
    }

    // 处理成员
    let mut members = vec![next];
    members.extend(inner);

    for pair in members {
        match pair.as_rule() {
            Rule::record_member => {
                for member_pair in pair.into_inner() {
                    match member_pair.as_rule() {
                        Rule::record_field_def => {
                            let mut f_inner = member_pair.into_inner();
                            let id_pair = f_inner.next().ok_or_else(|| AstBuilderError("record_field_def: missing name".into()))?;
                            let (id_line, id_col) = id_pair.as_span().start_pos().line_col();
                            let id = IdentifierNode {
                                name: id_pair.as_str(),
                                line: id_line,
                                col: id_col,
                            };
                            let type_pair = f_inner.next().ok_or_else(|| AstBuilderError("record_field_def: missing type".into()))?;
                            let ty = build_type_name_node(type_pair)?;
                            fields.push((id, ty));
                        }
                        Rule::function_definition => {
                            methods.push(build_function_definition_node(member_pair)?);
                        }
                        _ => return Err(AstBuilderError(format!(
                            "record_member: unexpected inner rule: {:?}",
                            member_pair.as_rule()
                        ))),
                    }
                }
            }
            Rule::record_field_def => {
                let mut f_inner = pair.into_inner();
                let id_pair = f_inner.next().ok_or_else(|| AstBuilderError("record_field_def: missing name".into()))?;
                let (id_line, id_col) = id_pair.as_span().start_pos().line_col();
                let id = IdentifierNode {
                    name: id_pair.as_str(),
                    line: id_line,
                    col: id_col,
                };
                let type_pair = f_inner.next().ok_or_else(|| AstBuilderError("record_field_def: missing type".into()))?;
                let ty = build_type_name_node(type_pair)?;
                fields.push((id, ty));
            }
            Rule::function_definition => {
                methods.push(build_function_definition_node(pair)?);
            }
            _ => return Err(AstBuilderError(format!(
                "record_definition: unexpected rule in body: {:?}",
                pair.as_rule()
            ))),
        }
    }

    Ok(RecordDefinitionNode {
        name,
        implements,
        fields,
        methods,
        line,
        col,
    })
}
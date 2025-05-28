use pest::iterators::Pair;
use crate::ast::ast::{CoreTypeNameNode, IdentifierNode, RecordDefinitionNode, TypeNameNode};
use crate::parser::builder::build_function_definition_node::build_function_definition_node;
use crate::parser::builder::build_type_name_node::build_type_name_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_record_definition_node<'a>(pair: Pair<'a, Rule>) -> Result<RecordDefinitionNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner();

    // 跳过 KEYWORD_RECORD
    let name_pair = inner.next().ok_or_else(|| AstBuilderError("record_definition: missing name".into()))?;
    let (name_line, name_col) = name_pair.as_span().start_pos().line_col();
    let name = IdentifierNode {
        name: name_pair.as_str(),
        line: name_line,
        col: name_col,
    };

    // 可选 implements
    let mut implements = Vec::new();
    let peek = inner.peek().map(|p| p.as_rule());
    if peek == Some(Rule::record_implements_clause) {
        let clause = inner.next().unwrap();
        for type_name_pair in clause.into_inner() {
            // 只取实现名的 IdentifierNode
            match build_type_name_node(type_name_pair)? {
                TypeNameNode { core: CoreTypeNameNode::Simple(id), .. } => implements.push(id),
                _ => return Err(AstBuilderError("record_implements_clause: only simple type names allowed".into())),
            }
        }
    }

    // 跳过 "{"
    // 后面依次 record_member*
    let mut fields = Vec::new();
    let mut methods = Vec::new();
    for member_pair in inner {
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
                let method = build_function_definition_node(member_pair)?;
                methods.push(method);
            }
            _ => {}
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
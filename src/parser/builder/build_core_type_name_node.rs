use pest::iterators::Pair;
use crate::ast::ast::{CoreTypeNameNode, IdentifierNode};
use crate::parser::builder::build_type_name_node::build_type_name_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_core_type_name_node<'a>(pair: Pair<'a, Rule>) -> Result<CoreTypeNameNode<'a>, AstBuilderError> {
    match pair.as_rule() {
        // 非-silent 规则 core_type
        Rule::core_type => {
            let mut inner = pair.into_inner();
            let content = inner
                .next()
                .ok_or_else(|| AstBuilderError("core_type: missing content".into()))?;
            build_core_type_name_node(content)
        }
        // 泛型：Generic
        Rule::generic_type_def => {
            let mut inner = pair.into_inner();
            // 构造器名字 "Array" 或 "Optional"
            let cons_pair = inner.next().unwrap();
            let name_id = IdentifierNode {
                name: cons_pair.as_str(),
                line: cons_pair.as_span().start_pos().line_col().0,
                col: cons_pair.as_span().start_pos().line_col().1,
            };
            // 下一个是 type_name
            let type_arg_pair = inner.next().unwrap();
            let arg_tn = build_type_name_node(type_arg_pair)?;
            Ok(CoreTypeNameNode::Generic {
                name: name_id,
                type_args: vec![arg_tn],
            })
        }
        // 原子类型
        Rule::KEYWORD_INT
        | Rule::KEYWORD_LONG
        | Rule::KEYWORD_FLOAT
        | Rule::KEYWORD_DOUBLE
        | Rule::KEYWORD_BOOL
        | Rule::KEYWORD_CHAR
        | Rule::KEYWORD_STRING
        | Rule::KEYWORD_ANY => {
            let id = IdentifierNode {
                name: pair.as_str(),
                line: pair.as_span().start_pos().line_col().0,
                col: pair.as_span().start_pos().line_col().1,
            };
            Ok(CoreTypeNameNode::Simple(id))
        }
        // 用户自定义标识符
        Rule::identifier => {
            let id = IdentifierNode {
                name: pair.as_str(),
                line: pair.as_span().start_pos().line_col().0,
                col: pair.as_span().start_pos().line_col().1,
            };
            Ok(CoreTypeNameNode::Simple(id))
        }
        other => Err(AstBuilderError(format!("core_type: unexpected rule: {:?}", other))),
    }
}
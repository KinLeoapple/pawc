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
            // 这里只允许 Array<T>
            let mut inner = pair.into_inner();
            let cons_pair = inner.next().unwrap();
            let gen_kw = cons_pair.as_str();
            if gen_kw != "Array" {
                return Err(AstBuilderError(format!(
                    "unsupported generic type `{}`, use `T?` instead of `Optional<T>`",
                    gen_kw
                )));
            }
            let name_id = IdentifierNode {
                name: cons_pair.as_str(),
                line: cons_pair.as_span().start_pos().line_col().0,
                col: cons_pair.as_span().start_pos().line_col().1,
            };
            let arg_pair = inner.next().unwrap();
            let arg = build_type_name_node(arg_pair)?;
            Ok(CoreTypeNameNode::Generic {
                name: name_id,
                type_args: vec![arg],
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
use pest::iterators::Pair;
use crate::ast::ast::{CoreTypeNameNode, IdentifierNode};
use crate::parser::builder::build_type_name_node::build_type_name_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_core_type_name_node<'a>(pair: Pair<'a, Rule>) -> Result<CoreTypeNameNode<'a>, AstBuilderError> {
    // pair: core_type_name
    let mut inner = pair.clone().into_inner();
    let first = inner.next().ok_or_else(|| AstBuilderError("core_type_name: missing content".into()))?;

    match first.as_rule() {
        Rule::identifier => {
            // 可能是简单类型，也可能后面跟泛型参数
            if let Some(next) = inner.next() {
                // 一般 grammar.pest 会写成 identifier ~ "<" ~ type_name ~ ... ~ ">"
                // 此时 first 是类型名，next 应该是 "<"
                // 继续收集类型参数
                let mut type_args = Vec::new();
                for t in inner {
                    if t.as_rule() == Rule::type_name {
                        type_args.push(build_type_name_node(t)?);
                    }
                }
                let (line, col) = first.as_span().start_pos().line_col();
                Ok(CoreTypeNameNode::Generic {
                    name: IdentifierNode { name: first.as_str(), line, col },
                    type_args,
                })
            } else {
                let (line, col) = first.as_span().start_pos().line_col();
                Ok(CoreTypeNameNode::Simple(IdentifierNode { name: first.as_str(), line, col }))
            }
        }
        _ => Err(AstBuilderError(format!("Unknown core_type_name: {:?}", first.as_rule()))),
    }
}
use pest::iterators::Pair;
use crate::ast::ast::{ExpressionNode, IdentifierNode, RecordInitFieldNode, RecordInitNode};
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_record_init_node<'a>(pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner();

    // 1. 类型名
    let typename_pair = inner.next().ok_or_else(|| AstBuilderError("record_init: missing typename".into()))?;
    let (tline, tcol) = typename_pair.as_span().start_pos().line_col();
    let typename = IdentifierNode {
        name: typename_pair.as_str(),
        line: tline,
        col: tcol,
    };

    // 2. 字段们
    let mut fields = Vec::new();
    for item in inner {
        if item.as_rule() == Rule::record_init_field {
            let mut f_inner = item.into_inner();
            let field_name_pair = f_inner.next().ok_or_else(|| AstBuilderError("record_init_field: missing name".into()))?;
            let (fline, fcol) = field_name_pair.as_span().start_pos().line_col();
            let field_name = IdentifierNode {
                name: field_name_pair.as_str(),
                line: fline,
                col: fcol,
            };
            let expr_pair = f_inner.next().ok_or_else(|| AstBuilderError("record_init_field: missing expr".into()))?;
            let expr = build_expression_node(expr_pair)?;
            fields.push(RecordInitFieldNode {
                name: field_name,
                expr,
                line: fline,
                col: fcol,
            });
        }
    }

    Ok(ExpressionNode::RecordInit(RecordInitNode {
        typename,
        fields,
        line,
        col,
    }))
}
use pest::iterators::Pair;
use crate::ast::ast::{ExpressionNode, LiteralNode, StatementNode};
use crate::parser::builder::build_string_literal_node::build_string_literal_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_ask_statement_simple_node<'a>(pair: Pair<'a, Rule>) -> Result<StatementNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    // ask_statement_simple = ask_expression
    // ask_expression = KEYWORD_ASK ~ expression
    let mut inner = pair.into_inner();
    let ask_expr_pair = inner.next().ok_or_else(|| AstBuilderError("ask_statement_simple: missing expression".into()))?;

    // 只允许字符串/插值作为输入提示
    let prompt = match ask_expr_pair.as_rule() {
        Rule::string_literal => {
            if let ExpressionNode::Literal(LiteralNode::StringLiteral(s)) = build_string_literal_node(ask_expr_pair)? {
                s
            } else {
                return Err(AstBuilderError("ask_statement_simple: prompt is not a valid string literal".into()));
            }
        }
        _ => return Err(AstBuilderError("ask_statement_simple: prompt must be a string literal".into())),
    };

    Ok(StatementNode::Ask {
        prompt,
        target: None,
        line,
        col,
    })
}
use pest::iterators::Pair;
use crate::ast::ast::{ExpressionNode, IdentifierNode, LiteralNode, StatementNode};
use crate::parser::builder::build_string_literal_node::build_string_literal_node;
use crate::parser::builder::build_type_name_node::build_type_name_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_variable_input_assignment_node<'a>(pair: Pair<Rule>) -> Result<StatementNode, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner();

    // 跳过 let
    let name_pair = inner.next().ok_or_else(|| AstBuilderError("variable_input_assignment: missing identifier".into()))?;
    let (name_line, name_col) = name_pair.as_span().start_pos().line_col();
    let name = IdentifierNode {
        name: name_pair.as_str(),
        line: name_line,
        col: name_col,
    };

    // 可选类型
    let mut type_name = None;
    let peek = inner.peek().map(|p| p.as_rule());
    if peek == Some(Rule::type_name) {
        let type_pair = inner.next().unwrap();
        type_name = Some(build_type_name_node(type_pair)?);
    }

    // 跳过 <-
    if let Some(next) = inner.peek() {
        if next.as_str() == "<-" {
            inner.next();
        }
    }

    // ask_expression
    let ask_pair = inner.next().ok_or_else(|| AstBuilderError("variable_input_assignment: missing ask_expression".into()))?;
    // ask_expression = KEYWORD_ASK ~ expression
    let mut ask_inner = ask_pair.into_inner();
    let expr_pair = ask_inner.next().ok_or_else(|| AstBuilderError("ask_expression: missing expression".into()))?;

    // 只允许字符串/插值作为输入提示
    let prompt = match expr_pair.as_rule() {
        Rule::string_literal => {
            if let ExpressionNode::Literal(LiteralNode::StringLiteral(s)) = build_string_literal_node(expr_pair)? {
                s
            } else {
                return Err(AstBuilderError("ask_expression: prompt is not a valid string literal".into()));
            }
        }
        _ => return Err(AstBuilderError("ask_expression: prompt must be a string literal".into())),
    };

    Ok(StatementNode::Ask {
        prompt,
        target: Some((
            name,
            type_name.ok_or_else(|| AstBuilderError("variable_input_assignment: missing type".into()))?,
        )),
        line,
        col,
    })
}
use pest::iterators::Pair;
use crate::ast::ast::{ExpressionNode, LiteralNode};
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_character_literal_node<'a>(pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    let s = pair.as_str();
    // s 格式为 'c' 或 '\n' 或 '\u{XXXX}'
    if s.len() < 3 || !s.starts_with('\'') || !s.ends_with('\'') {
        return Err(AstBuilderError(format!("Invalid character literal: {}", s)));
    }
    let inner = &s[1..s.len() - 1];

    let ch = if inner.starts_with("\\u{") {
        // Unicode 转义
        let hex = &inner[3..inner.len() - 1];
        u32::from_str_radix(hex, 16)
            .ok()
            .and_then(std::char::from_u32)
            .ok_or_else(|| AstBuilderError(format!("Invalid unicode char literal: {}", s)))?
    } else if inner.starts_with('\\') {
        match &inner[1..] {
            "n" => '\n',
            "r" => '\r',
            "t" => '\t',
            "0" => '\0',
            "\\" => '\\',
            "'" => '\'',
            "\"" => '\"',
            _ => return Err(AstBuilderError(format!("Unknown escape sequence: {}", s))),
        }
    } else if inner.chars().count() == 1 {
        inner.chars().next().unwrap()
    } else {
        return Err(AstBuilderError(format!("Invalid character literal length: {}", s)));
    };

    Ok(ExpressionNode::Literal(LiteralNode::Char(ch)))
}
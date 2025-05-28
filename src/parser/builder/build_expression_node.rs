use crate::ast::ast::ExpressionNode;
use crate::parser::builder::build_additive_expression_node::build_additive_expression_node;
use crate::parser::builder::build_array_literal_node::build_array_literal_node;
use crate::parser::builder::build_await_expression_node::build_await_expression_node;
use crate::parser::builder::build_boolean_literal_node::build_boolean_literal_node;
use crate::parser::builder::build_cast_expression_node::build_cast_expression_node;
use crate::parser::builder::build_character_literal_node::build_character_literal_node;
use crate::parser::builder::build_comparison_expression_node::build_comparison_expression_node;
use crate::parser::builder::build_double_literal_node::build_double_literal_node;
use crate::parser::builder::build_equality_expression_node::build_equality_expression_node;
use crate::parser::builder::build_float_literal_node::build_float_literal_node;
use crate::parser::builder::build_identifier_expression_node::build_identifier_expression_node;
use crate::parser::builder::build_integer_literal_node::build_integer_literal_node;
use crate::parser::builder::build_logical_and_expression_node::build_logical_and_expression_node;
use crate::parser::builder::build_logical_or_expression_node::build_logical_or_expression_node;
use crate::parser::builder::build_long_literal_node::build_long_literal_node;
use crate::parser::builder::build_multiplicative_expression_node::build_multiplicative_expression_node;
use crate::parser::builder::build_null_literal_node::build_null_literal_node;
use crate::parser::builder::build_postfix_expression_node::build_postfix_expression_node;
use crate::parser::builder::build_primary_expression_node::build_primary_expression_node;
use crate::parser::builder::build_record_init_node::build_record_init_node;
use crate::parser::builder::build_string_literal_node::build_string_literal_node;
use crate::parser::builder::build_unary_expression_node::build_unary_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};
use pest::iterators::Pair;
use crate::parser::builder::build_ask_expression_node::build_ask_expression_node;

pub fn build_expression_node<'a>(
    pair: Pair<'a, Rule>,
) -> Result<ExpressionNode<'a>, AstBuilderError> {
    match pair.as_rule() {
        Rule::ask_expression => build_ask_expression_node(pair),
        Rule::logical_or_expression => build_logical_or_expression_node(pair),
        Rule::logical_and_expression => build_logical_and_expression_node(pair),
        Rule::equality_expression => build_equality_expression_node(pair),
        Rule::comparison_expression => build_comparison_expression_node(pair),
        Rule::additive_expression => build_additive_expression_node(pair),
        Rule::multiplicative_expression => build_multiplicative_expression_node(pair),
        Rule::unary_expression => build_unary_expression_node(pair),
        Rule::cast_expression => build_cast_expression_node(pair),
        Rule::await_expression => build_await_expression_node(pair),
        Rule::postfix_expression => build_postfix_expression_node(pair),
        Rule::primary_expression => build_primary_expression_node(pair),
        Rule::boolean_literal => build_boolean_literal_node(pair),
        Rule::null_literal => build_null_literal_node(pair),
        Rule::integer_literal => build_integer_literal_node(pair),
        Rule::long_literal => build_long_literal_node(pair),
        Rule::float_literal => build_float_literal_node(pair),
        Rule::double_literal => build_double_literal_node(pair),
        Rule::string_literal => build_string_literal_node(pair),
        Rule::character_literal => build_character_literal_node(pair),
        Rule::array_literal => build_array_literal_node(pair),
        Rule::identifier => build_identifier_expression_node(pair),
        Rule::record_init => build_record_init_node(pair),
        Rule::expression => build_expression_node(pair.into_inner().next().unwrap()),
        _ => Err(AstBuilderError(format!(
            "Unknown expression rule: {:?}",
            pair.as_rule()
        ))),
    }
}

use pest::iterators::Pair;
use crate::ast::ast::ExpressionNode;
use crate::parser::builder::build_array_literal_node::build_array_literal_node;
use crate::parser::builder::build_boolean_literal_node::build_boolean_literal_node;
use crate::parser::builder::build_character_literal_node::build_character_literal_node;
use crate::parser::builder::build_double_literal_node::build_double_literal_node;
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::builder::build_float_literal_node::build_float_literal_node;
use crate::parser::builder::build_identifier_expression_node::build_identifier_expression_node;
use crate::parser::builder::build_integer_literal_node::build_integer_literal_node;
use crate::parser::builder::build_long_literal_node::build_long_literal_node;
use crate::parser::builder::build_null_literal_node::build_null_literal_node;
use crate::parser::builder::build_record_init_node::build_record_init_node;
use crate::parser::builder::build_string_literal_node::build_string_literal_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_primary_expression_node<'a>(pair: Pair<'a, Rule>) -> Result<ExpressionNode<'a>, AstBuilderError> {
    match pair.as_rule() {
        Rule::boolean_literal   => build_boolean_literal_node(pair),
        Rule::null_literal      => build_null_literal_node(pair),
        Rule::integer_literal   => build_integer_literal_node(pair),
        Rule::long_literal      => build_long_literal_node(pair),
        Rule::float_literal     => build_float_literal_node(pair),
        Rule::double_literal    => build_double_literal_node(pair),
        Rule::string_literal    => build_string_literal_node(pair),
        Rule::character_literal => build_character_literal_node(pair),
        Rule::array_literal     => build_array_literal_node(pair),
        Rule::identifier        => build_identifier_expression_node(pair),
        Rule::record_init       => build_record_init_node(pair),
        Rule::expression        => build_expression_node(pair.into_inner().next().unwrap()),
        _ => Err(AstBuilderError(format!("Unknown primary expression rule: {:?}", pair.as_rule()))),
    }
}
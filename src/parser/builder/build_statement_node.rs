use pest::iterators::Pair;
use crate::ast::ast::StatementNode;
use crate::parser::builder::build_ask_statement_simple_node::build_ask_statement_simple_node;
use crate::parser::builder::build_assignment_statement_node::build_assignment_statement_node;
use crate::parser::builder::build_bark_statement_node::build_bark_statement_node;
use crate::parser::builder::build_break_statement_node::build_break_statement_node;
use crate::parser::builder::build_continue_statement_node::build_continue_statement_node;
use crate::parser::builder::build_error_handling_node::build_error_handling_node;
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::builder::build_if_node::build_if_node;
use crate::parser::builder::build_import_node::build_import_node;
use crate::parser::builder::build_loop_node::build_loop_node;
use crate::parser::builder::build_return_statement_node::build_return_statement_node;
use crate::parser::builder::build_say_statement_node::build_say_statement_node;
use crate::parser::builder::build_variable_declaration_node::build_variable_declaration_node;
use crate::parser::builder::build_variable_input_assignment_node::build_variable_input_assignment_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_statement_node<'a>(pair: Pair<'a, Rule>) -> Result<StatementNode<'a>, AstBuilderError> {
    // Check if the received 'pair' is a generic 'statement' rule.
    // If so, it's likely a wrapper around a more specific statement rule.
    let specific_statement_pair = if pair.as_rule() == Rule::statement {
        // Get the inner pair, which should be the actual specific statement.
        pair.into_inner().next().ok_or_else(|| {
            AstBuilderError("Generic 'statement' rule was empty, expected a specific statement type inside.".to_string())
        })?
    } else {
        // If it's not Rule::statement, assume it's already a specific statement pair.
        pair
    };

    // Now, match on the rule of the (potentially unwrapped) specific_statement_pair.
    match specific_statement_pair.as_rule() {
        Rule::variable_declaration => build_variable_declaration_node(specific_statement_pair),
        Rule::variable_input_assignment => build_variable_input_assignment_node(specific_statement_pair),
        Rule::assignment_statement => build_assignment_statement_node(specific_statement_pair),
        Rule::say_statement => build_say_statement_node(specific_statement_pair),
        Rule::ask_statement_simple => build_ask_statement_simple_node(specific_statement_pair),
        Rule::return_statement => build_return_statement_node(specific_statement_pair),
        Rule::if_statement => build_if_node(specific_statement_pair),
        Rule::loop_statement => build_loop_node(specific_statement_pair),
        Rule::error_handling_statement => build_error_handling_node(specific_statement_pair),
        Rule::bark_statement => build_bark_statement_node(specific_statement_pair),
        Rule::break_statement => build_break_statement_node(specific_statement_pair),
        Rule::continue_statement => build_continue_statement_node(specific_statement_pair),
        Rule::import_statement => Ok(StatementNode::Import(build_import_node(specific_statement_pair)?)),
        Rule::expression_statement => {
            // An expression_statement typically wraps a single expression.
            let expr = build_expression_node(specific_statement_pair.into_inner().next()
                .ok_or_else(|| AstBuilderError("expression_statement is empty".to_string()))?)?;
            Ok(StatementNode::Expression(expr))
        }
        _ => Err(AstBuilderError(format!(
            "Unknown specific statement rule: {:?}", // Changed error message for clarity
            specific_statement_pair.as_rule()
        ))),
    }
}
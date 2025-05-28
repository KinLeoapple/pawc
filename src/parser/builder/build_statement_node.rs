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
    let rule_to_dispatch_on;
    // This is the pair that contains all components for the specific builder.
    // The specific builders (e.g., build_variable_declaration_node) will call `into_inner()` on this.
    let pair_for_children_consumption = pair.clone();

    if pair.as_rule() == Rule::statement {
        // It's a generic `statement` rule, peek at its first child to determine type.
        let first_child = pair.into_inner().peek().ok_or_else(|| {
            AstBuilderError("Generic 'statement' rule is empty, cannot determine specific type.".to_string())
        })?;
        rule_to_dispatch_on = first_child.as_rule();
    } else {
        // It's already a specific rule (e.g., if called from a context that parsed specific statements).
        rule_to_dispatch_on = pair.as_rule();
    }

    // Now, match on the rule of the (potentially unwrapped) specific_statement_pair.
    match rule_to_dispatch_on {
        Rule::KEYWORD_LET => build_variable_declaration_node(pair_for_children_consumption),
        Rule::KEYWORD_SAY => build_say_statement_node(pair_for_children_consumption),
        Rule::KEYWORD_IF => build_if_node(pair_for_children_consumption),
        Rule::KEYWORD_RETURN => build_return_statement_node(pair_for_children_consumption),
        Rule::KEYWORD_LOOP => build_loop_node(pair_for_children_consumption),
        Rule::KEYWORD_SNIFF => build_error_handling_node(pair_for_children_consumption),
        Rule::KEYWORD_BARK => build_bark_statement_node(pair_for_children_consumption),
        Rule::KEYWORD_BREAK => build_break_statement_node(pair_for_children_consumption),
        Rule::KEYWORD_CONTINUE => build_continue_statement_node(pair_for_children_consumption),
        Rule::KEYWORD_IMPORT => Ok(StatementNode::Import(build_import_node(pair_for_children_consumption)?)),

        Rule::variable_declaration => build_variable_declaration_node(pair_for_children_consumption),
        Rule::variable_input_assignment => build_variable_input_assignment_node(pair_for_children_consumption),
        Rule::assignment_statement => build_assignment_statement_node(pair_for_children_consumption),
        Rule::say_statement => build_say_statement_node(pair_for_children_consumption),
        Rule::ask_statement_simple => build_ask_statement_simple_node(pair_for_children_consumption),
        Rule::return_statement => build_return_statement_node(pair_for_children_consumption),
        Rule::if_statement => build_if_node(pair_for_children_consumption),
        Rule::loop_statement => build_loop_node(pair_for_children_consumption),
        Rule::error_handling_statement => build_error_handling_node(pair_for_children_consumption),
        Rule::bark_statement => build_bark_statement_node(pair_for_children_consumption),
        Rule::break_statement => build_break_statement_node(pair_for_children_consumption),
        Rule::continue_statement => build_continue_statement_node(pair_for_children_consumption),
        Rule::import_statement => Ok(StatementNode::Import(build_import_node(pair_for_children_consumption)?)),
        Rule::expression_statement => {
            // An expression_statement typically wraps a single expression.
            let expr = build_expression_node(pair_for_children_consumption.into_inner().next()
                .ok_or_else(|| AstBuilderError("expression_statement is empty".to_string()))?)?;
            Ok(StatementNode::Expression(expr))
        }
        _ => Err(AstBuilderError(format!(
            "Unknown specific statement rule: {:?}", // Changed error message for clarity
            pair_for_children_consumption.as_rule()
        ))),
    }
}
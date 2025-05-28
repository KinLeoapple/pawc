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
    match pair.as_rule() {
        Rule::variable_declaration         => build_variable_declaration_node(pair),
        Rule::variable_input_assignment    => build_variable_input_assignment_node(pair),
        Rule::assignment_statement         => build_assignment_statement_node(pair),
        Rule::say_statement                => build_say_statement_node(pair),
        Rule::ask_statement_simple         => build_ask_statement_simple_node(pair),
        Rule::return_statement             => build_return_statement_node(pair),
        Rule::if_statement                 => build_if_node(pair),
        Rule::loop_statement               => build_loop_node(pair),
        Rule::error_handling_statement     => build_error_handling_node(pair),
        Rule::bark_statement               => build_bark_statement_node(pair),
        Rule::break_statement              => build_break_statement_node(pair),
        Rule::continue_statement           => build_continue_statement_node(pair),
        Rule::import_statement             => {
            Ok(StatementNode::Import(build_import_node(pair)?))
        }
        Rule::expression_statement         => {
            let expr = build_expression_node(pair.into_inner().next().unwrap())?;
            Ok(StatementNode::Expression(expr))
        }
        _ => Err(AstBuilderError(format!("Unknown statement rule: {:?}", pair.as_rule()))),
    }
}
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
use pest::iterators::Pair;

pub fn build_statement_node<'a>(
    pair: Pair<'a, Rule>,
) -> Result<StatementNode<'a>, AstBuilderError> {
    // 如果这是一个 wrapper `statement`，就直接 unwrap 到内部那一层（say_statement、let_statement 等）
    let specific = if pair.as_rule() == Rule::statement {
        let mut inner = pair.into_inner();
        inner
            .next()
            .ok_or_else(|| AstBuilderError("Generic 'statement' is empty".into()))?
    } else {
        pair
    };

    match specific.as_rule() {
        Rule::variable_declaration => build_variable_declaration_node(specific),
        // Rule::variable_input_assignment => build_variable_input_assignment_node(specific),
        Rule::assignment_statement => build_assignment_statement_node(specific),
        Rule::say_statement => build_say_statement_node(specific),
        Rule::ask_statement_simple => build_ask_statement_simple_node(specific),
        Rule::return_statement => build_return_statement_node(specific),
        Rule::if_statement => build_if_node(specific),
        Rule::loop_statement => build_loop_node(specific),
        Rule::error_handling_statement => build_error_handling_node(specific),
        Rule::bark_statement => build_bark_statement_node(specific),
        Rule::break_statement => build_break_statement_node(specific),
        Rule::continue_statement => build_continue_statement_node(specific),
        Rule::import_statement => Ok(StatementNode::Import(build_import_node(specific)?)),
        Rule::expression_statement => {
            let expr_pair = specific
                .into_inner()
                .next()
                .ok_or_else(|| AstBuilderError("expression_statement is empty".into()))?;
            let expr = build_expression_node(expr_pair)?;
            Ok(StatementNode::Expression(expr))
        }
        other => Err(AstBuilderError(format!(
            "Unknown statement rule: {:?}",
            other
        ))),
    }
}

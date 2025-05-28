use pest::iterators::Pair;
use crate::ast::ast::StatementNode;
use crate::parser::builder::build_expression_node::build_expression_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_return_statement_node<'a>(pair: Pair<'a, Rule>) -> Result<StatementNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner(); // Children of the `return_statement` rule.
    // Expected structure from grammar.pest: KEYWORD_RETURN ~ expression?

    // The first child must be KEYWORD_RETURN.
    // Consume this token.
    let keyword_token = inner.next().ok_or_else(|| AstBuilderError("Return statement rule is unexpectedly empty. Expected KEYWORD_RETURN.".into()))?;
    if keyword_token.as_rule() != Rule::KEYWORD_RETURN {
        return Err(AstBuilderError(format!(
            "Expected KEYWORD_RETURN as the first part of a return statement, but found {:?}.",
            keyword_token.as_rule()
        )));
    }

    // The next child, if it exists, is the expression to be returned.
    let expr_opt = inner.next() // This will be Some(expression_pair) if an expression is present, or None otherwise.
        .map(|expression_pair| build_expression_node(expression_pair)) // If Some(expression_pair), build it.
        .transpose()?; // Converts Result<Option<T>, E> to Option<Result<T,E>> then to Result<Option<T>,E>.

    // Ensure no other tokens follow the optional expression within the return_statement rule,
    // which would indicate a grammar or parsing logic mismatch.
    if inner.next().is_some() {
        return Err(AstBuilderError("Unexpected additional tokens found after the expression (or lack thereof) in a return statement.".into()));
    }

    Ok(StatementNode::Return {
        expr: expr_opt,
        line,
        col,
    })
}
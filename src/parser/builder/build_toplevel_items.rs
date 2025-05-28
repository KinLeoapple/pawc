use pest::iterators::Pair;
use crate::ast::ast::{TopLevelItem, TopLevelKind};
use crate::parser::builder::build_function_definition_node::build_function_definition_node;
use crate::parser::builder::build_import_node::build_import_node;
use crate::parser::builder::build_protocol_definition_node::build_protocol_definition_node;
use crate::parser::builder::build_record_definition_node::build_record_definition_node;
use crate::parser::builder::build_statement_node::build_statement_node;
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_toplevel_items<'a>(program_pair: Pair<'a, Rule>) -> Result<Vec<TopLevelItem<'a>>, AstBuilderError> {
    let mut items = Vec::new();
    // program_pair is expected to be the pair for the `program` rule.
    // Its children (program_pair.into_inner()) are the actual top-level constructs.
    for child_of_program in program_pair.into_inner() {
        let (line, col) = child_of_program.as_span().start_pos().line_col(); // Get line/col from the item itself

        match child_of_program.as_rule() {
            Rule::top_level_item => {
                // This rule wraps an actual underlying item. Unwrap it.
                let actual_item_pair = child_of_program.into_inner().next()
                    .ok_or_else(|| AstBuilderError("Encountered an empty top_level_item rule.".to_string()))?;

                // Use the line and column from the actual unwrapped item.
                let (item_line, item_col) = actual_item_pair.as_span().start_pos().line_col();

                let node = match actual_item_pair.as_rule() {
                    Rule::import_statement => {
                        TopLevelKind::ModuleImport(build_import_node(actual_item_pair)?)
                    }
                    Rule::function_definition => {
                        TopLevelKind::Function(build_function_definition_node(actual_item_pair)?)
                    }
                    Rule::record_definition => {
                        TopLevelKind::Record(build_record_definition_node(actual_item_pair)?)
                    }
                    Rule::protocol_definition => {
                        TopLevelKind::Protocol(build_protocol_definition_node(actual_item_pair)?)
                    }
                    Rule::statement => {
                        TopLevelKind::Statement(build_statement_node(actual_item_pair)?)
                    }
                    _ => return Err(AstBuilderError(format!(
                        "Invalid rule found inside top_level_item: {:?}",
                        actual_item_pair.as_rule()
                    ))),
                };
                items.push(TopLevelItem { node, line: item_line, col: item_col });
            }
            // Direct handling for rules as per the original file structure.
            // These cases would be hit if these rules can appear directly under `program`
            // without being wrapped by `top_level_item`.
            Rule::import_statement => {
                items.push(TopLevelItem {
                    node: TopLevelKind::ModuleImport(build_import_node(child_of_program)?),
                    line, // line/col from child_of_program
                    col,
                });
            }
            Rule::function_definition => {
                items.push(TopLevelItem {
                    node: TopLevelKind::Function(build_function_definition_node(child_of_program)?),
                    line, // line/col from child_of_program
                    col,
                });
            }
            Rule::record_definition => {
                items.push(TopLevelItem {
                    node: TopLevelKind::Record(build_record_definition_node(child_of_program)?),
                    line, // line/col from child_of_program
                    col,
                });
            }
            Rule::protocol_definition => {
                items.push(TopLevelItem {
                    node: TopLevelKind::Protocol(build_protocol_definition_node(child_of_program)?),
                    line, // line/col from child_of_program
                    col,
                });
            }
            Rule::statement => {
                let stmt = build_statement_node(child_of_program)?;
                items.push(TopLevelItem {
                    node: TopLevelKind::Statement(stmt),
                    line, // line/col from child_of_program
                    col,
                });
            }
            Rule::EOI => {
                // If EOI is a valid rule and can appear as a child of the program rule,
                // it's often ignored at this stage.
                // (As seen in build_array_literal_node.rs)
            }
            _ => return Err(AstBuilderError(format!(
                "Unknown top-level rule: {:?}", // Restored original error message format
                child_of_program.as_rule()
            ))),
        }
    }
    Ok(items)
}

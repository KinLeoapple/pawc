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
    for pair in program_pair.into_inner() {
        let (line, col) = pair.as_span().start_pos().line_col();
        match pair.as_rule() {
            Rule::import_statement => {
                items.push(TopLevelItem {
                    node: TopLevelKind::ModuleImport(build_import_node(pair)?),
                    line,
                    col,
                });
            }
            Rule::function_definition => {
                items.push(TopLevelItem {
                    node: TopLevelKind::Function(build_function_definition_node(pair)?),
                    line,
                    col,
                });
            }
            Rule::record_definition => {
                items.push(TopLevelItem {
                    node: TopLevelKind::Record(build_record_definition_node(pair)?),
                    line,
                    col,
                });
            }
            Rule::protocol_definition => {
                items.push(TopLevelItem {
                    node: TopLevelKind::Protocol(build_protocol_definition_node(pair)?),
                    line,
                    col,
                });
            }
            Rule::statement => {
                // 处理顶层语句（如 say, let, if, expression_statement, ...）
                let stmt = build_statement_node(pair)?;
                items.push(TopLevelItem {
                    node: TopLevelKind::Statement(stmt),
                    line,
                    col,
                });
            }
            _ => return Err(AstBuilderError(format!("Unknown top-level rule: {:?}", pair.as_rule()))),
        }
    }
    Ok(items)
}
use crate::ast::ast::{IdentifierNode, ImportNode, ModulePath};
use crate::parser::parser::{AstBuilderError, Rule};
use pest::iterators::Pair;

pub fn build_import_node<'a>(pair: Pair<'a, Rule>) -> Result<ImportNode<'a>, AstBuilderError> {
    let (line, col) = pair.as_span().start_pos().line_col();
    let mut inner = pair.into_inner();

    let first = inner
        .next()
        .ok_or_else(|| AstBuilderError("Import: empty import_statement".into()))?;
    if first.as_rule() != Rule::KEYWORD_IMPORT {
        return Err(AstBuilderError(format!(
            "Import: expected 'import', got {:?}",
            first.as_rule()
        )));
    }

    let path_pair = inner
        .next()
        .ok_or_else(|| AstBuilderError("Import: missing import_path".into()))?;
    if path_pair.as_rule() != Rule::import_path {
        return Err(AstBuilderError(format!(
            "Import: expected import_path, got {:?}",
            path_pair.as_rule()
        )));
    }

    // 构造 ModulePath
    let mut segments = Vec::new();
    for seg in path_pair.into_inner() {
        if seg.as_rule() == Rule::identifier {
            let (ln, cl) = seg.as_span().start_pos().line_col();
            segments.push(IdentifierNode {
                name: seg.as_str(),
                line: ln,
                col: cl,
            });
        }
    }
    let path = ModulePath {
        segments,
        line,
        col,
    };

    let alias = if let Some(next) = inner.next() {
        if next.as_rule() != Rule::KEYWORD_AS {
            return Err(AstBuilderError(format!(
                "Import: expected 'as', got {:?}",
                next.as_rule()
            )));
        }
        let id_pair = inner
            .next()
            .ok_or_else(|| AstBuilderError("Import: missing alias".into()))?;
        let (ln, cl) = id_pair.as_span().start_pos().line_col();
        Some(IdentifierNode {
            name: id_pair.as_str(),
            line: ln,
            col: cl,
        })
    } else {
        None
    };

    Ok(ImportNode { path, alias })
}

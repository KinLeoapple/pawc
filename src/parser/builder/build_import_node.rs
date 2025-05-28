use pest::iterators::Pair;
use crate::ast::ast::{IdentifierNode, ImportNode, ModulePath};
use crate::parser::parser::{AstBuilderError, Rule};

pub fn build_import_node<'a>(pair: Pair<'a, Rule>) -> Result<ImportNode<'a>, AstBuilderError> {
    // pair: import_statement
    let mut inner = pair.into_inner();
    let first = inner.next().ok_or_else(|| AstBuilderError("Import missing path".into()))?;

    // 路径段
    let mut segments = Vec::new();
    let (mut start_line, mut start_col) = (0, 0);
    if first.as_rule() == Rule::import_path {
        for (i, seg) in first.into_inner().enumerate() {
            let (line, col) = seg.as_span().start_pos().line_col();
            if i == 0 {
                start_line = line;
                start_col = col;
            }
            segments.push(IdentifierNode {
                name: seg.as_str(),
                line,
                col,
            });
        }
    } else {
        return Err(AstBuilderError(format!("Import: expected import_path, got {:?}", first.as_rule())));
    }
    let path = ModulePath {
        segments,
        line: start_line,
        col: start_col,
    };

    // 可选别名
    let mut alias = None;
    if let Some(next) = inner.next() {
        // 结构为 (KEYWORD_AS ~ identifier)
        let mut alias_inner = next.into_inner();
        let alias_pair = alias_inner.next().ok_or_else(|| AstBuilderError("Import: missing alias ident".into()))?;
        let (l, c) = alias_pair.as_span().start_pos().line_col();
        alias = Some(IdentifierNode {
            name: alias_pair.as_str(),
            line: l,
            col: c,
        });
    }

    Ok(ImportNode { path, alias })
}
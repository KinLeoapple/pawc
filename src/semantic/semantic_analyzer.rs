use crate::semantic::symbol_table::SymbolTable;

pub struct SemanticAnalyzer<'a> {
    symbols: SymbolTable<'a>,
}
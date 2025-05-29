extern crate pest;

mod parser;
mod ast;
mod semantic;

use crate::parser::parser::{parse, PawScriptParser, Rule};
use once_cell::sync::OnceCell;
use pest::Parser;

pub static STACK_SIZE: OnceCell<usize> = OnceCell::with_value(1);

fn main() {
    let src = std::fs::read_to_string("test.paw").unwrap();
    let pairs = PawScriptParser::parse(Rule::program, &src)
        .expect("Parse failed");

    let ast = parse(pairs).expect("AST build failed");

    println!("{:#?}", ast);
}

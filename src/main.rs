use crate::cli::cli::run_sync;

mod semantic;
mod parser;
mod lexer;
mod interpreter;
mod ast;
mod cli;
mod error;

fn main() {
    run_sync()
}
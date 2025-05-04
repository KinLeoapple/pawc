use crate::cli::cli::run;

mod semantic;
mod parser;
mod lexer;
mod interpreter;
mod ast;
mod cli;
mod error;

#[tokio::main]
async fn main() {
    run().await
}
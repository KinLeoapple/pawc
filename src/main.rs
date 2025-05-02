// src/main.rs

mod ast;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod semantic;

use clap::Parser;
use interpreter::interpreter::Interpreter;
use lexer::lex::Lexer;
use parser::parser::Parser as PawParser;
use semantic::type_checker::TypeChecker;
use std::{fs, path::PathBuf};
use crate::error::error::PawError;

/// 🐾 PawScript interpreter — execute .paw scripts
#[derive(Parser, Debug)]
#[command(
    name = "pawc",
    version = "0.1.5",
    author = "Kinleoapple",
    about = "🐾 PawScript interpreter — execute .paw scripts"
)]
struct Args {
    /// Path to the .paw script to run
    #[arg(value_name = "SCRIPT", required = true)]
    script: PathBuf,

    /// Verbosity: -v, -vv for more debug output
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() {
    let args = Args::parse();
    if let Err(err) = run(&args.script, args.verbose) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

/// Load, parse, type‐check and run a PawScript file.
fn run(script: &PathBuf, verbose: u8) -> Result<(), PawError> {
    // 1. Read file
    let src = fs::read_to_string(script).map_err(|e| PawError::Internal {
        code:    "E1000".into(),
        message: format!("Failed to read script '{}': {}", script.display(), e),
        line:    0,
        column:  0,
        snippet: None,
        hint:    Some("Ensure the file exists and is readable.".into()),
    })?;

    // 2. Lex & parse
    let tokens = Lexer::new(&src).tokenize();
    if verbose > 0 {
        eprintln!("Tokens: {:#?}", tokens);
    }

    let mut parser = PawParser::new(tokens);
    let ast = parser.parse_program().map_err(|mut err| {
        // If you want, you could fill in err.line/column/snippet here
        err
    })?;

    if verbose > 0 {
        eprintln!("AST: {:#?}", ast);
    }

    // 3. Static type check
    let mut tc = TypeChecker::new();
    tc.check_statements(&ast).map_err(|mut err| {
        // err already has code/message/etc.
        err
    })?;

    // 4. Interpret
    let mut interp = Interpreter::new();
    interp.run(&ast).map_err(|mut err| {
        // runtime errors already carry full PawError fields
        err
    })?;

    Ok(())
}

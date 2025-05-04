// src/cli/cli.rs

use crate::parser::parser::Parser as PawParser;
use crate::{
    error::error::PawError,
    interpreter::env::Env,
    interpreter::interpreter::Interpreter,
    lexer::lexer::Lexer,
    semantic::type_checker::TypeChecker,
};
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use futures_util::TryFutureExt;

/// 🐾 PawScript interpreter — execute .paw scripts
#[derive(Parser, Debug)]
#[command(
    name = "pawc",
    version = "0.1.8",
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

pub(crate) async fn run() {
    let args = Args::parse();
    if let Err(err) = run_script(&args.script, args.verbose).await {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

/// Load, parse, type‐check and run a PawScript file.
async fn run_script(script: &PathBuf, verbose: u8) -> Result<(), PawError> {
    // 1. Read file
    let src = fs::read_to_string(script).map_err(|e| PawError::Internal {
        file: script.to_str().unwrap_or_default().into(),
        code: "E1000".into(),
        message: format!("Failed to read script '{}': {}", script.display(), e),
        line: 0,
        column: 0,
        snippet: None,
        hint: Some("Ensure the file exists and is readable.".into()),
    })?;

    // 2. Lex & parse
    let tokens = Lexer::new(&src).tokenize();
    if verbose > 0 {
        eprintln!("Tokens: {:#?}", tokens);
    }

    let mut parser = PawParser::new(tokens, &src, &*script.to_string_lossy());
    let ast = parser.parse_program().map_err(|err| {
        // If you want, you could fill in err.line/column/snippet here
        err
    })?;

    if verbose > 0 {
        eprintln!("AST: {:#?}", ast);
    }

    // 3. Static type check
    let mut tc = TypeChecker::new(&*script.to_string_lossy());
    tc.check_program(&ast).map_err(|err| {
        // err already has code/message/etc.
        err
    })?;

    // 4. Interpret
    let env = Env::new();
    let mut interp = Interpreter::new(env, &*script.to_string_lossy());
    interp.eval_statements(&ast).await?;

    Ok(())
}

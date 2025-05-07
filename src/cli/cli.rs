// src/cli/cli.rs

use crate::WORKER_STACK_SIZE;
use crate::STACK_SIZE;
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

/// 🐾 PawScript interpreter — execute .paw scripts
#[derive(Parser, Debug)]
#[command(
    name = "pawc",
    version = "0.1.9",
    author = "Kinleoapple",
    about = "🐾 PawScript interpreter — execute .paw scripts"
)]
struct Args {
    /// Path to the .paw script to run
    #[arg(value_name = "SCRIPT", required = true)]
    script: PathBuf,

    /// 栈大小（MiB），默认 16
    #[arg(long, default_value = "1")]
    pub stack_size: usize,

    /// Tokio 每个 worker 线程的栈大小（MiB），默认 8
    #[arg(long, default_value = "1")]
    pub worker_stack_size: usize,
}

pub(crate) async fn run() {
    let args = Args::parse();
    STACK_SIZE.set(args.stack_size).ok();
    WORKER_STACK_SIZE.set(args.worker_stack_size).ok();
    if let Err(err) = run_script(&args.script).await {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

/// Load, parse, type‐check and run a PawScript file.
async fn run_script(script: &PathBuf) -> Result<(), PawError> {
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

    let mut parser = PawParser::new(tokens, &src, &*script.to_string_lossy());
    let ast = parser.parse_program().map_err(|err| {
        // If you want, you could fill in err.line/column/snippet here
        err
    })?;

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

// src/cli/cli.rs

use crate::interpreter::interpreter::Engine;
use crate::parser::parser::Parser as PawParser;
use crate::{
    error::error::PawError, interpreter::env::Env, interpreter::interpreter::Interpreter,
    lexer::lexer::Lexer, semantic::type_checker::TypeChecker, STACK_SIZE,
};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use crate::utils::package::derive_package_name;

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

    /// 栈大小（MiB），默认 1
    #[arg(long, default_value = "1")]
    pub stack_size: usize, // MiB
}

pub(crate) async fn run() {
    let args = Args::parse();
    STACK_SIZE.set(args.stack_size).ok();

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
        err
    })?;

    // 3. Static type check — 先隐式推断包名，再传给 TypeChecker
    let project_root = std::env::current_dir()
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    let pkg = derive_package_name(script.as_path(), &project_root);
    let mut tc = TypeChecker::new(&*script.to_string_lossy(), &*pkg);
    tc.check_program(&ast, &project_root).map_err(|err| {
        err
    })?;

    // 4. Interpret
    let env = Env::new();
    let engine = Engine::new(env, &*script.to_string_lossy());
    vuot::run(Interpreter {
        engine,
        statements: &ast,
    })
    .await?;

    Ok(())
}

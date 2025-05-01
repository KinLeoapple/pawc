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
    version = "0.1.4",
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
    // 1) 先用 clap 解析命令行
    let args = Args::parse();

    // 2) 调用 run，并把错误打印出来
    if let Err(err) = run(&args.script, args.verbose) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

/// 把原来那个 run() 改成接收 script 路径和 verbosity
fn run(script: &PathBuf, verbose: u8) -> Result<(), PawError> {
    // 1. 读文件
    let src = fs::read_to_string(script)
        .map_err(|e| PawError::Internal { message: e.to_string() })?;

    // 2. 词法 & 语法
    let tokens = Lexer::new(&src).tokenize();
    if verbose > 0 {
        eprintln!("Tokens: {:#?}", tokens);
    }

    let mut parser = PawParser::new(tokens);
    let ast = parser.parse_program()?;  // 可能返回 PawError::Syntax

    if verbose > 0 {
        eprintln!("AST: {:#?}", ast);
    }

    // 3. 静态类型检查
    let mut tc = TypeChecker::new();
    tc.check_statements(&ast)?;         // 可能返回 PawError::Type

    // 4. 解释执行
    let mut interp = Interpreter::new();
    interp.run(&ast)?;                  // 可能返回 PawError::Runtime

    Ok(())
}

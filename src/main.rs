mod ast;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod semantic;


use error::PawError;
use interpreter::Interpreter;
use parser::Parser;
use std::env;
use std::fs;
use crate::lexer::lex::Lexer;
use crate::semantic::type_checker::TypeChecker;

fn main() {
    // 把所有逻辑放在一个函数里，返回 Result
    if let Err(err) = run() {
        // 用 Display 打印错误
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<(), PawError> {
    // 1. 读文件
    let path = env::args().nth(1)
        .ok_or_else(|| PawError::Internal { message: "Please provide a script path".into() })?;
    let src = fs::read_to_string(&path)
        .map_err(|e| PawError::Internal { message: e.to_string() })?;

    // 2. 词法 & 语法
    let tokens = Lexer::new(&src).tokenize();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_program()?;  // 这里可能返回 PawError::Syntax

    // 3. 静态类型检查
    let mut tc = TypeChecker::new();
    tc.check_statements(&ast)?;         // 这里可能返回 PawError::Type

    // 4. 执行
    let mut interp = Interpreter::new();
    interp.run(&ast)?;                  // 这里可能返回 PawError::Runtime

    Ok(())
}
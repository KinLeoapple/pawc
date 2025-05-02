// src/interpreter.rs

use crate::ast::expr::ExprKind;
use crate::ast::{BinaryOp, Expr, Statement, StatementKind};
use crate::error::error::PawError;
use crate::lexer::lex::Lexer;
use crate::parser::parser::Parser;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::Write;

/// 运行时值
#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Long(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    Array(Vec<Value>),
    Function {
        params: Vec<String>,
        body: Vec<Statement>,
        // 闭包时捕获的外部环境
        env: Env,
    },
    Module(HashMap<String, Value>),
    Void, // return; 时使用
    Null,
}

// －－－ 手写 PartialEq －－－
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Long(a), Value::Long(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a1), Value::Array(a2)) => a1 == a2,
            (Value::Void, Value::Void) => true,
            (Value::Null, Value::Null) => true,
            // Function、不同变体或类型不匹配都算不相等
            _ => false,
        }
    }
}

// －－－ 手写 PartialOrd －－－
impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
            (Value::Long(a), Value::Long(b)) => a.partial_cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
            (Value::Char(a), Value::Char(b)) => a.partial_cmp(b),
            (Value::String(a), Value::String(b)) => a.partial_cmp(b),
            // 其余情况（Bool、Array、Function、Void）不支持大小比较
            _ => None,
        }
    }
}

impl Value {
    fn to_bool(&self, expr: &Expr, file: &str) -> Result<bool, PawError> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(PawError::Type {
                file: file.to_string(),
                code: "E4001".into(),
                message: format!("Cannot convert {:?} to bool", self),
                line: expr.line,
                column: expr.col,
                snippet: None,
                hint: None,
            }),
        }
    }

    fn to_string_value(&self) -> String {
        match self {
            Value::Null => "null".to_string(),
            Value::String(s) => s.clone(),
            Value::Int(i) => i.to_string(),
            Value::Long(l) => l.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Char(c) => c.to_string(),
            Value::Array(a) => format!("{:?}", a),
            Value::Module(m) => format!("{:?}", m),
            Value::Void => "void".to_string(),
            Value::Function { .. } => "<fn>".into(),
        }
    }
}

/// 执行结果，用于控制流（return/break/continue）
#[derive(Debug)]
enum ExecResult {
    Normal,
    Return(Value),
    Break,
    Continue,
}

/// 运行时环境：一系列嵌套作用域
#[derive(Debug, Clone, PartialEq)]
pub struct Env {
    scopes: Vec<HashMap<String, Value>>,
    file: String,
}

impl Env {
    pub fn new(filename: &str) -> Self {
        Env {
            scopes: vec![HashMap::new()],
            file: filename.to_string(),
        }
    }

    pub fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: String, val: Value) {
        let top = self.scopes.last_mut().unwrap();
        top.insert(name, val);
    }

    pub fn set(&mut self, name: &str, val: Value) -> Result<(), PawError> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.into(), val);
                return Ok(());
            }
        }
        Err(PawError::UndefinedVariable {
            file: self.file.clone(),
            code: "E2003".into(),
            name: name.into(),
            line: 0,
            column: 0,
            snippet: None,
            hint: Some("Check spelling or scope.".into()),
        })
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(name) {
                return Some(v.clone());
            }
        }
        None
    }
}

/// 解释器主体
pub struct Interpreter {
    env: Env,
    file: String,
}

impl Interpreter {
    pub fn new(filename: &str) -> Self {
        Interpreter { env: Env::new(filename), file: filename.to_string() }
    }

    /// 执行整个程序
    pub fn run(&mut self, stmts: &[Statement]) -> Result<(), PawError> {
        // 先把所有顶层函数声明绑到环境里
        for stmt in stmts {
            if let StatementKind::FunDecl {
                name,
                params,
                return_type: _,
                body,
            } = &stmt.kind
            {
                let f = Value::Function {
                    params: params.iter().map(|p| p.name.clone()).collect(),
                    body: body.clone(),
                    env: self.env.clone(),
                };
                self.env.define(name.clone(), f);
            }
        }
        // 然后执行顶层语句
        self.exec_block(stmts)?;
        Ok(())
    }

    /// 执行一系列语句
    fn exec_block(&mut self, stmts: &[Statement]) -> Result<ExecResult, PawError> {
        for stmt in stmts {
            match self.exec_stmt(stmt)? {
                ExecResult::Normal => continue,
                other => return Ok(other),
            }
        }
        Ok(ExecResult::Normal)
    }

    /// 执行单条语句
    fn exec_stmt(&mut self, stmt: &Statement) -> Result<ExecResult, PawError> {
        match &stmt.kind {
            StatementKind::Import { module, alias } => {
                // 构造文件名并读取
                let mut path = std::path::PathBuf::new();
                for seg in module {
                    path.push(seg);
                }
                path.set_extension("paw");
                let src = std::fs::read_to_string(&path).map_err(|e| PawError::Internal {
                    file: self.file.clone(),
                    code: "E5001".into(),
                    message: format!("cannot import {:?}: {}", path, e),
                    line: 0,
                    column: 0,
                    snippet: None,
                    hint: Some("Check that the file exists and is readable.".into()),
                })?;

                // 重新编译运行子模块
                let tokens = Lexer::new(&src).tokenize();
                let mut p = Parser::new(tokens, &src, path.to_str().unwrap_or_default());
                let ast = p.parse_program()?;
                let mut tc = crate::semantic::type_checker::TypeChecker::new(path.to_str().unwrap_or_default());
                tc.check_statements(&ast)?;
                let mut sub = Interpreter::new(path.to_str().unwrap_or_default());
                sub.run(&ast)?;

                // 把子模块顶层符号收集到一个 map
                let mut module_map = HashMap::new();
                for scope in &sub.env.scopes {
                    for (k, v) in scope {
                        module_map.insert(k.clone(), v.clone());
                        // 同时也可扁平绑定 alias.k
                        let full = format!("{}.{}", alias, k);
                        self.env.define(full, v.clone());
                    }
                }
                // 给 alias 自身绑定一个 Module 值
                self.env.define(alias.clone(), Value::Module(module_map));
                Ok(ExecResult::Normal)
            }

            StatementKind::Let { name, ty: _, value } => {
                let v = self.eval_expr(value)?;
                self.env.define(name.clone(), v);
                Ok(ExecResult::Normal)
            }

            StatementKind::Say(expr) => {
                let v = self.eval_expr(expr)?;
                println!("{}", v.to_string_value());
                Ok(ExecResult::Normal)
            }

            StatementKind::Assign { name, value } => {
                let v = self.eval_expr(value)?;
                self.env.set(name, v)?; // 更新已存在的变量
                Ok(ExecResult::Normal)
            }

            StatementKind::Ask {
                name,
                ty: _,
                prompt,
            } => {
                print!("{}", prompt);
                std::io::stdout().flush().map_err(|e| PawError::Internal {
                    file: self.file.clone(),
                    code: "E5002".into(),
                    message: e.to_string(),
                    line: 0,
                    column: 0,
                    snippet: None,
                    hint: Some("Failed to flush stdout.".into()),
                })?;
                let mut line = String::new();
                std::io::stdin()
                    .read_line(&mut line)
                    .map_err(|e| PawError::Internal {
                        file: self.file.clone(),
                        code: "E5003".into(),
                        message: e.to_string(),
                        line: 0,
                        column: 0,
                        snippet: None,
                        hint: Some("Failed to read from stdin.".into()),
                    })?;
                self.env.define(name.clone(), Value::String(line.into()));
                Ok(ExecResult::Normal)
            }

            StatementKind::AskPrompt(prompt) => {
                print!("{}", prompt);
                std::io::stdout().flush().map_err(|e| PawError::Internal {
                    file: self.file.clone(),
                    code: "E5002".into(),
                    message: e.to_string(),
                    line: 0,
                    column: 0,
                    snippet: None,
                    hint: Some("Failed to flush stdout.".into()),
                })?;
                let mut line = String::new();
                std::io::stdin()
                    .read_line(&mut line)
                    .map_err(|e| PawError::Internal {
                        file: self.file.clone(),
                        code: "E5003".into(),
                        message: e.to_string(),
                        line: 0,
                        column: 0,
                        snippet: None,
                        hint: Some("Failed to read from stdin.".into()),
                    })?;
                Ok(ExecResult::Normal)
            }

            StatementKind::Return(opt) => {
                let v = if let Some(e) = opt {
                    self.eval_expr(e)?
                } else {
                    Value::Void
                };
                Ok(ExecResult::Return(v))
            }

            StatementKind::Break => Ok(ExecResult::Break),
            StatementKind::Continue => Ok(ExecResult::Continue),

            StatementKind::Expr(expr) => {
                let _ = self.eval_expr(expr)?;
                Ok(ExecResult::Normal)
            }

            StatementKind::If {
                condition,
                body,
                else_branch,
            } => {
                let c = self.eval_expr(condition)?;
                if c.to_bool(condition, &*self.file)? {
                    self.env.push();
                    let res = self.exec_block(body)?;
                    self.env.pop();
                    Ok(res)
                } else if let Some(else_stmt) = else_branch {
                    self.env.push();
                    let res = self.exec_stmt(else_stmt)?;
                    self.env.pop();
                    Ok(res)
                } else {
                    Ok(ExecResult::Normal)
                }
            }

            StatementKind::LoopForever(body) => {
                loop {
                    self.env.push();
                    match self.exec_block(body)? {
                        ExecResult::Normal => {}
                        ExecResult::Break => {
                            self.env.pop();
                            break;
                        }
                        ExecResult::Continue => {}
                        ret @ ExecResult::Return(_) => {
                            self.env.pop();
                            return Ok(ret);
                        }
                    }
                    self.env.pop();
                }
                Ok(ExecResult::Normal)
            }

            StatementKind::LoopWhile { condition, body } => {
                while self.eval_expr(condition)?.to_bool(condition, &*self.file)? {
                    self.env.push();
                    match self.exec_block(body)? {
                        ExecResult::Normal => {}
                        ExecResult::Break => {
                            self.env.pop();
                            break;
                        }
                        ExecResult::Continue => {}
                        ret @ ExecResult::Return(_) => {
                            self.env.pop();
                            return Ok(ret);
                        }
                    }
                    self.env.pop();
                }
                Ok(ExecResult::Normal)
            }

            StatementKind::LoopRange {
                var,
                start,
                end,
                body,
            } => {
                let s = match self.eval_expr(start)? {
                    Value::Int(i) => i,
                    _ => {
                        return Err(PawError::Type {
                            file: self.file.clone(),
                            code: "E4001".into(),
                            message: "Range start not Int".into(),
                            line: 0,
                            column: 0,
                            snippet: None,
                            hint: Some("Use an Int for range start.".into()),
                        })
                    }
                };
                let e = match self.eval_expr(end)? {
                    Value::Int(i) => i,
                    _ => {
                        return Err(PawError::Type {
                            file: self.file.clone(),
                            code: "E4001".into(),
                            message: "Range end not Int".into(),
                            line: 0,
                            column: 0,
                            snippet: None,
                            hint: Some("Use an Int for range end.".into()),
                        })
                    }
                };
                for i in s..e {
                    self.env.push();
                    self.env.define(var.clone(), Value::Int(i));
                    match self.exec_block(body)? {
                        ExecResult::Normal => {}
                        ExecResult::Break => {
                            self.env.pop();
                            break;
                        }
                        ExecResult::Continue => {}
                        ret @ ExecResult::Return(_) => {
                            self.env.pop();
                            return Ok(ret);
                        }
                    }
                    self.env.pop();
                }
                Ok(ExecResult::Normal)
            }

            StatementKind::FunDecl { .. } => Ok(ExecResult::Normal),

            StatementKind::Block(stmts) => {
                self.env.push();
                let res = self.exec_block(stmts)?;
                self.env.pop();
                Ok(res)
            }

            StatementKind::Throw(expr) => {
                let v = self.eval_expr(expr)?;
                Err(PawError::Codegen {
                    file: self.file.clone(),
                    code: "E6001".into(),
                    message: format!("{:?}", v),
                    line: 0,
                    column: 0,
                    snippet: None,
                    hint: Some("Uncaught exception.".into()),
                })
            }

            StatementKind::TryCatchFinally {
                body,
                err_name,
                handler,
                finally,
            } => {
                let try_res = (|| -> Result<ExecResult, PawError> {
                    for s in body {
                        self.exec_stmt(s)?;
                    }
                    Ok(ExecResult::Normal)
                })();

                if let Err(err) = try_res {
                    let msg = err.to_string();
                    self.env.define(err_name.clone(), Value::String(msg));
                    for s in handler {
                        self.exec_stmt(s)?;
                    }
                }

                for s in finally {
                    self.exec_stmt(s)?;
                }

                Ok(ExecResult::Normal)
            }
        }
    }

    /// 计算表达式的值
    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, PawError> {
        match &expr.kind {
            ExprKind::LiteralInt(i) => Ok(Value::Int(*i)),
            ExprKind::LiteralLong(l) => Ok(Value::Long(*l)),
            ExprKind::LiteralFloat(f) => Ok(Value::Float(*f)),
            ExprKind::LiteralString(s) => Ok(Value::String(s.clone())),
            ExprKind::LiteralBool(b) => Ok(Value::Bool(*b)),
            ExprKind::LiteralChar(c) => Ok(Value::Char(*c)),
            ExprKind::LiteralNopaw => Ok(Value::Null),

            ExprKind::Cast { expr: inner, ty } => {
                let v = self.eval_expr(inner)?;
                match (v, ty.as_str()) {
                    (Value::Int(i), "Float") => Ok(Value::Float(i as f64)),
                    (Value::Int(i), "Long") => Ok(Value::Long(i as i64)),
                    (Value::Long(l), "Float") => Ok(Value::Float(l as f64)),
                    (Value::Long(l), "Int") => Ok(Value::Int(l as i32)),
                    (Value::Float(f), "Int") => Ok(Value::Int(f as i32)),
                    (Value::Float(f), "Long") => Ok(Value::Long(f as i64)),
                    (Value::String(s), "Int") => {
                        let n = s.parse::<i32>().map_err(|_| PawError::Type {
                            file: self.file.clone(),
                            code: "E4001".into(),
                            message: format!("Cannot cast string '{}' to Int", s),
                            line: expr.line,
                            column: expr.col,
                            snippet: None,
                            hint: Some("Ensure the string contains a valid integer.".into()),
                        })?;
                        Ok(Value::Int(n))
                    }
                    (val, "String") => Ok(Value::String(val.to_string_value())),
                    (val, "Bool") => Ok(Value::Bool(val.to_bool(inner, &*self.file)?)),
                    (val, t) if format!("{:?}", val) == t => Ok(val),
                    (val, t) => Err(PawError::Type {
                        file: self.file.clone(),
                        code: "E4001".into(),
                        message: format!("Cannot cast {:?} to {}", val, t),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: None,
                    }),
                }
            }

            ExprKind::Var(name) => self
                .env
                .get(name)
                .ok_or_else(|| PawError::UndefinedVariable {
                    file: self.file.clone(),
                    code: "E2003".into(),
                    name: name.clone(),
                    line: expr.line,
                    column: expr.col,
                    snippet: None,
                    hint: Some("Check variable name or scope.".into()),
                }),

            ExprKind::UnaryOp { op, expr: inner } => {
                let v = self.eval_expr(inner)?;
                match (op.as_str(), v.clone()) {
                    ("-", Value::Int(i)) => Ok(Value::Int(-i)),
                    ("-", Value::Long(l)) => Ok(Value::Long(-l)),
                    ("-", Value::Float(f)) => Ok(Value::Float(-f)),
                    ("!", v) => Ok(Value::Bool(!v.to_bool(inner, &*self.file)?)),
                    _ => Err(PawError::Type {
                        file: self.file.clone(),
                        code: "E4001".into(),
                        message: format!("Bad unary `{}` on {:?}", op, v),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: None,
                    }),
                }
            }

            ExprKind::BinaryOp { op, left, right } => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                let val = match op {
                    BinaryOp::Add => {
                        if let Value::String(a) = l.clone() {
                            return Ok(Value::String(a + &r.to_string_value()));
                        }
                        if let Value::String(b) = r.clone() {
                            return Ok(Value::String(l.to_string_value() + &b));
                        }
                        match (l, r) {
                            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                            (Value::Long(a), Value::Long(b)) => Ok(Value::Long(a + b)),
                            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                            _ => Err(PawError::Type {
                                file: self.file.clone(),
                                code: "E4001".into(),
                                message: "Bad + operands".into(),
                                line: expr.line,
                                column: expr.col,
                                snippet: None,
                                hint: None,
                            }),
                        }
                    }
                    BinaryOp::Sub => Ok(match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
                        (Value::Long(a), Value::Long(b)) => Value::Long(a - b),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                        _ => {
                            return Err(PawError::Type {
                                file: self.file.clone(),
                                code: "E4001".into(),
                                message: "Bad - operands".into(),
                                line: expr.line,
                                column: expr.col,
                                snippet: None,
                                hint: None,
                            })
                        }
                    }),
                    BinaryOp::Mul => Ok(match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
                        (Value::Long(a), Value::Long(b)) => Value::Long(a * b),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
                        _ => {
                            return Err(PawError::Type {
                                file: self.file.clone(),
                                code: "E4001".into(),
                                message: "Bad * operands".into(),
                                line: expr.line,
                                column: expr.col,
                                snippet: None,
                                hint: None,
                            })
                        }
                    }),
                    BinaryOp::Div => Ok(match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a / b),
                        (Value::Long(a), Value::Long(b)) => Value::Long(a / b),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
                        _ => {
                            return Err(PawError::Type {
                                file: self.file.clone(),
                                code: "E4001".into(),
                                message: "Bad / operands".into(),
                                line: expr.line,
                                column: expr.col,
                                snippet: None,
                                hint: None,
                            })
                        }
                    }),
                    BinaryOp::Mod => Ok(match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a % b),
                        (Value::Long(a), Value::Long(b)) => Value::Long(a % b),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a % b),
                        _ => {
                            return Err(PawError::Type {
                                file: self.file.clone(),
                                code: "E4001".into(),
                                message: "Bad % operands".into(),
                                line: expr.line,
                                column: expr.col,
                                snippet: None,
                                hint: None,
                            })
                        }
                    }),
                    BinaryOp::EqEq => Ok(Value::Bool(l == r)),
                    BinaryOp::NotEq => Ok(Value::Bool(l != r)),
                    BinaryOp::Lt => Ok(Value::Bool(l < r)),
                    BinaryOp::Le => Ok(Value::Bool(l <= r)),
                    BinaryOp::Gt => Ok(Value::Bool(l > r)),
                    BinaryOp::Ge => Ok(Value::Bool(l >= r)),
                    BinaryOp::And => Ok(Value::Bool(l.to_bool(expr, &*self.file)? && r.to_bool(expr, &*self.file)?)),
                    BinaryOp::Or => Ok(Value::Bool(l.to_bool(expr, &*self.file)? || r.to_bool(expr, &*self.file)?)),
                    BinaryOp::As => {
                        return Err(PawError::Internal {
                            file: self.file.clone(),
                            code: "E5004".into(),
                            message: format!("Unhandled binary operator in interpreter: {:?}", op),
                            line: expr.line,
                            column: expr.col,
                            snippet: None,
                            hint: Some("This should have been lowered to a cast.".into()),
                        });
                    }
                };
                Ok(val?)
            }

            ExprKind::Call { name, args } => {
                let f = if let Some(f) = self.env.get(name) {
                    f
                } else if let Some((mod_name, member)) = name.split_once('.') {
                    match self.env.get(mod_name) {
                        Some(Value::Module(table)) => {
                            table.get(member).cloned().ok_or_else(|| {
                                PawError::UndefinedVariable {
                                    file: self.file.clone(),
                                    code: "E2003".into(),
                                    name: name.clone(),
                                    line: expr.line,
                                    column: expr.col,
                                    snippet: None,
                                    hint: Some("Check spelling or scope.".into()),
                                }
                            })?
                        }
                        _ => {
                            return Err(PawError::UndefinedVariable {
                                file: self.file.clone(),
                                code: "E2003".into(),
                                name: name.clone(),
                                line: expr.line,
                                column: expr.col,
                                snippet: None,
                                hint: Some("Check spelling or scope.".into()),
                            })
                        }
                    }
                } else {
                    return Err(PawError::UndefinedVariable {
                        file: self.file.clone(),
                        code: "E2003".into(),
                        name: name.clone(),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: Some("Check spelling or scope.".into()),
                    });
                };
                if let Value::Function {
                    params,
                    body,
                    env: fn_env,
                } = f
                {
                    if args.len() != params.len() {
                        return Err(PawError::Type {
                            file: self.file.clone(),
                            code: "E4001".into(),
                            message: "Arg count mismatch".into(),
                            line: expr.line,
                            column: expr.col,
                            snippet: None,
                            hint: None,
                        });
                    }
                    let mut sub = Interpreter {
                        env: fn_env.clone(),
                        file: self.file.clone(),
                    };
                    sub.env.push();
                    for (p, arg) in params.iter().zip(args.iter()) {
                        let v = self.eval_expr(arg)?;
                        sub.env.define(p.clone(), v);
                    }
                    let res = sub.exec_block(&body)?;
                    if let ExecResult::Return(v) = res {
                        Ok(v)
                    } else if let Some(last_stmt) = body.last() {
                        if let StatementKind::Expr(expr) = &last_stmt.kind {
                            sub.eval_expr(expr)
                        } else {
                            Ok(Value::Void)
                        }
                    } else {
                        Ok(Value::Void)
                    }
                } else {
                    Err(PawError::Type {
                        file: self.file.clone(),
                        code: "E4001".into(),
                        message: format!("{} is not a function", name),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: None,
                    })
                }
            }

            ExprKind::ArrayLiteral(elems) => {
                let mut vec = Vec::new();
                for e in elems {
                    vec.push(self.eval_expr(e)?);
                }
                Ok(Value::Array(vec))
            }

            ExprKind::Index { array, index } => {
                let arr = self.eval_expr(array)?;
                let idx = self.eval_expr(index)?;
                let i = match idx {
                    Value::Int(i) => i as usize,
                    _ => {
                        return Err(PawError::Type {
                            file: self.file.clone(),
                            code: "E4001".into(),
                            message: "Index not Int".into(),
                            line: expr.line,
                            column: expr.col,
                            snippet: None,
                            hint: Some("Use an Int index.".into()),
                        })
                    }
                };
                if let Value::Array(v) = arr {
                    v.get(i).cloned().ok_or_else(|| PawError::Internal {
                        file: self.file.clone(),
                        code: "E5005".into(),
                        message: "Index out of bounds".into(),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: None,
                    })
                } else {
                    Err(PawError::Type {
                        file: self.file.clone(),
                        code: "E4001".into(),
                        message: "Not an array".into(),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: None,
                    })
                }
            }

            ExprKind::Property { object, name } => {
                let obj_val = self.eval_expr(object)?;
                match obj_val {
                    Value::Array(v) if name == "length" => Ok(Value::Int(v.len() as i32)),
                    Value::String(v) if name == "length" => Ok(Value::Int(v.len() as i32)),
                    Value::Module(table) => {
                        table
                            .get(name)
                            .cloned()
                            .ok_or_else(|| PawError::UndefinedVariable {
                                file: self.file.clone(),
                                code: "E2003".into(),
                                name: format!("{}.{}", "<module>", name),
                                line: expr.line,
                                column: expr.col,
                                snippet: None,
                                hint: Some("Check module member name.".into()),
                            })
                    }
                    _ => Err(PawError::Type {
                        file: self.file.clone(),
                        code: "E4001".into(),
                        message: format!("Type {:?} has no property `{}`", obj_val, name),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: None,
                    }),
                }
            }
        }
    }
}

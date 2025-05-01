// src/interpreter.rs

use crate::ast::{BinaryOp, Expr, Statement, StatementKind};
use crate::lexer::lex::Lexer;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::{self, Write};
use crate::error::error::PawError;
use crate::parser::parser::Parser;

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
    fn to_bool(&self) -> Result<bool, PawError> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(PawError::Type {
                message: format!("Cannot convert {:?} to bool", self),
            }),
        }
    }

    fn to_string_value(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Int(i) => i.to_string(),
            Value::Long(l) => l.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Char(c) => c.to_string(),
            Value::Array(a) => format!("{:?}", a),
            _ => "<fn>".into(),
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
}

impl Env {
    pub fn new() -> Self {
        Env {
            scopes: vec![HashMap::new()],
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
        // 向上查找第一个定义该变量的作用域并更新
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.into(), val);
                return Ok(());
            }
        }
        Err(PawError::UndefinedVariable { name: name.into() })
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
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter { env: Env::new() }
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
                    message: format!("cannot import {:?}: {}", path, e),
                })?;

                // 重新编译运行子模块
                let tokens = Lexer::new(&src).tokenize();
                let mut p = Parser::new(tokens);
                let ast = p.parse_program()?;
                let mut tc = crate::semantic::type_checker::TypeChecker::new();
                tc.check_statements(&ast)?;
                let mut sub = Interpreter::new();
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
                // 最关键：给 alias 自身绑定一个 Module 值
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
                self.env.set(name, v)?; // 更新已经存在的变量
                Ok(ExecResult::Normal)
            }
            StatementKind::Ask {
                name,
                ty: _,
                prompt,
            } => {
                print!("{}", prompt);
                io::stdout().flush().map_err(|e| PawError::Internal {
                    message: e.to_string(),
                })?;
                let mut line = String::new();
                io::stdin()
                    .read_line(&mut line)
                    .map_err(|e| PawError::Internal {
                        message: e.to_string(),
                    })?;
                // 简化：只支持读字符串
                self.env.define(name.clone(), Value::String(line.into()));
                Ok(ExecResult::Normal)
            }
            StatementKind::AskPrompt(prompt) => {
                // 仅提示，不保存
                print!("{}", prompt);
                io::stdout().flush().map_err(|e| PawError::Internal {
                    message: e.to_string(),
                })?;
                let mut line = String::new();
                io::stdin()
                    .read_line(&mut line)
                    .map_err(|e| PawError::Internal {
                        message: e.to_string(),
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
                if c.to_bool()? {
                    self.env.push();
                    let res = self.exec_block(body)?;
                    self.env.pop();
                    Ok(res)
                } else if let Some(else_stmt) = else_branch {
                    // else_branch 是 Box<Statement>
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
                while self.eval_expr(condition)?.to_bool()? {
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
                            message: "Range start not Int".into(),
                        })
                    }
                };
                let e = match self.eval_expr(end)? {
                    Value::Int(i) => i,
                    _ => {
                        return Err(PawError::Type {
                            message: "Range end not Int".into(),
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
            StatementKind::FunDecl { .. } => {
                // 已在 run() 里提前注册，这里跳过
                Ok(ExecResult::Normal)
            }
            StatementKind::Block(stmts) => {
                self.env.push();
                let res = self.exec_block(stmts)?;
                self.env.pop();
                Ok(res)
            }
            StatementKind::Throw(expr) => {
                // 先求值
                let v = self.eval_expr(expr)?;
                // 我们把所有异常都当字符串处理
                Err(PawError::Codegen {
                    message: format!("{:?}", v),
                })
            }
            StatementKind::TryCatchFinally {
                body,
                err_name,
                handler,
                finally,
            } => {
                // 用闭包捕获 Err
                let try_res = (|| -> Result<ExecResult, PawError> {
                    for s in body {
                        self.exec_stmt(s)?; // 如果中间有 Err，会立即return Err
                    }
                    Ok(ExecResult::Normal) // 正常完成
                })();

                match try_res {
                    Ok(_) => {
                        // nothing to do: try 块正常结束
                    }
                    Err(err) => {
                        // 把 err 转为字符串绑定到 err_name
                        let msg = err.to_string();
                        self.env.define(err_name.clone(), Value::String(msg));
                        // 执行 handler（snatch）块
                        for s in handler {
                            self.exec_stmt(s)?;
                        }
                    }
                }

                // 无论 try 是否抛错，都执行 lastly
                for s in finally {
                    self.exec_stmt(s)?;
                }

                Ok(ExecResult::Normal)
            }
        }
    }

    /// 计算表达式的值
    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, PawError> {
        match expr {
            Expr::LiteralInt(i) => Ok(Value::Int(*i)),
            Expr::LiteralLong(l) => Ok(Value::Long(*l)),
            Expr::LiteralFloat(f) => Ok(Value::Float(*f)),
            Expr::LiteralString(s) => Ok(Value::String(s.clone())),
            Expr::LiteralBool(b) => Ok(Value::Bool(*b)),
            Expr::LiteralChar(c) => Ok(Value::Char(*c)),
            Expr::Cast { expr: inner, ty } => {
                let v = self.eval_expr(inner)?;
                match (v, ty.as_str()) {
                    (Value::Int(i), "Float") => Ok(Value::Float(i as f64)),
                    (Value::Int(i), "Long") => Ok(Value::Long(i as i64)),
                    (Value::Long(l), "Float") => Ok(Value::Float(l as f64)),
                    (Value::Long(l), "Int") => Ok(Value::Int(l as i32)),
                    (Value::Float(f), "Int") => Ok(Value::Int(f as i32)),
                    (Value::Float(f), "Long") => Ok(Value::Long(f as i64)),
                    // string ↔ char conversions, etc. if you wish…
                    (Value::String(s), "Int") => {
                        let n = s.parse::<i32>().map_err(|_| PawError::Type {
                            message: format!("Cannot cast string '{}' to Int", s),
                        })?;
                        Ok(Value::Int(n))
                    }
                    // casting to String: call to_string()
                    (val, "String") => Ok(Value::String(val.to_string_value())),
                    // casting to Bool?
                    (val, "Bool") => Ok(Value::Bool(val.to_bool()?)),
                    // if target equals value’s own type, just pass through
                    (val, t) if format!("{:?}", val) == t => Ok(val),
                    // otherwise error
                    (val, t) => Err(PawError::Type {
                        message: format!("Cannot cast {:?} to {}", val, t),
                    }),
                }
            }
            Expr::Var(name) => self
                .env
                .get(name)
                .ok_or_else(|| PawError::UndefinedVariable { name: name.clone() }),
            Expr::UnaryOp { op, expr: inner } => {
                let v = self.eval_expr(inner)?;
                match (op.as_str(), v.clone()) {
                    ("-", Value::Int(i)) => Ok(Value::Int(-i)),
                    ("-", Value::Long(l)) => Ok(Value::Long(-l)),
                    ("-", Value::Float(f)) => Ok(Value::Float(-f)),
                    ("!", v) => Ok(Value::Bool(!v.to_bool()?)),
                    _ => Err(PawError::Type {
                        message: format!("Bad unary `{}` on {:?}", op, v),
                    }),
                }
            }
            Expr::BinaryOp { op, left, right } => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                let val = match op {
                    BinaryOp::Add => {
                        // —— 支持字符串拼接 ——
                        // 如果左侧是字符串，直接把右侧也 to_string 并拼接
                        if let Value::String(a) = l.clone() {
                            return Ok(Value::String(a + &r.to_string_value()));
                        }
                        // 如果右侧是字符串，同理
                        if let Value::String(b) = r.clone() {
                            return Ok(Value::String(l.to_string_value() + &b));
                        }
                        // 否则回退到数值加法
                        match (l, r) {
                            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                            (Value::Long(a), Value::Long(b)) => Ok(Value::Long(a + b)),
                            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                            _ => Err(PawError::Type {
                                message: "Bad + operands".into(),
                            }),
                        }
                    }
                    BinaryOp::Sub => Ok(match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
                        (Value::Long(a), Value::Long(b)) => Value::Long(a - b),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                        _ => {
                            return Err(PawError::Type {
                                message: "Bad - operands".into(),
                            })
                        }
                    }),
                    BinaryOp::Mul => Ok(match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
                        (Value::Long(a), Value::Long(b)) => Value::Long(a * b),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a * b),
                        _ => {
                            return Err(PawError::Type {
                                message: "Bad * operands".into(),
                            })
                        }
                    }),
                    BinaryOp::Div => Ok(match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a / b),
                        (Value::Long(a), Value::Long(b)) => Value::Long(a / b),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
                        _ => {
                            return Err(PawError::Type {
                                message: "Bad / operands".into(),
                            })
                        }
                    }),
                    BinaryOp::Mod => Ok(match (l, r) {
                        (Value::Int(a), Value::Int(b)) => Value::Int(a % b),
                        (Value::Long(a), Value::Long(b)) => Value::Long(a % b),
                        (Value::Float(a), Value::Float(b)) => Value::Float(a % b),
                        _ => {
                            return Err(PawError::Type {
                                message: "Bad % operands".into(),
                            })
                        }
                    }),

                    BinaryOp::EqEq => Ok(Value::Bool(l == r)),
                    BinaryOp::NotEq => Ok(Value::Bool(l != r)),
                    BinaryOp::Lt => Ok(Value::Bool(l < r)),
                    BinaryOp::Le => Ok(Value::Bool(l <= r)),
                    BinaryOp::Gt => Ok(Value::Bool(l > r)),
                    BinaryOp::Ge => Ok(Value::Bool(l >= r)),
                    BinaryOp::And => Ok(Value::Bool(l.to_bool()? && r.to_bool()?)),
                    BinaryOp::Or => Ok(Value::Bool(l.to_bool()? || r.to_bool()?)),
                    BinaryOp::As => {
                        // this really ought never happen here,
                        // because you lowered `As` into Expr::Cast already.
                        return Err(PawError::Internal {
                            message: format!("Unhandled binary operator in interpreter: {:?}", op),
                        });
                    }
                };
                Ok(val?)
            }
            Expr::Call { name, args } => {
                // 取出函数值
                let f = if let Some(f) = self.env.get(name) {
                    f
                } else if let Some((mod_name, member)) = name.split_once('.') {
                    // 再试 “模块.成员” 形式：先拿模块值，再从它的 table 里取成员
                    match self.env.get(mod_name) {
                        Some(Value::Module(table)) => table
                            .get(member)
                            .cloned()
                            .ok_or_else(|| PawError::UndefinedVariable { name: name.clone() })?,
                        _ => return Err(PawError::UndefinedVariable { name: name.clone() }),
                    }
                } else {
                    return Err(PawError::UndefinedVariable { name: name.clone() });
                };
                if let Value::Function {
                    params,
                    body,
                    env: fn_env,
                } = f
                {
                    if args.len() != params.len() {
                        return Err(PawError::Type {
                            message: "Arg count mismatch".into(),
                        });
                    }
                    // new interpreter 用函数定义时的 env 作为闭包环境
                    let mut sub = Interpreter {
                        env: fn_env.clone(),
                    };
                    sub.env.push();
                    for (p, arg) in params.iter().zip(args.iter()) {
                        let v = self.eval_expr(arg)?;
                        sub.env.define(p.clone(), v);
                    }
                    let res = sub.exec_block(&body)?;
                    // 如果中途有显式 return，就返回它
                    if let ExecResult::Return(v) = res {
                        Ok(v)
                    } else {
                        // 否则尝试隐式返回最后一条 Expr 语句的值
                        if let Some(last_stmt) = body.last() {
                            if let StatementKind::Expr(expr) = &last_stmt.kind {
                                sub.eval_expr(expr)
                            } else {
                                Ok(Value::Void)
                            }
                        } else {
                            Ok(Value::Void)
                        }
                    }
                } else {
                    Err(PawError::Type {
                        message: format!("{} is not a function", name),
                    })
                }
            }
            Expr::ArrayLiteral(elems) => {
                let mut vec = Vec::new();
                for e in elems {
                    vec.push(self.eval_expr(e)?);
                }
                Ok(Value::Array(vec))
            }
            Expr::Index { array, index } => {
                let arr = self.eval_expr(array)?;
                let idx = self.eval_expr(index)?;
                let i = match idx {
                    Value::Int(i) => i as usize,
                    _ => {
                        return Err(PawError::Type {
                            message: "Index not Int".into(),
                        })
                    }
                };
                if let Value::Array(mut v) = arr {
                    v.get(i).cloned().ok_or_else(|| PawError::Internal {
                        message: "Index out of bounds".into(),
                    })
                } else {
                    Err(PawError::Type {
                        message: "Not an array".into(),
                    })
                }
            }
            Expr::Property { object, name } => {
                let obj_val = self.eval_expr(object)?;
                match obj_val {
                    // （1）数组喝字符串的 length
                    Value::Array(v) if name == "length" => Ok(Value::Int(v.len() as i32)),
                    Value::String(v) if name == "length" => Ok(Value::Int(v.len() as i32)),

                    // （2）模块导出的成员：m.square / m.PI / …
                    Value::Module(table) => table
                        .get(name) // 查表
                        .cloned()
                        .ok_or_else(|| PawError::UndefinedVariable {
                            name: format!("{}.{}", "<module>", name),
                        }),

                    // 其他类型暂不支持属性访问
                    _ => Err(PawError::Type {
                        message: format!("Type {:?} has no property `{}`", obj_val, name),
                    }),
                }
            }
        }
    }
}

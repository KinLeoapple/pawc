// src/interpreter/interpreter.rs

use crate::ast::expr::{Expr, ExprKind};
use crate::ast::statement::{Statement, StatementKind};
use crate::error::error::PawError;
use crate::interpreter::env::Env;
use crate::interpreter::value::Value;
use crate::lexer::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::semantic::type_checker::TypeChecker;
use std::future::Future;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::pin::Pin;

/// 主解释器
pub struct Interpreter {
    pub env: Env,
    pub file: String,
}

impl Interpreter {
    /// 创建一个新的解释器实例
    pub fn new(env: Env, file: &str) -> Self {
        Interpreter {
            env,
            file: file.to_string(),
        }
    }

    /// 执行多条语句，遇到 return/throw 提前返回
    pub fn eval_statements<'a>(
        &'a mut self,
        stmts: &'a [Statement],
    ) -> Pin<Box<dyn Future<Output = Result<Option<Value>, PawError>> + 'a>> {
        Box::pin(async move {
            for stmt in stmts {
                if let Some(v) = self.eval_statement(stmt).await? {
                    return Ok(Some(v));
                }
            }
            Ok(None)
        })
    }

    /// 执行单条语句
    pub fn eval_statement<'a>(
        &'a mut self,
        stmt: &'a Statement,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Value>, PawError>> + 'a>> {
        Box::pin(async move {
            match &stmt.kind {
                StatementKind::Let { name, ty: _, value } => {
                    let v = self.eval_expr(value).await?;
                    self.env.define(name.clone(), v);
                    Ok(None)
                }

                StatementKind::Assign { name, value } => {
                    let v = self.eval_expr(value).await?;
                    self.env.assign(name, v)?;
                    Ok(None)
                }

                StatementKind::Say(expr) => {
                    let v = self.eval_expr(expr).await?;
                    println!("{:?}", v);
                    Ok(None)
                }

                StatementKind::Ask {
                    name,
                    ty: _,
                    prompt,
                } => {
                    print!("{}", prompt);
                    // 确保 prompt 立刻显示在终端
                    use std::io::Write;
                    let _ = std::io::stdout().flush();
                    let mut buf = String::new();
                    let _ = std::io::stdin().read_line(&mut buf);

                    self.env
                        .define(name.clone(), Value::String(buf.trim_end().to_string()));

                    Ok(None)
                }

                StatementKind::AskPrompt(prompt) => {
                    print!("{}", prompt);
                    // 同样要 flush
                    use std::io::Write;
                    let _ = std::io::stdout().flush();
                    let mut buf = String::new();
                    let _ = std::io::stdin().read_line(&mut buf);
                    Ok(None)
                }

                StatementKind::Import { module, alias } => {
                    // 1. 拼出文件路径
                    let base_path = Path::new(&self.file);
                    let mut path = PathBuf::new();
                    path.push(base_path.parent().unwrap_or(Path::new(".")));
                    for seg in module {
                        path.push(seg);
                    }
                    path.set_extension("paw");

                    // 2. 读源码
                    let src = std::fs::read_to_string(&path).map_err(|e| {
                        // 根据 kind 构造英文提示
                        let message = match e.kind() {
                            ErrorKind::NotFound => {
                                format!("Module file not found: {}", path.display())
                            }
                            ErrorKind::PermissionDenied => {
                                format!("Permission denied reading module file: {}", path.display())
                            }
                            _ => format!("Failed to read module file: {}", path.display()),
                        };
                        PawError::Internal {
                            file: self.file.clone(),
                            code: "E1002".into(),
                            message,
                            line: 0,
                            column: 0,
                            snippet: None,
                            hint: Some(
                                "Check that the module file exists and the path is correct".into(),
                            ),
                        }
                    })?;

                    // 3. 词法 & 解析
                    let tokens = Lexer::new(&src).tokenize();
                    let mut parser = Parser::new(tokens, &src, &*path.to_string_lossy());
                    let stmts = parser.parse_program()?;

                    // 4. 语义检查
                    let mut checker = TypeChecker::new(&*path.to_string_lossy());
                    checker.check_program(&stmts)?;

                    // 5. 执行模块
                    let module_env = Env::with_parent(&self.env);
                    let mut module_interp =
                        Interpreter::new(module_env.clone(), &*path.to_string_lossy());
                    let _ = module_interp.eval_statements(&stmts).await?;

                    // 6. 收集子环境所有顶层绑定，打包成 Module
                    let module_val = {
                        let m = module_env.bindings();
                        Value::Module(m)
                    };

                    self.env.define(alias.clone(), module_val);
                    Ok(None)
                }

                StatementKind::Return(opt) => {
                    let v = if let Some(e) = opt {
                        self.eval_expr(e).await?
                    } else {
                        Value::Null
                    };
                    Ok(Some(v))
                }

                StatementKind::Break => Ok(Some(Value::Bool(true))),
                StatementKind::Continue => Ok(Some(Value::Bool(false))),

                StatementKind::Expr(expr) => {
                    let _ = self.eval_expr(expr).await?;
                    Ok(None)
                }

                StatementKind::If {
                    condition,
                    body,
                    else_branch,
                } => {
                    let cond = self.eval_expr(condition).await?;
                    if let Value::Bool(true) = cond {
                        let res = self.eval_statements(body).await?;
                        if res.is_some() {
                            return Ok(res);
                        }
                    } else if let Some(else_stmt) = else_branch {
                        let res = self.eval_statement(else_stmt).await?;
                        if res.is_some() {
                            return Ok(res);
                        }
                    }
                    Ok(None)
                }

                StatementKind::LoopForever(body) => loop {
                    let res = self.eval_statements(body).await?;
                    if res.is_some() {
                        return Ok(res);
                    }
                },

                StatementKind::LoopWhile { condition, body } => {
                    while let Value::Bool(true) = self.eval_expr(condition).await? {
                        let res = self.eval_statements(body).await?;
                        if res.is_some() {
                            return Ok(res);
                        }
                    }
                    Ok(None)
                }

                StatementKind::LoopRange {
                    var,
                    start,
                    end,
                    body,
                } => {
                    let s = self.eval_expr(start).await?;
                    let e = self.eval_expr(end).await?;
                    if let (Value::Int(si), Value::Int(ei)) = (s, e) {
                        for i in si..ei {
                            self.env.define(var.clone(), Value::Int(i));
                            let res = self.eval_statements(body).await?;
                            if res.is_some() {
                                return Ok(res);
                            }
                        }
                    }
                    Ok(None)
                }

                StatementKind::FunDecl {
                    name,
                    params,
                    return_type: _,
                    is_async,
                    body,
                } => {
                    let func = Value::Function {
                        name: name.clone(),
                        params: params.clone(),
                        body: body.clone(),
                        env: self.env.clone(),
                        is_async: *is_async,
                    };
                    self.env.define(name.clone(), func);
                    Ok(None)
                }

                StatementKind::Block(stmts) => {
                    let child_env = Env::with_parent(&self.env);
                    let mut child = Interpreter::new(child_env, &self.file);
                    let _ = child.eval_statements(stmts).await?;
                    Ok(None)
                }

                StatementKind::TryCatchFinally {
                    body,
                    err_name,
                    handler,
                    finally,
                } => {
                    // try
                    let try_res = {
                        let mut ti = Interpreter::new(Env::with_parent(&self.env), &self.file);
                        ti.eval_statements(body).await
                    };
                    match try_res {
                        Ok(Some(v)) => return Ok(Some(v)),
                        Ok(None) => { /* 正常 */ }
                        Err(err) => {
                            if let PawError::Runtime { message, .. } = err {
                                // catch
                                let mut ci =
                                    Interpreter::new(Env::with_parent(&self.env), &self.file);
                                ci.env.define(err_name.clone(), Value::String(message));
                                let catch_r = ci.eval_statements(handler).await?;
                                // finally
                                let _ = Interpreter::new(Env::with_parent(&self.env), &self.file)
                                    .eval_statements(finally)
                                    .await?;
                                return Ok(catch_r);
                            } else {
                                return Err(err);
                            }
                        }
                    }
                    // finally after normal
                    let _ = Interpreter::new(Env::with_parent(&self.env), &self.file)
                        .eval_statements(finally)
                        .await?;
                    Ok(None)
                }

                StatementKind::RecordDecl { .. } => Ok(None),

                StatementKind::Throw(expr) => {
                    let v = self.eval_expr(expr).await?;
                    Err(PawError::Runtime {
                        file: self.file.clone(),
                        code: "E6001",
                        message: format!("{:?}", v),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: Some("Uncaught exception".into()),
                    })
                }
            }
        })
    }

    /// 计算表达式，返回一个可 await 的 Future
    pub fn eval_expr<'a>(
        &'a mut self,
        expr: &'a Expr,
    ) -> Pin<Box<dyn Future<Output = Result<Value, PawError>> + 'a>> {
        Box::pin(async move {
            match &expr.kind {
                ExprKind::LiteralInt(n) => Ok(Value::Int(*n)),
                ExprKind::LiteralLong(n) => Ok(Value::Long(*n)),
                ExprKind::LiteralFloat(f) => Ok(Value::Float(*f)),
                ExprKind::LiteralDouble(f) => Ok(Value::Double(*f)),
                ExprKind::LiteralString(s) => Ok(Value::String(s.clone())),
                ExprKind::LiteralBool(b) => Ok(Value::Bool(*b)),
                ExprKind::LiteralChar(c) => Ok(Value::Char(*c)),
                ExprKind::LiteralNopaw => Ok(Value::Null),

                ExprKind::Var(name) => {
                    self.env
                        .get(name.as_str())
                        .ok_or_else(|| PawError::UndefinedVariable {
                            file: self.file.clone(),
                            code: "E4001",
                            name: name.clone(),
                            line: expr.line,
                            column: expr.col,
                            snippet: None,
                            hint: Some("Did you declare this variable before use?".into()),
                        })
                }

                ExprKind::UnaryOp { op, expr: inner } => {
                    let v = self.eval_expr(inner).await?;
                    match op.as_str() {
                        "-" => match v {
                            Value::Int(i) => Ok(Value::Int(-i)),
                            Value::Long(l) => Ok(Value::Long(-l)),
                            Value::Float(f) => Ok(Value::Float(-f)),
                            other => Err(PawError::Runtime {
                                file: self.file.clone(),
                                code: "E3013",
                                message: format!("Bad unary `{}` on {:?}", op, other),
                                line: expr.line,
                                column: expr.col,
                                snippet: None,
                                hint: None,
                            }),
                        },
                        "!" => match v {
                            Value::Bool(b) => Ok(Value::Bool(!b)),
                            other => Err(PawError::Runtime {
                                file: self.file.clone(),
                                code: "E3013",
                                message: format!("Bad unary `{}` on {:?}", op, other),
                                line: expr.line,
                                column: expr.col,
                                snippet: None,
                                hint: None,
                            }),
                        },
                        _ => Err(PawError::Internal {
                            file: self.file.clone(),
                            code: "E6002",
                            message: format!("Unknown unary operator `{}`", op),
                            line: expr.line,
                            column: expr.col,
                            snippet: None,
                            hint: None,
                        }),
                    }
                }

                ExprKind::BinaryOp { op, left, right } => {
                    // 先 await 两边
                    let l = self.eval_expr(left).await?;
                    let r = self.eval_expr(right).await?;
                    use crate::ast::expr::BinaryOp::*;
                    use Value::*;

                    let result = match (op, l, r) {
                        // —— 字符串拼接 ——
                        (Add, String(a), String(b)) => String(a + &b),
                        (Add, String(a), other) => String(a + &format!("{:?}", other)),
                        (Add, other, String(b)) => String(format!("{:?}", other) + &b),

                        // —— 同类型基本情形 ——
                        (Add, Int(a), Int(b)) => Int(a + b),
                        (Add, Long(a), Long(b)) => Long(a + b),
                        (Add, Float(a), Float(b)) => Float(a + b),
                        (Add, Double(a), Double(b)) => Double(a + b),

                        (Sub, Int(a), Int(b)) => Int(a - b),
                        (Sub, Long(a), Long(b)) => Long(a - b),
                        (Sub, Float(a), Float(b)) => Float(a - b),
                        (Sub, Double(a), Double(b)) => Double(a - b),

                        (Mul, Int(a), Int(b)) => Int(a * b),
                        (Mul, Long(a), Long(b)) => Long(a * b),
                        (Mul, Float(a), Float(b)) => Float(a * b),
                        (Mul, Double(a), Double(b)) => Double(a * b),

                        (Div, Int(a), Int(b)) => Int(a / b),
                        (Div, Long(a), Long(b)) => Long(a / b),
                        (Div, Float(a), Float(b)) => Float(a / b),
                        (Div, Double(a), Double(b)) => Double(a / b),

                        (Mod, Int(a), Int(b)) => Int(a % b),
                        (Mod, Long(a), Long(b)) => Long(a % b),

                        // —— 混合 Int ↔ Float/Double ——
                        (Add, Int(a), Float(b)) => Float(a as f32 + b),
                        (Add, Float(a), Int(b)) => Float(a + b as f32),
                        (Add, Int(a), Double(b)) => Double(a as f64 + b),
                        (Add, Double(a), Int(b)) => Double(a + b as f64),
                        (Add, Long(a), Float(b)) => Float(a as f32 + b),
                        (Add, Float(a), Long(b)) => Float(a + b as f32),
                        (Add, Long(a), Double(b)) => Double(a as f64 + b),
                        (Add, Double(a), Long(b)) => Double(a + b as f64),

                        (Sub, Int(a), Float(b)) => Float(a as f32 - b),
                        (Sub, Float(a), Int(b)) => Float(a - b as f32),
                        (Sub, Int(a), Double(b)) => Double(a as f64 - b),
                        (Sub, Double(a), Int(b)) => Double(a - b as f64),
                        (Sub, Long(a), Float(b)) => Float(a as f32 - b),
                        (Sub, Float(a), Long(b)) => Float(a - b as f32),
                        (Sub, Long(a), Double(b)) => Double(a as f64 - b),
                        (Sub, Double(a), Long(b)) => Double(a - b as f64),

                        (Mul, Int(a), Float(b)) => Float(a as f32 * b),
                        (Mul, Float(a), Int(b)) => Float(a * b as f32),
                        (Mul, Int(a), Double(b)) => Double(a as f64 * b),
                        (Mul, Double(a), Int(b)) => Double(a * b as f64),
                        (Mul, Long(a), Float(b)) => Float(a as f32 * b),
                        (Mul, Float(a), Long(b)) => Float(a * b as f32),
                        (Mul, Long(a), Double(b)) => Double(a as f64 * b),
                        (Mul, Double(a), Long(b)) => Double(a * b as f64),

                        (Div, Int(a), Float(b)) => Float(a as f32 / b),
                        (Div, Float(a), Int(b)) => Float(a / b as f32),
                        (Div, Int(a), Double(b)) => Double(a as f64 / b),
                        (Div, Double(a), Int(b)) => Double(a / b as f64),
                        (Div, Long(a), Float(b)) => Float(a as f32 / b),
                        (Div, Float(a), Long(b)) => Float(a / b as f32),
                        (Div, Long(a), Double(b)) => Double(a as f64 / b),
                        (Div, Double(a), Long(b)) => Double(a / b as f64),

                        // —— 其它运算不变 ——
                        (EqEq, a, b) => Bool(a == b),
                        (NotEq, a, b) => Bool(a != b),

                        (Lt, Int(a), Int(b)) => Bool(a < b),
                        (Lt, Long(a), Long(b)) => Bool(a < b),
                        (Lt, Float(a), Float(b)) => Bool(a < b),
                        (Lt, Double(a), Double(b)) => Bool(a < b),

                        (Le, Int(a), Int(b)) => Bool(a <= b),
                        (Le, Long(a), Long(b)) => Bool(a <= b),
                        (Le, Float(a), Float(b)) => Bool(a <= b),
                        (Le, Double(a), Double(b)) => Bool(a <= b),

                        (Gt, Int(a), Int(b)) => Bool(a > b),
                        (Gt, Long(a), Long(b)) => Bool(a > b),
                        (Gt, Float(a), Float(b)) => Bool(a > b),
                        (Gt, Double(a), Double(b)) => Bool(a > b),

                        (Ge, Int(a), Int(b)) => Bool(a >= b),
                        (Ge, Long(a), Long(b)) => Bool(a >= b),
                        (Ge, Float(a), Float(b)) => Bool(a >= b),
                        (Ge, Double(a), Double(b)) => Bool(a >= b),

                        (And, Bool(a), Bool(b)) => Bool(a && b),
                        (Or, Bool(a), Bool(b)) => Bool(a || b),

                        (As, _, right_val) => right_val, // 强制转换

                        // 不支持的组合
                        (_op, left_val, right_val) => {
                            return Err(PawError::Runtime {
                                file: self.file.clone(),
                                code: "E3014",
                                message: format!("Cannot {:?} and {:?}", left_val, right_val),
                                line: expr.line,
                                column: expr.col,
                                snippet: None,
                                hint: None,
                            })
                        }
                    };

                    Ok(result)
                }

                ExprKind::Call { name, args } => {
                    // Evaluate arguments sequentially
                    let mut arg_vals = Vec::with_capacity(args.len());
                    for e in args {
                        arg_vals.push(self.eval_expr(e).await?);
                    }
                    // Look up function
                    let func = self
                        .env
                        .get(name)
                        .ok_or_else(|| PawError::UndefinedVariable {
                            file: self.file.clone(),
                            code: "E4001",
                            name: name.clone(),
                            line: expr.line,
                            column: expr.col,
                            snippet: None,
                            hint: Some("Did you declare this function before use?".into()),
                        })?;

                    if let Value::Function {
                        params,
                        body,
                        env: fenv,
                        is_async,
                        name: _fname,
                    } = func
                    {
                        if is_async {
                            // build a future
                            let mut new_interp =
                                Interpreter::new(Env::with_parent(&fenv), &self.file);
                            // bind args
                            for (p, v) in params.iter().zip(arg_vals.into_iter()) {
                                new_interp.env.define(p.name.clone(), v);
                            }
                            // run body
                            if let Some(ret) = new_interp.eval_statements(&body).await? {
                                Ok(ret)
                            } else {
                                Ok(Value::Null)
                            }
                        } else {
                            // synchronous
                            let prev = self.env.clone();
                            let mut child = Interpreter::new(Env::with_parent(&fenv), &self.file);
                            for (p, v) in params.iter().zip(arg_vals.into_iter()) {
                                child.env.define(p.name.clone(), v);
                            }
                            let res = child.eval_statements(&body).await?;
                            self.env = prev;
                            Ok(res.unwrap_or(Value::Null))
                        }
                    } else {
                        Err(PawError::Runtime {
                            file: self.file.clone(),
                            code: "E4002",
                            message: format!("{} is not callable", name),
                            line: expr.line,
                            column: expr.col,
                            snippet: None,
                            hint: None,
                        })
                    }
                }

                ExprKind::Cast { expr: inner, ty: _ty } => {
                    let v = self.eval_expr(inner).await?;
                    Ok(v)
                }

                ExprKind::ArrayLiteral(elems) => {
                    let mut items = Vec::with_capacity(elems.len());
                    for e in elems {
                        items.push(self.eval_expr(e).await?);
                    }
                    Ok(Value::Array(items))
                }

                ExprKind::Index { array, index } => {
                    let arr = self.eval_expr(array).await?;
                    let idx = self.eval_expr(index).await?;
                    match (arr, idx) {
                        (Value::Array(v), Value::Int(i)) => {
                            Ok(v.get(i as usize).cloned().unwrap_or(Value::Null))
                        }
                        _ => Err(PawError::Runtime {
                            file: self.file.clone(),
                            code: "E3012",
                            message: "Cannot index into non-array or non-int index".into(),
                            line: expr.line,
                            column: expr.col,
                            snippet: None,
                            hint: None,
                        }),
                    }
                }

                ExprKind::RecordInit { name: _, fields } => {
                    let mut map = std::collections::HashMap::new();
                    for (fname, fexpr) in fields {
                        let v = self.eval_expr(fexpr).await?;
                        map.insert(fname.clone(), v);
                    }
                    Ok(Value::Record(map))
                }

                ExprKind::Await { expr: inner } => {
                    // 先 eval 出一个 Value
                    let val = self.eval_expr(inner).await?;
                    if let Value::Future(fut_arc) = val {
                        // 不再 unwrap，而是如果锁被毒化就返回 Internal 错误
                        let mut guard = fut_arc.lock().map_err(|e| PawError::Internal {
                            file: self.file.clone(),
                            code: "E6004",
                            message: format!("Failed to lock future mutex: {}", e),
                            line: expr.line,
                            column: expr.col,
                            snippet: None,
                            hint: Some("Future mutex was poisoned".into()),
                        })?;
                        // 然后 await 里面的 Future，若 Future 本身返回 Err，也会被 ? 向上传递
                        let result = guard.as_mut().await?;
                        Ok(result)
                    } else {
                        Ok(val)
                    }
                }

                ExprKind::FieldAccess { expr: inner, field } => {
                    let obj = self.eval_expr(inner).await?;
                    match obj {
                        Value::Record(map) => {
                            if let Some(v) = map.get(field) {
                                Ok(v.clone())
                            } else {
                                Err(PawError::Runtime {
                                    file: self.file.clone(),
                                    code: "E3015",
                                    message: format!("Record has no field '{}'", field),
                                    line: expr.line,
                                    column: expr.col,
                                    snippet: None,
                                    hint: None,
                                })
                            }
                        }
                        other => Err(PawError::Runtime {
                            file: self.file.clone(),
                            code: "E6003",
                            message: format!("Cannot access field '{}' on {:?}", field, other),
                            line: expr.line,
                            column: expr.col,
                            snippet: None,
                            hint: Some(format!("Type {:?} has no fields", other)),
                        }),
                    }
                }

                ExprKind::MethodCall {
                    receiver,
                    method,
                    args,
                } => {
                    // 1. Evaluate the receiver expression
                    let recv = self.eval_expr(receiver).await?;
                    // 2. Evaluate all argument expressions
                    let mut arg_vals = Vec::with_capacity(args.len());
                    for a in args {
                        arg_vals.push(self.eval_expr(a).await?);
                    }
                    // 3. Dispatch based on the receiver’s runtime type
                    match recv {
                        // ————— String methods —————
                        Value::String(s) => match method.as_str() {
                            "trim" if arg_vals.is_empty() => {
                                Ok(Value::String(s.trim().to_string()))
                            }
                            "to_uppercase" if arg_vals.is_empty() => {
                                Ok(Value::String(s.to_uppercase()))
                            }
                            "to_lowercase" if arg_vals.is_empty() => {
                                Ok(Value::String(s.to_lowercase()))
                            }
                            "length" if arg_vals.is_empty() => {
                                Ok(Value::Int(s.chars().count() as i32))
                            }
                            "starts_with" => {
                                if let [Value::String(ref p)] = &arg_vals[..] {
                                    Ok(Value::Bool(s.starts_with(p)))
                                } else {
                                    Err(PawError::Runtime {
                                        file: self.file.clone(),
                                        code: "E6003".into(),
                                        message: format!(
                                            "Method `starts_with` expects one string argument, got {:?}",
                                            arg_vals
                                        ),
                                        line: expr.line,
                                        column: expr.col,
                                        snippet: None,
                                        hint: Some("Use: someString.starts_with(otherString)".into()),
                                    })
                                }
                            }
                            "ends_with" => {
                                if let [Value::String(ref p)] = &arg_vals[..] {
                                    Ok(Value::Bool(s.ends_with(p)))
                                } else {
                                    Err(PawError::Runtime {
                                        file: self.file.clone(),
                                        code: "E6003".into(),
                                        message: format!(
                                            "Method `ends_with` expects one string argument, got {:?}",
                                            arg_vals
                                        ),
                                        line: expr.line,
                                        column: expr.col,
                                        snippet: None,
                                        hint: Some("Use: someString.ends_with(otherString)".into()),
                                    })
                                }
                            }
                            "contains" => {
                                if let [Value::String(ref p)] = &arg_vals[..] {
                                    Ok(Value::Bool(s.contains(p)))
                                } else {
                                    Err(PawError::Runtime {
                                        file: self.file.clone(),
                                        code: "E6003".into(),
                                        message: format!(
                                            "Method `contains` expects one string argument, got {:?}",
                                            arg_vals
                                        ),
                                        line: expr.line,
                                        column: expr.col,
                                        snippet: None,
                                        hint: Some("Use: someString.contains(otherString)".into()),
                                    })
                                }
                            }
                            _ => Err(PawError::Runtime {
                                file: self.file.clone(),
                                code: "E6003".into(),
                                message: format!("Cannot call method '{}' on String", method),
                                line: expr.line,
                                column: expr.col,
                                snippet: None,
                                hint: Some(format!("Type String has no method '{}'", method)),
                            }),
                        },

                        // ————— Array methods —————
                        Value::Array(mut v) => match method.as_str() {
                            "push" if matches!(&arg_vals[..], [_x]) => {
                                v.push(arg_vals[0].clone());
                                Ok(Value::Array(v))
                            }
                            "pop" if arg_vals.is_empty() => {
                                if let Some(x) = v.pop() {
                                    Ok(x) // 直接把元素作为 Value::<T> 返回
                                } else {
                                    // 数组空时抛出运行时错误
                                    Err(PawError::Runtime {
                                        file: self.file.clone(),
                                        code: "E3016".into(), // 你可以定义一个新的错误码
                                        message: "Cannot pop from empty array".into(),
                                        line: expr.line,
                                        column: expr.col,
                                        snippet: None,
                                        hint: Some(
                                            "Ensure array is non-empty before calling pop".into(),
                                        ),
                                    })
                                }
                            }
                            "length" if arg_vals.is_empty() => Ok(Value::Int(v.len() as i32)),
                            _ => Err(PawError::Runtime {
                                file: self.file.clone(),
                                code: "E6003".into(),
                                message: format!("Cannot call method '{}' on Array", method),
                                line: expr.line,
                                column: expr.col,
                                snippet: None,
                                hint: Some("Type Array has no such method or wrong args".into()),
                            }),
                        },

                        // ————— Module: property lookup or immediate call —————
                        Value::Module(ref mmap) => {
                            if let Some(member) = mmap.get(method) {
                                match member.clone() {
                                    Value::Function {
                                        params,
                                        body,
                                        env: fenv,
                                        is_async,
                                        name: _,
                                    } => {
                                        // Async function call
                                        if is_async {
                                            let mut new_i = Interpreter::new(
                                                Env::with_parent(&fenv),
                                                &self.file,
                                            );
                                            for (p, v) in params.iter().zip(arg_vals.into_iter()) {
                                                new_i.env.define(p.name.clone(), v);
                                            }
                                            if let Some(ret) = new_i.eval_statements(&body).await? {
                                                Ok(ret)
                                            } else {
                                                Ok(Value::Null)
                                            }
                                        }
                                        // Sync function call
                                        else {
                                            let saved = self.env.clone();
                                            let mut child = Interpreter::new(
                                                Env::with_parent(&fenv),
                                                &self.file,
                                            );
                                            for (p, v) in params.iter().zip(arg_vals.into_iter()) {
                                                child.env.define(p.name.clone(), v);
                                            }
                                            let res = child.eval_statements(&body).await?;
                                            self.env = saved;
                                            Ok(res.unwrap_or(Value::Null))
                                        }
                                    }
                                    // Non‐function: only zero‐arg property access
                                    _ if arg_vals.is_empty() => Ok(member.clone()),
                                    _ => Err(PawError::Runtime {
                                        file: self.file.clone(),
                                        code: "E6003".into(),
                                        message: format!(
                                            "Cannot call method '{}' on Module",
                                            method
                                        ),
                                        line: expr.line,
                                        column: expr.col,
                                        snippet: None,
                                        hint: Some(format!(
                                            "Type Module has no method '{}'",
                                            method
                                        )),
                                    }),
                                }
                            } else {
                                Err(PawError::Runtime {
                                    file: self.file.clone(),
                                    code: "E6005".into(),
                                    message: format!("Module has no member '{}'", method),
                                    line: expr.line,
                                    column: expr.col,
                                    snippet: None,
                                    hint: None,
                                })
                            }
                        }

                        // ————— Fallback for everything else —————
                        other => Err(PawError::Runtime {
                            file: self.file.clone(),
                            code: "E6003".into(),
                            message: format!("Cannot call method '{}' on {:?}", method, other),
                            line: expr.line,
                            column: expr.col,
                            snippet: None,
                            hint: Some(format!("Type {:?} has no method '{}'", other, method)),
                        }),
                    }
                }
            }
        })
    }
}

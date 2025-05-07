// src/interpreter/interpreter.rs

use crate::ast::expr::{Expr, ExprKind};
use crate::ast::method::Method;
use crate::ast::statement::{Statement, StatementKind};
use crate::error::error::PawError;
use crate::interpreter::env::Env;
use crate::interpreter::value::{Value, ValueInner};
use crate::lexer::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::semantic::type_checker::TypeChecker;
use ahash::AHashMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use vuot::{Stack, StacklessFn};

pub struct Interpreter<'local> {
    pub engine: Engine,
    pub statements: &'local [Statement]
}

impl<'a> StacklessFn<'a, Result<Option<Value>, PawError>> for Interpreter<'_> {
    async fn call(mut self, stack: Stack<'_>) -> Result<Option<Value>, PawError> {
        self.engine.eval_statements(stack, self.statements).await
    }
}

/// 主解释器
pub struct Engine {
    pub env: Env,
    pub file: String,
}

impl Engine {
    /// 创建一个新的解释器实例
    pub fn new(env: Env, file: &str) -> Self {
        Engine {
            env,
            file: file.to_string(),
        }
    }

    /// 执行多条语句，遇到 return/throw 提前返回
    pub async fn eval_statements<'a>(
        &mut self,
        stack: Stack<'a>,
        stmts: &[Statement],
    ) -> Result<Option<Value>, PawError> {
        for stmt in stmts {
            if let Some(v) = stack.run(self.eval_statement(stack, stmt)).await? {
                return Ok(Some(v));
            }
        }
        Ok(None)
    }

    /// 执行单条语句
    pub async fn eval_statement<'a>(
        &mut self,
        stack: Stack<'a>,
        stmt: &Statement) -> Result<Option<Value>, PawError> {
        match &stmt.kind {
            StatementKind::Let { name, ty: _, value } => {
                let v = stack.run(self.eval_expr(stack, value)).await?;
                self.env.define(name.clone(), v);
                Ok(None)
            }

            StatementKind::Assign { name, value } => {
                let v = stack.run(self.eval_expr(stack, value)).await?;
                self.env.assign(name, v)?;
                Ok(None)
            }

            StatementKind::Say(expr) => {
                let v = stack.run(self.eval_expr(stack, expr)).await?;
                println!("{}", v);
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
                    Engine::new(module_env.clone(), &*path.to_string_lossy());
                let _ = stack.run(module_interp.eval_statements(stack, &stmts)).await?;

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
                    stack.run(self.eval_expr(stack, e)).await?
                } else {
                    Value::Null()
                };
                Ok(Some(v))
            }

            StatementKind::Break => Ok(Some(Value::Bool(true))),
            StatementKind::Continue => Ok(Some(Value::Bool(false))),

            StatementKind::Expr(expr) => {
                let _ = stack.run(self.eval_expr(stack, expr)).await?;
                Ok(None)
            }

            StatementKind::If {
                condition,
                body,
                else_branch,
            } => {
                // 1. 先计算 condition
                let cond_val = stack.run(self.eval_expr(stack, condition)).await?;

                // 2. 解构出内部 Arc<ValueInner>
                if let Value(inner_arc) = cond_val.clone() {
                    // inner_arc: Arc<ValueInner>
                    if let ValueInner::Bool(true) = &*inner_arc {
                        // then 分支
                        if let Some(v) = stack.run(self.eval_statements(stack, body)).await? {
                            return Ok(Some(v));
                        }
                        // 如果 then 不返回值，跳到最后的 Ok(None)
                    } else if let Some(else_stmt) = else_branch {
                        // else 分支（或嵌套的 if-else）
                        if let Some(v) = stack.run(self.eval_statement(stack, else_stmt)).await? {
                            return Ok(Some(v));
                        }
                    }
                }

                // 3. 默认返回 None
                Ok(None)
            }

            StatementKind::LoopForever(body) => loop {
                let res = stack.run(self.eval_statements(stack, body)).await?;
                if res.is_some() {
                    return Ok(res);
                }
            },

            StatementKind::LoopWhile { condition, body } => {
                loop {
                    // 1. 先求出条件
                    let cond_val = stack.run(self.eval_expr(stack, condition)).await?;
                    // 2. 判断是否为 Bool(true)
                    if cond_val != Value::Bool(true) {
                        break;
                    }
                    // 3. 条件为真时执行循环体
                    if let Some(v) = stack.run(self.eval_statements(stack, body)).await? {
                        // 如果循环体里 return/break/continue 返回了值，就直接透传
                        return Ok(Some(v));
                    }
                    // 否则继续下一次循环
                }
                Ok(None)
            }

            StatementKind::LoopRange {
                var,
                start,
                end,
                body,
            } => {
                // 先分别计算 start、end
                let s_val = stack.run(self.eval_expr(stack, start)).await?;
                let e_val = stack.run(self.eval_expr(stack, end)).await?;

                use crate::interpreter::value::ValueInner;
                // 解构出两个 i32
                let (si, ei) = match (&*s_val.0, &*e_val.0) {
                    (ValueInner::Int(si), ValueInner::Int(ei)) => (*si, *ei),
                    // 如果不是 Int，就直接跳过循环
                    _ => return Ok(None),
                };

                // 执行范围循环
                for i in si..ei {
                    self.env.define(var.clone(), Value::Int(i));
                    if let Some(v) = stack.run(self.eval_statements(stack, body)).await? {
                        return Ok(Some(v));
                    }
                }
                Ok(None)
            }

            StatementKind::LoopArray { var, array, body } => {
                // 1. 求值出数组对象
                let arr_val = stack.run(self.eval_expr(stack, array)).await?;
                // 2. 必须是 Array，否则跳过
                let elems = match &*arr_val.0 {
                    ValueInner::Array(v_arc) => &**v_arc,
                    _ => return Ok(None),
                };
                // 3. 遍历每个元素
                for item in elems {
                    // 将循环变量绑定到当前环境
                    self.env.define(var.clone(), item.clone());
                    // 执行循环体，遇到 return/break/continue 即透传
                    if let Some(v) = stack.run(self.eval_statements(stack, body)).await? {
                        return Ok(Some(v));
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
                let func = Value::Function(
                    name.clone(),
                    params.clone(),
                    body.clone(),
                    self.env.clone(),
                    *is_async,
                );
                self.env.define(name.clone(), func);
                Ok(None)
            }

            StatementKind::Block(stmts) => {
                let child_env = Env::with_parent(&self.env);
                let mut child = Engine::new(child_env, &self.file);
                let _ = stack.run(child.eval_statements(stack, stmts)).await?;
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
                    let mut ti = Engine::new(Env::with_parent(&self.env), &self.file);
                    stack.run(ti.eval_statements(stack, body)).await
                };
                match try_res {
                    Ok(Some(v)) => return Ok(Some(v)),
                    Ok(None) => { /* 正常 */ }
                    Err(err) => {
                        return if let PawError::Runtime { message, .. } = err {
                            // catch
                            let mut ci = Engine::new(Env::with_parent(&self.env), &self.file);
                            ci.env.define(err_name.clone(), Value::String(message));
                            let catch_r = stack.run(ci.eval_statements(stack, handler)).await?;
                            // finally
                            let _ = stack.run(Engine::new(Env::with_parent(&self.env), &self.file)
                                .eval_statements(stack, finally))
                                .await?;
                            Ok(catch_r)
                        } else {
                            Err(err)
                        }
                    }
                }
                // finally after normal
                let _ = stack.run(Engine::new(Env::with_parent(&self.env), &self.file)
                    .eval_statements(stack, finally))
                    .await?;
                Ok(None)
            }

            StatementKind::RecordDecl { .. } => Ok(None),

            StatementKind::Throw(expr) => {
                let v = stack.run(self.eval_expr(stack, expr)).await?;
                Err(PawError::Runtime {
                    file: self.file.clone(),
                    code: "E6001",
                    message: format!("{}", v),
                    line: stmt.line,
                    column: stmt.col,
                    snippet: None,
                    hint: Some("Uncaught exception".into()),
                })
            }
        }
    }

    /// 计算表达式，返回一个可 await 的 Future
    pub async fn eval_expr(&mut self, stack: Stack<'_>, expr: &Expr) -> Result<Value, PawError> {
        match &expr.kind {
            ExprKind::LiteralInt(n) => Ok(Value::Int(*n)),
            ExprKind::LiteralLong(n) => Ok(Value::Long(*n)),
            ExprKind::LiteralFloat(f) => Ok(Value::Float(*f)),
            ExprKind::LiteralDouble(f) => Ok(Value::Double(*f)),
            ExprKind::LiteralString(s) => Ok(Value::String(s.clone())),
            ExprKind::LiteralBool(b) => Ok(Value::Bool(*b)),
            ExprKind::LiteralChar(c) => Ok(Value::Char(*c)),
            ExprKind::LiteralNopaw => Ok(Value::Null()),

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
                // 1. 先求值子表达式
                let v = stack.run(self.eval_expr(stack, inner)).await?;
                use crate::interpreter::value::ValueInner;

                // 2. 匹配操作符，本分支保证每条路径都返回 Result<Value, PawError>
                match op.as_str() {
                    // 负号
                    "-" => {
                        // 解构 Value 到内部 Arc<ValueInner>
                        let inner_arc = match v {
                            Value(inner) => inner,
                        };
                        match &*inner_arc {
                            ValueInner::Int(i) => Ok(Value::Int(-i)),
                            ValueInner::Long(l) => Ok(Value::Long(-l)),
                            ValueInner::Float(f) => Ok(Value::Float(-f)),
                            other => Err(PawError::Runtime {
                                file: self.file.clone(),
                                code: "E3013".into(),
                                message: format!("Bad unary `{}` on {:?}", op, other),
                                line: expr.line,
                                column: expr.col,
                                snippet: None,
                                hint: None,
                            }),
                        }
                    }

                    // 逻辑非
                    "!" => {
                        let inner_arc = match v {
                            Value(inner) => inner,
                        };
                        match &*inner_arc {
                            ValueInner::Bool(b) => Ok(Value::Bool(!b)),
                            other => Err(PawError::Runtime {
                                file: self.file.clone(),
                                code: "E3013".into(),
                                message: format!("Bad unary `{}` on {:?}", op, other),
                                line: expr.line,
                                column: expr.col,
                                snippet: None,
                                hint: None,
                            }),
                        }
                    }

                    // 其他未知一元操作符
                    _ => Err(PawError::Internal {
                        file: self.file.clone(),
                        code: "E6002".into(),
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
                let l = stack.run(self.eval_expr(stack, left)).await?;
                let r = stack.run(self.eval_expr(stack, right)).await?;
                use crate::ast::expr::BinaryOp::*;
                use crate::interpreter::value::ValueInner::*;

                if let &As = op {
                    return Ok(r.clone());
                }

                if let &EqEq = op {
                    return Ok(Value::Bool(l == r));
                }
                if let &NotEq = op {
                    return Ok(Value::Bool(l != r));
                }

                let result = match (op, &*l.0, &*r.0) {
                    // —— 字符串拼接 ——
                    (Add, String(a), String(b)) => {
                        Value::String(a.as_str().to_string() + b.as_str())
                    }
                    (Add, String(a), other) => {
                        Value::String(a.as_str().to_string() + &format!("{}", other))
                    }
                    (Add, other, String(b)) => Value::String(format!("{}", other) + b.as_str()),

                    // —— 同类型基本情形 ——
                    (Add, Int(a), Int(b)) => Value::Int(a + b),
                    (Add, Long(a), Long(b)) => Value::Long(a + b),
                    (Add, Float(a), Float(b)) => Value::Float(a + b),
                    (Add, Double(a), Double(b)) => Value::Double(a + b),

                    (Sub, Int(a), Int(b)) => Value::Int(a - b),
                    (Sub, Long(a), Long(b)) => Value::Long(a - b),
                    (Sub, Float(a), Float(b)) => Value::Float(a - b),
                    (Sub, Double(a), Double(b)) => Value::Double(a - b),

                    (Mul, Int(a), Int(b)) => Value::Int(a * b),
                    (Mul, Long(a), Long(b)) => Value::Long(a * b),
                    (Mul, Float(a), Float(b)) => Value::Float(a * b),
                    (Mul, Double(a), Double(b)) => Value::Double(a * b),

                    (Div, Int(a), Int(b)) => Value::Int(a / b),
                    (Div, Long(a), Long(b)) => Value::Long(a / b),
                    (Div, Float(a), Float(b)) => Value::Float(a / b),
                    (Div, Double(a), Double(b)) => Value::Double(a / b),

                    (Mod, Int(a), Int(b)) => Value::Int(a % b),
                    (Mod, Long(a), Long(b)) => Value::Long(a % b),

                    // —— 混合 Int ↔ Float/Double ——
                    (Add, Int(a), Float(b)) => Value::Float((*a) as f32 + b),
                    (Add, Float(a), Int(b)) => Value::Float(a + (*b) as f32),
                    (Add, Int(a), Double(b)) => Value::Double((*a) as f64 + b),
                    (Add, Double(a), Int(b)) => Value::Double(a + (*b) as f64),
                    (Add, Long(a), Float(b)) => Value::Float((*a) as f32 + b),
                    (Add, Float(a), Long(b)) => Value::Float(a + (*b) as f32),
                    (Add, Long(a), Double(b)) => Value::Double((*a) as f64 + b),
                    (Add, Double(a), Long(b)) => Value::Double(a + (*b) as f64),

                    (Sub, Int(a), Float(b)) => Value::Float((*a) as f32 - b),
                    (Sub, Float(a), Int(b)) => Value::Float(a - (*b) as f32),
                    (Sub, Int(a), Double(b)) => Value::Double((*a) as f64 - b),
                    (Sub, Double(a), Int(b)) => Value::Double(a - (*b) as f64),
                    (Sub, Long(a), Float(b)) => Value::Float((*a) as f32 - b),
                    (Sub, Float(a), Long(b)) => Value::Float(a - (*b) as f32),
                    (Sub, Long(a), Double(b)) => Value::Double((*a) as f64 - b),
                    (Sub, Double(a), Long(b)) => Value::Double(a - (*b) as f64),

                    (Mul, Int(a), Float(b)) => Value::Float((*a) as f32 * b),
                    (Mul, Float(a), Int(b)) => Value::Float(a * (*b) as f32),
                    (Mul, Int(a), Double(b)) => Value::Double((*a) as f64 * b),
                    (Mul, Double(a), Int(b)) => Value::Double(a * (*b) as f64),
                    (Mul, Long(a), Float(b)) => Value::Float((*a) as f32 * b),
                    (Mul, Float(a), Long(b)) => Value::Float(a * (*b) as f32),
                    (Mul, Long(a), Double(b)) => Value::Double((*a) as f64 * b),
                    (Mul, Double(a), Long(b)) => Value::Double(a * (*b) as f64),

                    (Div, Int(a), Float(b)) => Value::Float((*a) as f32 / b),
                    (Div, Float(a), Int(b)) => Value::Float(a / (*b) as f32),
                    (Div, Int(a), Double(b)) => Value::Double((*a) as f64 / b),
                    (Div, Double(a), Int(b)) => Value::Double(a / (*b) as f64),
                    (Div, Long(a), Float(b)) => Value::Float((*a) as f32 / b),
                    (Div, Float(a), Long(b)) => Value::Float(a / (*b) as f32),
                    (Div, Long(a), Double(b)) => Value::Double((*a) as f64 / b),
                    (Div, Double(a), Long(b)) => Value::Double(a / (*b) as f64),

                    (Lt, Int(a), Int(b)) => Value::Bool(a < b),
                    (Lt, Long(a), Long(b)) => Value::Bool(a < b),
                    (Lt, Float(a), Float(b)) => Value::Bool(a < b),
                    (Lt, Double(a), Double(b)) => Value::Bool(a < b),

                    (Le, Int(a), Int(b)) => Value::Bool(a <= b),
                    (Le, Long(a), Long(b)) => Value::Bool(a <= b),
                    (Le, Float(a), Float(b)) => Value::Bool(a <= b),
                    (Le, Double(a), Double(b)) => Value::Bool(a <= b),

                    (Gt, Int(a), Int(b)) => Value::Bool(a > b),
                    (Gt, Long(a), Long(b)) => Value::Bool(a > b),
                    (Gt, Float(a), Float(b)) => Value::Bool(a > b),
                    (Gt, Double(a), Double(b)) => Value::Bool(a > b),

                    (Ge, Int(a), Int(b)) => Value::Bool(a >= b),
                    (Ge, Long(a), Long(b)) => Value::Bool(a >= b),
                    (Ge, Float(a), Float(b)) => Value::Bool(a >= b),
                    (Ge, Double(a), Double(b)) => Value::Bool(a >= b),

                    (And, Bool(a), Bool(b)) => Value::Bool(*a && *b),
                    (Or, Bool(a), Bool(b)) => Value::Bool(*a || *b),

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
                // 1. 先求值所有参数
                let mut arg_vals = Vec::with_capacity(args.len());
                for e in args {
                    arg_vals.push(stack.run(self.eval_expr(stack, e)).await?);
                }

                // 2. 查找函数
                let func_val = self.env.get(name).ok_or_else(|| PawError::UndefinedVariable {
                    file: self.file.clone(),
                    code: "E4001",
                    name: name.clone(),
                    line: expr.line,
                    column: expr.col,
                    snippet: None,
                    hint: Some("Did you declare this function before use?".into()),
                })?;

                // 3. 解出内部 ValueInner
                use crate::interpreter::value::{Value, ValueInner};
                let inner_arc = match func_val {
                    Value(inner) => inner,         // 解出 Arc<ValueInner>
                };

                // 4. 匹配 Function 分支
                match &*inner_arc {
                    ValueInner::Function {
                        params,
                        body,
                        env: fenv,
                        is_async,
                        ..
                    } => {
                        if *is_async {
                            // —— 异步调用 ——
                            let mut new_interp = Engine::new(Env::with_parent(fenv), &self.file);
                            for (p, v) in params.iter().zip(arg_vals) {
                                new_interp.env.define(p.name.clone(), v);
                            }
                            if let Some(ret) = stack.run(new_interp.eval_statements(stack, body)).await? {
                                Ok(ret)
                            } else {
                                Ok(Value::Null())
                            }
                        } else {
                            // —— 同步调用 ——
                            let saved = self.env.clone();
                            let mut child = Engine::new(Env::with_parent(fenv), &self.file);
                            for (p, v) in params.iter().zip(arg_vals) {
                                child.env.define(p.name.clone(), v);
                            }
                            let res = stack.run(child.eval_statements(stack, body)).await?;
                            self.env = saved;
                            Ok(res.unwrap_or(Value::Null()))
                        }
                    }

                    // —— 不是函数，直接报错 —— 
                    _ => Err(PawError::Runtime {
                        file: self.file.clone(),
                        code: "E4002".into(),
                        message: format!("{} is not callable", name),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: None,
                    }),
                }
            }


            ExprKind::Cast {
                expr: inner,
                ty: _ty,
            } => {
                let v = stack.run(self.eval_expr(stack, inner)).await?;
                Ok(v)
            }

            ExprKind::ArrayLiteral(elems) => {
                let mut items = Vec::with_capacity(elems.len());
                for e in elems {
                    items.push(stack.run(self.eval_expr(stack, e)).await?);
                }
                Ok(Value::Array(items))
            }

            ExprKind::Index { array, index } => {
                // 1. 先 Eval 两个子表达式
                let arr_val = stack.run(self.eval_expr(stack, array)).await?;
                let idx_val = stack.run(self.eval_expr(stack, index)).await?;

                use crate::interpreter::value::ValueInner;

                // 2. 解出内部枚举，然后匹配 Array 和 Int
                let result = match (&*arr_val.0, &*idx_val.0) {
                    // 如果左侧是 Array，右侧是 Int，就取元素
                    (ValueInner::Array(v_arc), ValueInner::Int(i)) => {
                        // v_arc: &Arc<Vec<Value>>
                        let vec = &**v_arc;
                        vec.get(*i as usize)
                            .cloned()
                            .unwrap_or(Value::Null())
                    }
                    // 其余情况，都抛运行时错误
                    _ => {
                        return Err(PawError::Runtime {
                            file: self.file.clone(),
                            code: "E3012".into(),
                            message: "Cannot index into non-array or non-int index".into(),
                            line: expr.line,
                            column: expr.col,
                            snippet: None,
                            hint: None,
                        });
                    }
                };

                Ok(result)
            }

            ExprKind::RecordInit { name: _, fields } => {
                let mut map = AHashMap::new();
                for (fname, fexpr) in fields {
                    let v = stack.run(self.eval_expr(stack, fexpr)).await?;
                    map.insert(fname.clone(), v);
                }
                Ok(Value::Record(map))
            }

            ExprKind::Await { expr: inner } => {
                // 1. 先 eval 出一个 Value
                let val = stack.run(self.eval_expr(stack, inner)).await?;

                // 2. 解出内部的 ValueInner
                use crate::interpreter::value::ValueInner;
                // Value 是 tuple struct(Value(pub Arc<ValueInner>))
                if let Value(inner_arc) = val.clone() {
                    // inner_arc: Arc<ValueInner>
                    if let ValueInner::Future(fut_arc) = &*inner_arc {
                        // 3. 用 futures::lock::Mutex 异步锁
                        let mut guard = fut_arc.lock().await;
                        // 4. await 内部的 Future
                        let result = guard.as_mut().await?;
                        return Ok(result);
                    }
                }

                // 5. 如果不是 Future，原样返回
                Ok(val)
            }

            ExprKind::FieldAccess { expr: inner, field } => {
                // 1. 先 eval 出一个 Value
                let obj_val = stack.run(self.eval_expr(stack, inner)).await?;

                // 2. 解出内部的 ValueInner
                use crate::interpreter::value::ValueInner;
                if let ValueInner::Record(map_arc) = &*obj_val.0 {
                    // map_arc: &Arc<AHashMap<String, Value>>
                    let map: &AHashMap<String, Value> = &**map_arc;

                    // 3. 在 Record map 中查字段
                    if let Some(v) = map.get(field) {
                        Ok(v.clone())
                    } else {
                        // Record 中无此字段
                        Err(PawError::Runtime {
                            file: self.file.clone(),
                            code: "E3015".into(),
                            message: format!("Record has no field '{}'", field),
                            line: expr.line,
                            column: expr.col,
                            snippet: None,
                            hint: None,
                        })
                    }
                } else {
                    // 非 Record 类型，报错
                    Err(PawError::Runtime {
                        file: self.file.clone(),
                        code: "E6003".into(),
                        message: format!("Cannot access field '{}' on {:?}", field, obj_val),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: Some(format!("Type {:?} has no fields", obj_val)),
                    })
                }
            }

            ExprKind::MethodCall {
                receiver,
                method,
                args,
            } => {
                // 1. Evaluate the receiver expression
                let recv = stack.run(self.eval_expr(stack, receiver)).await?;
                // 2. Evaluate all argument expressions
                let mut arg_vals = Vec::with_capacity(args.len());
                for a in args {
                    arg_vals.push(stack.run(self.eval_expr(stack, a)).await?);
                }
                // 3. Dispatch based on the receiver’s runtime type
                match recv {
                    Value(inner_arc) => match &*inner_arc {
                        ValueInner::String(s) => {
                            // ————— String methods —————
                            match method {
                                Method::Trim if arg_vals.is_empty() => {
                                    Ok(Value::String(s.as_str().trim().to_string()))
                                }
                                Method::ToUppercase if arg_vals.is_empty() => {
                                    Ok(Value::String(s.as_str().to_uppercase()))
                                }
                                Method::ToLowercase if arg_vals.is_empty() => {
                                    // 先把 &Arc<String> 解成 &str，然后 to_lowercase 得到 String
                                    let lower: String = s.as_str().to_lowercase();
                                    Ok(lower.into())
                                }
                                Method::Length if arg_vals.is_empty() => {
                                    Ok(Value::Int(s.as_str().chars().count() as i32))
                                }
                                Method::StartsWith if arg_vals.len() == 1 => {
                                    if let Some(p) = arg_vals[0].as_str() {
                                        Ok(Value::Bool(s.as_str().starts_with(p)))
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
                                Method::EndsWith if arg_vals.len() == 1 => {
                                    if let Some(p) = arg_vals[0].as_str() {
                                        Ok(Value::Bool(s.as_str().ends_with(p)))
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
                                Method::Contains if arg_vals.len() == 1 => {
                                    if let Some(p) = arg_vals[0].as_str() {
                                        Ok(Value::Bool(s.as_str().contains(p)))
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
                            }
                        }

                        // ————— Array methods —————
                        ValueInner::Array(v_arc) => {
                            let mut v = (**v_arc).clone();

                            match method {
                                Method::Push if matches!(&arg_vals[..], [_x]) => {
                                    v.push(arg_vals[0].clone());
                                    Ok(Value::Array(v))
                                }
                                Method::Pop if arg_vals.is_empty() => {
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
                                                "Ensure array is non-empty before calling pop"
                                                    .into(),
                                            ),
                                        })
                                    }
                                }
                                Method::Length if arg_vals.is_empty() => {
                                    Ok(Value::Int(v.len() as i32))
                                }
                                _ => Err(PawError::Runtime {
                                    file: self.file.clone(),
                                    code: "E6003".into(),
                                    message: format!("Cannot call method '{}' on Array", method),
                                    line: expr.line,
                                    column: expr.col,
                                    snippet: None,
                                    hint: Some(
                                        "Type Array has no such method or wrong args".into(),
                                    ),
                                }),
                            }
                        }

                        // ————— Module: property lookup or immediate call —————
                        ValueInner::Module(module_map_arc) => {
                            let module_map = &**module_map_arc;
                            let key = method.as_str();

                            if let Some(member_val) = module_map.get(key) {
                                if let ValueInner::Function {
                                    params,
                                    body,
                                    env: fenv,
                                    is_async,
                                    name: _,
                                } = &*member_val.0
                                {
                                    let params = (**params).clone();
                                    let body = (**body).clone();
                                    let fenv = fenv.clone();
                                    let is_async = *is_async;

                                    // Async function call
                                    if is_async {
                                        let mut new_i =
                                            Engine::new(Env::with_parent(&fenv), &self.file);
                                        for (p, v) in params.iter().zip(arg_vals.into_iter()) {
                                            new_i.env.define(p.name.clone(), v);
                                        }
                                        if let Some(ret) = stack.run(new_i.eval_statements(stack, &body)).await? {
                                            Ok(ret)
                                        } else {
                                            Ok(Value::Null())
                                        }
                                    }
                                    // Sync function call
                                    else {
                                        let saved = self.env.clone();
                                        let mut child =
                                            Engine::new(Env::with_parent(&fenv), &self.file);
                                        for (p, v) in params.iter().zip(arg_vals.into_iter()) {
                                            child.env.define(p.name.clone(), v);
                                        }
                                        let res = stack.run(child.eval_statements(stack, &body)).await?;
                                        self.env = saved;
                                        Ok(res.unwrap_or(Value::Null()))
                                    }
                                }
                                // Non‐function: only zero‐arg property access
                                else if arg_vals.is_empty() {
                                    Ok(member_val.clone())
                                } else {
                                    Err(PawError::Runtime {
                                        file: self.file.clone(),
                                        code: "E6003".into(),
                                        message: format!("Cannot call method '{}' on Module", key),
                                        line: expr.line,
                                        column: expr.col,
                                        snippet: None,
                                        hint: Some(format!("Type Module has no method '{}'", key)),
                                    })
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
                    },
                }
            }
        }
    }
}
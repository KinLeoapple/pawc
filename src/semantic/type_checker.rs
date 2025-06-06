use crate::ast::expr::{Expr, ExprKind};
use crate::ast::param::Param;
use crate::ast::statement::{Statement, StatementKind};
use crate::error::error::PawError;
use crate::semantic::scope::{PawType, Scope};
use std::collections::HashSet;

/// 静态类型检查器
pub struct TypeChecker {
    pub scope: Scope,
    pub throwing_functions: HashSet<String>,
    current_fn: Option<String>,
    current_file: String,
}

impl TypeChecker {
    pub fn new(filename: &str) -> Self {
        Self {
            scope: Scope::new(),
            throwing_functions: HashSet::new(),
            current_fn: None,
            current_file: filename.into(),
        }
    }

    pub fn with_parent(parent: &Scope, filename: &str) -> Self {
        Self {
            scope: Scope::with_parent(parent),
            throwing_functions: HashSet::new(),
            current_fn: None,
            current_file: filename.into(),
        }
    }

    /// 顶级入口：预注册函数签名并检查所有语句
    pub fn check_program(&mut self, stmts: &[Statement]) -> Result<(), PawError> {
        // 1. 预注册函数名和签名
        for stmt in stmts {
            if let StatementKind::FunDecl {
                name,
                return_type,
                params: _params,
                ..
            } = &stmt.kind
            {
                let ret_ty = return_type
                    .as_deref()
                    .map(PawType::from_str)
                    .unwrap_or(PawType::Void);
                self.scope
                    .define(name, ret_ty, stmt.line, stmt.col, &self.current_file)
                    .map_err(|_| PawError::DuplicateDefinition {
                        file: self.current_file.clone(),
                        code: "E2005",
                        name: name.clone(),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: Some("Function already defined".into()),
                    })?;
            }
        }
        // 2. 检查每条语句
        for stmt in stmts {
            self.check_statement(stmt)?;
        }
        Ok(())
    }

    pub fn check_statement(&mut self, stmt: &Statement) -> Result<(), PawError> {
        match &stmt.kind {
            StatementKind::Let {
                name,
                ty: declared_str,
                value,
            } => {
                // 1. 推断出值的类型
                let mut inferred = self.check_expr(value)?;

                // 2. 把声明的字符串转成 PawType，Unknown 的情况下尝试从 scope 拿用户定义的
                let mut declared_ty = match PawType::from_str(declared_str) {
                    PawType::Unknown => self.scope.lookup(declared_str).unwrap_or(PawType::Unknown),
                    other => other,
                };

                // 3. 如果是 nopaw 字面量，就直接当作 declared_ty
                if let ExprKind::LiteralNopaw = &value.kind {
                    inferred = declared_ty.clone();
                }

                // 4. 检查兼容性：Exact，T→T?，以及任意数值类型互转
                let ok = if inferred == declared_ty {
                    true
                } else if let PawType::Optional(inner) = &declared_ty {
                    // T → Optional<T>
                    &inferred == inner.as_ref()
                } else if inferred.is_numeric() && declared_ty.is_numeric() {
                    // 不同数值类型之间也允许
                    true
                } else {
                    false
                };

                if !ok {
                    return Err(PawError::Type {
                        file: self.current_file.clone(),
                        code: "E3003",
                        message: format!(
                            "Type mismatch in let '{}': expected {}, found {}",
                            name, declared_ty, inferred
                        ),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: Some("Ensure assigned value matches declared type".into()),
                    });
                }

                // 5. 把真正的 PawType 存到 scope
                self.scope
                    .define(&*name, declared_ty, stmt.line, stmt.col, &self.current_file)?;
            }

            StatementKind::Assign { name, value } => {
                // 1. 拿到变量声明时的类型
                let declared_ty = self.scope.lookup(name).unwrap_or(PawType::Any);
                // 2. 推断出待赋值表达式的类型
                let mut inferred = self.check_expr(value)?;
                // 3. 如果赋值的是 nopaw 字面量，且声明类型本身是 Optional<T>，则直接当成 declared_ty
                if let ExprKind::LiteralNopaw = &value.kind {
                    if let PawType::Optional(_) = &declared_ty {
                        inferred = declared_ty.clone();
                    }
                }
                // 4. 检查兼容性：
                //    - 精确相等
                //    - T -> Optional<T>
                //    - 不同数值类型之间互相赋值也允许
                let ok = if inferred == declared_ty {
                    true
                } else if let PawType::Optional(inner) = &declared_ty {
                    &inferred == inner.as_ref()
                } else if inferred.is_numeric() && declared_ty.is_numeric() {
                    true
                } else {
                    false
                };
                if !ok {
                    return Err(PawError::Type {
                        file: self.current_file.clone(),
                        code: "E3003",
                        message: format!(
                            "Type mismatch in assign '{}': expected {}, found {}",
                            name, declared_ty, inferred
                        ),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: Some("Ensure assigned value matches declared type".into()),
                    });
                }
            }

            StatementKind::FunDecl {
                name,
                params,
                return_type,
                body,
                is_async: _is_async,
            } => {
                // 切换到当前函数
                let prev_fn = self.current_fn.clone();
                self.current_fn = Some(name.clone());

                // 在子作用域中检查函数体
                let mut sub = TypeChecker::with_parent(&self.scope, &self.current_file);
                // 参数入作用域
                for Param {
                    name: pn, ty: pty, ..
                } in params
                {
                    let t = PawType::from_str(pty);
                    sub.scope
                        .define(pn, t, stmt.line, stmt.col, &self.current_file)
                        .map_err(|_| PawError::DuplicateDefinition {
                            file: self.current_file.clone(),
                            code: "E2005",
                            name: pn.clone(),
                            line: stmt.line,
                            column: stmt.col,
                            snippet: None,
                            hint: None,
                        })?;
                }
                // 先检查函数体内部所有语句
                sub.check_program(body)?;

                // 如果声明了返回类型，就扫描所有 return 语句，确保类型一致或可提升到 Optional
                if let Some(ret_ty_str) = return_type {
                    let declared = PawType::from_str(ret_ty_str);
                    // 递归扫描函数体里的 return
                    fn scan_returns(
                        stmts: &[Statement],
                        declared: &PawType,
                        checker: &mut TypeChecker,
                        file: &str,
                    ) -> Result<(), PawError> {
                        for stmt in stmts {
                            match &stmt.kind {
                                StatementKind::Return(opt_expr) => {
                                    let actual = if let Some(expr) = opt_expr {
                                        checker.check_expr(expr)?
                                    } else {
                                        PawType::Void
                                    };
                                    let ok = &actual == declared
                                        || matches!(declared, PawType::Optional(inner) if &actual == inner.as_ref());
                                    if !ok {
                                        return Err(PawError::Type {
                                            file: file.to_string(),
                                            code: "E3004",
                                            message: format!(
                                                "Return type mismatch in function '{}': declared {}, found {}",
                                                checker.current_fn.as_deref().unwrap_or("<anon>"),
                                                declared,
                                                actual
                                            ),
                                            line: stmt.line,
                                            column: stmt.col,
                                            snippet: None,
                                            hint: Some("Ensure return matches declared return type".into()),
                                        });
                                    }
                                }
                                StatementKind::Block(inner) => {
                                    scan_returns(inner, declared, checker, file)?
                                }
                                StatementKind::If {
                                    body, else_branch, ..
                                } => {
                                    scan_returns(body, declared, checker, file)?;
                                    if let Some(else_stmt) = else_branch {
                                        scan_returns(
                                            &[(*else_stmt.clone())],
                                            declared,
                                            checker,
                                            file,
                                        )?;
                                    }
                                }
                                StatementKind::LoopForever(body)
                                | StatementKind::LoopWhile { body, .. } => {
                                    scan_returns(body, declared, checker, file)?
                                }
                                StatementKind::LoopRange { body, .. } => {
                                    scan_returns(body, declared, checker, file)?
                                }
                                StatementKind::TryCatchFinally {
                                    body,
                                    handler,
                                    finally,
                                    ..
                                } => {
                                    scan_returns(body, declared, checker, file)?;
                                    scan_returns(handler, declared, checker, file)?;
                                    scan_returns(finally, declared, checker, file)?;
                                }
                                _ => {}
                            }
                        }
                        Ok(())
                    }
                    // 执行扫描
                    scan_returns(body, &declared, &mut sub, &self.current_file)?;
                }

                // 将子检查器收集到的 throwing_functions 合并回来
                self.throwing_functions.extend(sub.throwing_functions);
                self.current_fn = prev_fn;
            }

            StatementKind::If {
                condition,
                body,
                else_branch,
            } => {
                let cond_ty = self.check_expr(condition)?;
                if cond_ty != PawType::Bool {
                    return Err(PawError::Type {
                        file: self.current_file.clone(),
                        code: "E3006",
                        message: "If condition must be Bool".into(),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: None,
                    });
                }
                let mut child = TypeChecker::with_parent(&self.scope, &self.current_file);
                child.check_program(body)?;
                if let Some(else_stmt) = else_branch {
                    child.check_statement(else_stmt)?;
                }
            }

            StatementKind::LoopForever(body) => {
                let mut child = TypeChecker::with_parent(&self.scope, &self.current_file);
                child.check_program(body)?;
            }

            StatementKind::LoopWhile { condition, body } => {
                let c = self.check_expr(condition)?;
                if c != PawType::Bool {
                    return Err(PawError::Type {
                        file: self.current_file.clone(),
                        code: "E3007",
                        message: "Loop condition must be Bool".into(),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: None,
                    });
                }
                let mut child = TypeChecker::with_parent(&self.scope, &self.current_file);
                child.check_program(body)?;
            }

            StatementKind::LoopRange {
                var,
                start,
                end,
                body,
            } => {
                let s = self.check_expr(start)?;
                let e = self.check_expr(end)?;
                if s != e {
                    return Err(PawError::Type {
                        file: self.current_file.clone(),
                        code: "E3008",
                        message: format!("Range bounds mismatch: {} vs {}", s, e),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: None,
                    });
                }
                let mut child = TypeChecker::with_parent(&self.scope, &self.current_file);
                child
                    .scope
                    .define(var, s.clone(), stmt.line, stmt.col, &self.current_file)?;
                child.check_program(body)?;
            }

            StatementKind::Return(opt) => {
                if let Some(e) = opt {
                    let _ = self.check_expr(e)?;
                }
            }

            StatementKind::LoopArray { var, array, body } => {
                // 1. 推断出 array 表达式的类型
                let arr_ty = self.check_expr(array)?;
                // 2. 必须是 Array<T>，取出 inner
                let elem_ty = match arr_ty {
                    PawType::Array(inner) => *inner,
                    other => {
                        return Err(PawError::Type {
                            file: self.current_file.clone(),
                            code: "E3018", // 新增一个错误码，比如 E3018
                            message: format!("Expected Array<T> in loop, found {}", other),
                            line: stmt.line,
                            column: stmt.col,
                            snippet: None,
                            hint: Some("Loop over an Array<T> only".into()),
                        });
                    }
                };
                // 3. 在子作用域中把循环变量绑定为 elem_ty
                let mut child = TypeChecker::with_parent(&self.scope, &self.current_file);
                child.scope.define(
                    var,
                    elem_ty.clone(),
                    stmt.line,
                    stmt.col,
                    &self.current_file,
                )?;
                // 4. 检查循环体
                child.check_program(body)?;
            }

            StatementKind::Throw(expr) => {
                let ty = self.check_expr(expr)?;
                if ty != PawType::String {
                    return Err(PawError::Type {
                        file: self.current_file.clone(),
                        code: "E3001",
                        message: format!("Cannot bark non-string: {}", ty),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: Some("Only String may be thrown".into()),
                    });
                }
                if let Some(fn_name) = &self.current_fn {
                    self.throwing_functions.insert(fn_name.clone());
                }
            }

            StatementKind::Ask { name, ty, .. } => {
                let expected = PawType::from_str(ty);
                self.scope
                    .define(name, expected, stmt.line, stmt.col, &*self.current_file)
                    .map_err(|_| PawError::DuplicateDefinition {
                        file: self.current_file.clone(),
                        code: "E2005",
                        name: name.clone(),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: None,
                    })?;
            }

            StatementKind::Import { module: _, alias } => {
                // 模块别名注册成 Module
                self.scope
                    .define(
                        &*alias,
                        PawType::Module,
                        stmt.line,
                        stmt.col,
                        &self.current_file,
                    )
                    .map_err(|_| PawError::DuplicateDefinition {
                        file: self.current_file.clone(),
                        code: "E2005",
                        name: alias.clone(),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: Some("Module already imported".into()),
                    })?;
                return Ok(());
            }

            StatementKind::Say(_)
            | StatementKind::AskPrompt(_)
            | StatementKind::Block(_)
            | StatementKind::Continue
            | StatementKind::Break
            | StatementKind::Expr(_) => {
                // 这几种语句无需额外检查或已经在 check_expr 中处理
                if let StatementKind::Expr(e) = &stmt.kind {
                    let _ = self.check_expr(e)?;
                }
            }
            StatementKind::RecordDecl { name, fields, .. } => {
                // 把字段列表转换成 Vec<(String, PawType)>
                let field_types: Vec<(String, PawType)> = fields
                    .iter()
                    .map(|p| (p.name.clone(), PawType::from_str(&p.ty)))
                    .collect();
                self.scope
                    .define(
                        name,
                        PawType::Record(field_types),
                        stmt.line,
                        stmt.col,
                        &self.current_file,
                    )
                    .map_err(|_| PawError::DuplicateDefinition {
                        file: self.current_file.clone(),
                        code: "E2005",
                        name: name.clone(),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: Some("Record already defined".into()),
                    })?;
            }
            StatementKind::TryCatchFinally {
                body,
                err_name,
                handler,
                finally,
            } => {
                // 先忽略 try 里抛出的错误，正常检查主体
                let _ = TypeChecker::with_parent(&self.scope, &self.current_file)
                    .check_program(body)?; // 或者你的批量检查方法名

                // Catch 分支：在子作用域里把 err_name 定义成 String，然后检查 handler
                let mut catch_checker = TypeChecker::with_parent(&self.scope, &self.current_file);
                catch_checker
                    .scope
                    .define(
                        err_name,
                        PawType::String,
                        stmt.line,
                        stmt.col,
                        &self.current_file,
                    )
                    .map_err(|_| PawError::DuplicateDefinition {
                        file: self.current_file.clone(),
                        code: "E2005",
                        name: err_name.clone(),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: None,
                    })?;
                catch_checker.check_program(handler)?;

                // Finally 分支也要在新作用域检查
                TypeChecker::with_parent(&self.scope, &self.current_file).check_program(finally)?;
            }
        }
        Ok(())
    }

    pub fn check_expr(&mut self, expr: &Expr) -> Result<PawType, PawError> {
        match &expr.kind {
            ExprKind::LiteralInt(_) => Ok(PawType::Int),
            ExprKind::LiteralLong(_) => Ok(PawType::Long),
            ExprKind::LiteralFloat(_) => Ok(PawType::Float),
            ExprKind::LiteralDouble(_) => Ok(PawType::Double),
            ExprKind::LiteralString(_) => Ok(PawType::String),
            ExprKind::LiteralBool(_) => Ok(PawType::Bool),
            ExprKind::LiteralChar(_) => Ok(PawType::Char),
            ExprKind::LiteralNopaw => Ok(PawType::Optional(Box::new(PawType::Any))),

            ExprKind::Var(n) => self
                .scope
                .lookup(n)
                .ok_or_else(|| PawError::UndefinedVariable {
                    file: self.current_file.clone(),
                    code: "E4001",
                    name: n.clone(),
                    line: expr.line,
                    column: expr.col,
                    snippet: None,
                    hint: Some("Did you declare this variable before use?".into()),
                }),

            ExprKind::UnaryOp { op, expr: inner } => {
                let t = self.check_expr(inner)?;
                match op.as_str() {
                    "-" if t.is_numeric() => Ok(t),
                    "!" if t == PawType::Bool => Ok(PawType::Bool),
                    _ => Err(PawError::Type {
                        file: self.current_file.clone(),
                        code: "E3013",
                        message: format!("Bad unary '{}' on {}", op, t),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: None,
                    }),
                }
            }

            ExprKind::BinaryOp { op, left, right } => {
                let l = self.check_expr(left)?;
                let r = self.check_expr(right)?;
                l.binary_result(op, &r).map_err(|msg| PawError::Type {
                    file: self.current_file.clone(),
                    code: "E3014",
                    message: msg,
                    line: expr.line,
                    column: expr.col,
                    snippet: None,
                    hint: None,
                })
            }

            ExprKind::Call { name, args } => {
                for a in args {
                    let _ = self.check_expr(a)?;
                }
                // 模块调用一律 Any
                if name.contains('.') {
                    Ok(PawType::Any)
                } else {
                    self.scope
                        .lookup(name)
                        .ok_or_else(|| PawError::UndefinedVariable {
                            file: self.current_file.clone(),
                            code: "E4001",
                            name: name.clone(),
                            line: expr.line,
                            column: expr.col,
                            snippet: None,
                            hint: None,
                        })
                }
            }

            ExprKind::Cast { expr: inner, ty } => {
                let from = self.check_expr(inner)?;
                let to = PawType::from_str(ty);
                if to == PawType::Any || from == to || (from.is_numeric() && to.is_numeric()) {
                    Ok(to)
                } else {
                    Err(PawError::Type {
                        file: self.current_file.clone(),
                        code: "E3009",
                        message: format!("Cannot cast {} to {}", from, to),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: None,
                    })
                }
            }

            ExprKind::ArrayLiteral(elems) => {
                // 1. 初始类型设为 Any
                let mut elem_ty = PawType::Any;
                // 2. 记录是否出现过 nopaw
                let mut saw_nopaw = false;

                for e in elems {
                    // 遇到 nopaw 只标记，不做类型合并
                    if let ExprKind::LiteralNopaw = &e.kind {
                        saw_nopaw = true;
                        continue;
                    }
                    // 否则正常推断这个元素的类型
                    let t = self.check_expr(e)?;

                    if elem_ty == PawType::Any {
                        // 第一个真值元素决定类型
                        elem_ty = t;
                    } else if elem_ty == t {
                        // 同类型，OK
                    } else if let PawType::Optional(inner) = &elem_ty {
                        // elem_ty 是 Optional(X)，只接受 X
                        if &t == inner.as_ref() {
                            // OK，保持 Optional(X)
                        } else {
                            return Err(PawError::Type {
                                file: self.current_file.clone(),
                                code: "E3010",
                                message: format!("Array elements mismatch: {} vs {}", elem_ty, t),
                                line: expr.line,
                                column: expr.col,
                                snippet: None,
                                hint: None,
                            });
                        }
                    } else if let PawType::Optional(inner2) = t.clone() {
                        // t 是 Optional(X)，且 elem_ty == X，就把 elem_ty 提升为 Optional(X)
                        if elem_ty == *inner2 {
                            elem_ty = PawType::Optional(Box::new(elem_ty));
                        } else {
                            return Err(PawError::Type {
                                file: self.current_file.clone(),
                                code: "E3010",
                                message: format!("Array elements mismatch: {} vs {}", elem_ty, t),
                                line: expr.line,
                                column: expr.col,
                                snippet: None,
                                hint: None,
                            });
                        }
                    } else {
                        // 其它任意组合都报错
                        return Err(PawError::Type {
                            file: self.current_file.clone(),
                            code: "E3010",
                            message: format!("Array elements mismatch: {} vs {}", elem_ty, t),
                            line: expr.line,
                            column: expr.col,
                            snippet: None,
                            hint: None,
                        });
                    }
                }

                // 如果见过 nopaw，就把最终类型标记为可空
                let final_ty = if saw_nopaw {
                    PawType::Optional(Box::new(elem_ty))
                } else {
                    elem_ty
                };

                Ok(PawType::Array(Box::new(final_ty)))
            }

            ExprKind::Index { array, index } => {
                let at = self.check_expr(array)?;
                let it = self.check_expr(index)?;
                if it != PawType::Int {
                    return Err(PawError::Type {
                        file: self.current_file.clone(),
                        code: "E3011",
                        message: format!("Index must be Int, found {}", it),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: None,
                    });
                }
                if let PawType::Array(inner) = at {
                    Ok(*inner)
                } else {
                    Err(PawError::Type {
                        file: self.current_file.clone(),
                        code: "E3012",
                        message: format!("Cannot index into {}", at),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: None,
                    })
                }
            }

            ExprKind::FieldAccess { expr: inner, field } => {
                let ot = self.check_expr(inner)?;
                if let PawType::Record(fields) = ot {
                    fields
                        .into_iter()
                        .find(|(n, _)| n == field)
                        .map(|(_, t)| t)
                        .ok_or_else(|| PawError::Type {
                            file: self.current_file.clone(),
                            code: "E3015",
                            message: format!("Record has no field {}", field),
                            line: expr.line,
                            column: expr.col,
                            snippet: None,
                            hint: None,
                        })
                } else {
                    Err(PawError::Type {
                        file: self.current_file.clone(),
                        code: "E3016",
                        message: format!("{} is not a record", ot),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: None,
                    })
                }
            }

            ExprKind::MethodCall {
                receiver,
                method,
                args,
            } => {
                // 1. 推断出接收者的类型
                let recv_t = self.check_expr(receiver)?;
                // 2. 推断出所有参数类型
                let mut arg_types = Vec::with_capacity(args.len());
                for arg in args {
                    arg_types.push(self.check_expr(arg)?);
                }

                // —— String 方法 ——
                if recv_t == PawType::String {
                    match method.as_str() {
                        "trim" | "to_uppercase" | "to_lowercase" => {
                            // 无参数
                            if !arg_types.is_empty() {
                                return Err(PawError::Type {
                                    file: self.current_file.clone(),
                                    code: "E3023",
                                    message: format!(
                                        "Method '{}' on String takes no arguments, found {}",
                                        method,
                                        arg_types.len()
                                    ),
                                    line: expr.line,
                                    column: expr.col,
                                    snippet: None,
                                    hint: None,
                                });
                            }
                            Ok(PawType::String)
                        }
                        "length" => {
                            if !arg_types.is_empty() {
                                return Err(PawError::Type {
                                    file: self.current_file.clone(),
                                    code: "E3023",
                                    message: format!(
                                        "Method 'length' on String takes no arguments, found {}",
                                        arg_types.len()
                                    ),
                                    line: expr.line,
                                    column: expr.col,
                                    snippet: None,
                                    hint: None,
                                });
                            }
                            Ok(PawType::Int)
                        }
                        "starts_with" | "ends_with" | "contains" => {
                            // 这些方法需要且仅需要一个 String 参数
                            if arg_types.len() != 1 {
                                return Err(PawError::Type {
                                    file: self.current_file.clone(),
                                    code: "E3024",
                                    message: format!(
                                        "Method '{}' on String requires 1 argument, found {}",
                                        method,
                                        arg_types.len()
                                    ),
                                    line: expr.line,
                                    column: expr.col,
                                    snippet: None,
                                    hint: None,
                                });
                            }
                            if arg_types[0] != PawType::String {
                                return Err(PawError::Type {
                                    file: self.current_file.clone(),
                                    code: "E3025",
                                    message: format!(
                                        "Method '{}' on String requires String argument, found {}",
                                        method, arg_types[0]
                                    ),
                                    line: expr.line,
                                    column: expr.col,
                                    snippet: None,
                                    hint: None,
                                });
                            }
                            Ok(PawType::Bool)
                        }
                        _ => Err(PawError::Type {
                            file: self.current_file.clone(),
                            code: "E3021",
                            message: format!("Type String has no method '{}'", method),
                            line: expr.line,
                            column: expr.col,
                            snippet: None,
                            hint: None,
                        }),
                    }
                }
                // —— Array 方法 ——
                else if let PawType::Array(inner) = recv_t.clone() {
                    match method.as_str() {
                        "push" => {
                            // push 需要且仅需要一个参数，类型要与 inner 匹配
                            if arg_types.len() != 1 {
                                return Err(PawError::Type {
                                    file: self.current_file.clone(),
                                    code: "E3024",
                                    message: format!(
                                        "Method 'push' on Array requires 1 argument, found {}",
                                        arg_types.len()
                                    ),
                                    line: expr.line,
                                    column: expr.col,
                                    snippet: None,
                                    hint: None,
                                });
                            }
                            if arg_types[0] != *inner {
                                return Err(PawError::Type {
                                    file: self.current_file.clone(),
                                    code: "E3022",
                                    message: format!(
                                        "push 参数类型不匹配：expected {}, found {}",
                                        inner, arg_types[0]
                                    ),
                                    line: expr.line,
                                    column: expr.col,
                                    snippet: None,
                                    hint: None,
                                });
                            }
                            Ok(PawType::Void)
                        }
                        "pop" => {
                            if !arg_types.is_empty() {
                                return Err(PawError::Type {
                                    file: self.current_file.clone(),
                                    code: "E3023",
                                    message: format!(
                                        "Method 'pop' on Array takes no arguments, found {}",
                                        arg_types.len()
                                    ),
                                    line: expr.line,
                                    column: expr.col,
                                    snippet: None,
                                    hint: None,
                                });
                            }
                            Ok(*inner)
                        }
                        "length" => {
                            if !arg_types.is_empty() {
                                return Err(PawError::Type {
                                    file: self.current_file.clone(),
                                    code: "E3023",
                                    message: format!(
                                        "Method 'length' on Array takes no arguments, found {}",
                                        arg_types.len()
                                    ),
                                    line: expr.line,
                                    column: expr.col,
                                    snippet: None,
                                    hint: None,
                                });
                            }
                            Ok(PawType::Int)
                        }
                        _ => {
                            return Err(PawError::Type {
                                file: self.current_file.clone(),
                                code: "E3021",
                                message: format!(
                                    "Type {} has no method '{}'",
                                    PawType::Array(inner),
                                    method
                                ),
                                line: expr.line,
                                column: expr.col,
                                snippet: None,
                                hint: None,
                            });
                        }
                    }
                }
                // —— Module 方法 ——
                else if recv_t == PawType::Module {
                    // import 进来的模块对任意方法调用均返回 Any
                    Ok(PawType::Any)
                }
                // —— 其它类型不支持 MethodCall ——
                else {
                    Err(PawError::Type {
                        file: self.current_file.clone(),
                        code: "E3021",
                        message: format!("Type {} has no method '{}'", recv_t, method),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: None,
                    })
                }
            }

            ExprKind::RecordInit { name, fields } => {
                // 1. 拿 record 定义
                let rec_ty = self
                    .scope
                    .lookup(name)
                    .ok_or_else(|| PawError::UndefinedVariable {
                        file: self.current_file.clone(),
                        code: "E4001",
                        name: name.clone(),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: Some("Did you declare this record before use?".into()),
                    })?
                    .clone();
                // 2. 必须是 Record(...) 类型
                let defs = if let PawType::Record(defs) = rec_ty.clone() {
                    defs
                } else {
                    return Err(PawError::Type {
                        file: self.current_file.clone(),
                        code: "E3016",
                        message: format!("{} is not a record type", rec_ty),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: None,
                    });
                };
                // 3. 逐字段检查
                for (fname, fexpr) in fields {
                    // 找到期望类型
                    let expected = defs
                        .iter()
                        .find(|(n, _)| n == fname)
                        .map(|(_, t)| t.clone())
                        .ok_or_else(|| PawError::Type {
                            file: self.current_file.clone(),
                            code: "E3015",
                            message: format!("Record `{}` has no field `{}`", name, fname),
                            line: expr.line,
                            column: expr.col,
                            snippet: None,
                            hint: None,
                        })?;
                    // nopaw 视为 expected；否则递归检查
                    let actual = if let ExprKind::LiteralNopaw = &fexpr.kind {
                        expected.clone()
                    } else {
                        self.check_expr(fexpr)?
                    };
                    // 允许 T 和 T? 互赋
                    let ok = if actual == expected {
                        true
                    } else if let PawType::Optional(inner) = &expected {
                        actual == *inner.as_ref()
                    } else {
                        false
                    };
                    if !ok {
                        return Err(PawError::Type {
                            file: self.current_file.clone(),
                            code: "E3017",
                            message: format!(
                                "Field `{}` of record `{}`: expected {}, found {}",
                                fname, name, expected, actual
                            ),
                            line: expr.line,
                            column: expr.col,
                            snippet: None,
                            hint: None,
                        });
                    }
                }
                Ok(rec_ty)
            }

            ExprKind::Await { expr: inner } => self.check_expr(inner),
        }
    }
}

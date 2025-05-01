// src/semantic/type_checker.rs

use crate::ast::{BinaryOp, Expr, Statement, StatementKind};
use crate::semantic::scope::{PawType, Scope};
use std::collections::HashSet;
use crate::error::error::PawError;

fn is_numeric(t: &PawType) -> bool {
    matches!(
        t,
        PawType::Int | PawType::Long | PawType::Float | PawType::Double
    )
}

/// 静态类型检查器
pub struct TypeChecker {
    pub scope: Scope,
    pub throwing_functions: HashSet<String>,
    current_fn: Option<String>,
}

impl TypeChecker {
    /// 创建一个全新顶层作用域
    pub fn new() -> Self {
        TypeChecker {
            scope: Scope::new(),
            throwing_functions: HashSet::new(),
            current_fn: None,
        }
    }

    /// 以已有作用域为父，创建子检查器
    pub fn with_parent(parent: &Scope) -> Self {
        TypeChecker {
            scope: Scope::with_parent(&parent.clone()),
            throwing_functions: HashSet::new(),
            current_fn: None,
        }
    }

    /// 检查多条语句（两阶段）
    pub fn check_statements(&mut self, stmts: &[Statement]) -> Result<(), PawError> {
        // —— 阶段一：预注册所有函数签名 ——
        for stmt in stmts {
            if let StatementKind::FunDecl {
                name, return_type, ..
            } = &stmt.kind
            {
                // 解析返回类型
                let ret_ty = return_type
                    .as_deref()
                    .map(PawType::from_str)
                    .unwrap_or(PawType::Void);
                // 注册到作用域
                self.scope
                    .define(name, ret_ty)
                    .map_err(|_| PawError::DuplicateDefinition { name: name.clone() })?;
            }
        }

        // —— 阶段二：逐条检查 ——
        for stmt in stmts {
            self.check_statement(stmt)?;
        }
        Ok(())
    }

    /// 检查单条语句
    pub fn check_statement(&mut self, stmt: &Statement) -> Result<(), PawError> {
        match &stmt.kind {
            StatementKind::Import { alias, .. } => {
                // 把别名写进当前作用域，类型就是 Module
                self.scope.define(alias, PawType::Module).map_err(|_| {
                    PawError::DuplicateDefinition {
                        name: alias.clone(),
                    }
                })?;
            }
            StatementKind::Throw(expr) => {
                // 验证 throw 后面跟的表达式是可转为 String 的类型
                let ty = self.check_expr(expr)?;
                if ty != PawType::String && ty != PawType::Any {
                    return Err(PawError::Type {
                        message: format!("Cannot bark non-string: {:?}", ty),
                    });
                }
            }
            StatementKind::Let { name, ty, value } => {
                let expected = PawType::from_str(ty);
                let actual = self.check_expr(value)?;
                if expected != actual && expected != PawType::Any {
                    return Err(PawError::Type {
                        message: format!(
                            "Type mismatch in let '{}': expected {}, found {}",
                            name, expected, actual
                        ),
                    });
                }
                self.scope
                    .define(name, expected)
                    .map_err(|_| PawError::DuplicateDefinition { name: name.clone() })?;
            }

            StatementKind::Assign { name, value } => {
                let actual = self.check_expr(value)?;
                let expected = self
                    .scope
                    .lookup(name)
                    .ok_or_else(|| PawError::UndefinedVariable { name: name.clone() })?;
                if expected != actual && expected != PawType::Any {
                    return Err(PawError::Type {
                        message: format!(
                            "Type mismatch in assignment to '{}': expected {}, found {}",
                            name, expected, actual
                        ),
                    });
                }
            }

            StatementKind::Say(expr) => {
                self.check_expr(expr)?;
            }

            StatementKind::Ask { name, ty, .. } => {
                let expected = PawType::from_str(ty);
                self.scope
                    .define(name, expected)
                    .map_err(|_| PawError::DuplicateDefinition { name: name.clone() })?;
            }

            StatementKind::AskPrompt(_) => {}

            StatementKind::Return(opt) => {
                if let Some(expr) = opt {
                    self.check_expr(expr)?;
                }
            }

            StatementKind::Break | StatementKind::Continue => {}

            StatementKind::Expr(expr) => {
                self.check_expr(expr)?;
            }

            StatementKind::If {
                condition,
                body,
                else_branch,
            } => {
                let cond_ty = self.check_expr(condition)?;
                if cond_ty != PawType::Bool {
                    return Err(PawError::Type {
                        message: "Condition of if must be Bool".into(),
                    });
                }
                let mut child = TypeChecker::with_parent(&self.scope);
                child.check_statements(body)?;
                if let Some(else_stmt) = else_branch {
                    child.check_statement(else_stmt)?;
                }
            }

            StatementKind::LoopForever(body) => {
                let mut child = TypeChecker::with_parent(&self.scope);
                child.check_statements(body)?;
            }

            StatementKind::LoopWhile { condition, body } => {
                let cond_ty = self.check_expr(condition)?;
                if cond_ty != PawType::Bool {
                    return Err(PawError::Type {
                        message: "Condition of loop must be Bool".into(),
                    });
                }
                let mut child = TypeChecker::with_parent(&self.scope);
                child.check_statements(body)?;
            }

            StatementKind::LoopRange {
                var,
                start,
                end,
                body,
            } => {
                let start_ty = self.check_expr(start)?;
                let end_ty = self.check_expr(end)?;
                if start_ty != end_ty {
                    return Err(PawError::Type {
                        message: format!(
                            "Range bounds must have same type, got {} vs {}",
                            start_ty, end_ty
                        ),
                    });
                }
                let mut child = TypeChecker::with_parent(&self.scope);
                child
                    .scope
                    .define(var, start_ty.clone())
                    .map_err(|_| PawError::DuplicateDefinition { name: var.clone() })?;
                child.check_statements(body)?;
            }

            StatementKind::FunDecl {
                name,
                params,
                return_type: _,
                body,
            } => {
                // 进入新函数
                let prev = self.current_fn.clone();
                self.current_fn = Some(name.clone());
                // 函数本身已经在第一阶段预注册了签名，这里只检查函数体
                let mut fun_scope = TypeChecker::with_parent(&self.scope);
                for p in params {
                    let p_ty = PawType::from_str(&p.ty);
                    fun_scope.scope.define(&p.name, p_ty).map_err(|_| {
                        PawError::DuplicateDefinition {
                            name: p.name.clone(),
                        }
                    })?;
                }
                fun_scope.check_statements(body)?;
                // 把子检查器发现的 throwing_functions 合并回来
                if let Some(fns) = &fun_scope.current_fn {
                    if fun_scope.throwing_functions.contains(fns) {
                        self.throwing_functions.insert(fns.clone());
                    }
                }
                self.throwing_functions.extend(fun_scope.throwing_functions);
                self.current_fn = prev;
            }

            StatementKind::Block(stmts) => {
                let mut nested = TypeChecker::with_parent(&self.scope);
                nested.check_statements(stmts)?;
            }

            // sniff { … } snatch (e) { … } [lastly { … }]
            StatementKind::TryCatchFinally {
                body,
                err_name,
                handler,
                finally,
            } => {
                // —— 检 try(sniff) 块 ——
                let mut try_sc = TypeChecker::with_parent(&self.scope);
                try_sc.check_statements(body)?;

                // —— 检 catch(snatch) 块 ——
                let mut catch_sc = TypeChecker::with_parent(&self.scope);
                catch_sc
                    .scope
                    .define(err_name, PawType::String)
                    .map_err(|_| PawError::DuplicateDefinition {
                        name: err_name.clone(),
                    })?;
                catch_sc.check_statements(handler)?;

                // —— 检 finally(lastly) 块 ——
                let mut fin_sc = TypeChecker::with_parent(&self.scope);
                fin_sc.check_statements(finally)?;
            }
        }
        Ok(())
    }

    /// 计算表达式的静态类型
    pub fn check_expr(&mut self, expr: &Expr) -> Result<PawType, PawError> {
        if let Expr::Cast { expr: inner, ty } = expr {
            // 检查子表达式是否本身类型合法
            let _inner_ty = self.check_expr(inner)?;
            // 目标类型
            let target = PawType::from_str(ty);
            // 允许数值间一切转换，或 same→same，或 any→任何
            return if target == PawType::Any
                || _inner_ty == target
                || (is_numeric(&_inner_ty) && is_numeric(&target))
            {
                Ok(target)
            } else {
                Err(PawError::Type {
                    message: format!("Cannot cast {} to {}", _inner_ty, target),
                })
            };
        }

        match expr {
            Expr::LiteralInt(_) => Ok(PawType::Int),
            Expr::LiteralLong(_) => Ok(PawType::Long),
            Expr::LiteralFloat(_) => Ok(PawType::Float),
            Expr::LiteralString(_) => Ok(PawType::String),
            Expr::LiteralBool(_) => Ok(PawType::Bool),
            Expr::LiteralChar(_) => Ok(PawType::Char),

            Expr::ArrayLiteral(elems) => {
                // 如果空数组，元素类型先标记为 Any
                let elem_ty = if let Some(first) = elems.first() {
                    let ty0 = self.check_expr(first)?;
                    // 确保其它元素类型一致
                    for e in elems.iter().skip(1) {
                        let ty1 = self.check_expr(e)?;
                        if ty0 != ty1 {
                            return Err(PawError::Type {
                                message: format!(
                                    "Array literal element types mismatch: {} vs {}",
                                    ty0, ty1
                                ),
                            });
                        }
                    }
                    ty0
                } else {
                    PawType::Any
                };
                Ok(PawType::Array(Box::new(elem_ty)))
            }
            Expr::Index { array, index } => {
                let arr_ty = self.check_expr(array)?;
                let idx_ty = self.check_expr(index)?;
                if idx_ty != PawType::Int {
                    return Err(PawError::Type {
                        message: format!("Index must be Int, found {}", idx_ty),
                    });
                }
                match arr_ty {
                    PawType::Array(inner) => Ok(*inner),
                    other => Err(PawError::Type {
                        message: format!("Cannot index into non-array type {}", other),
                    }),
                }
            }

            Expr::UnaryOp { op, expr } => {
                let ty = self.check_expr(expr)?;
                match op.as_str() {
                    "-" => match ty {
                        PawType::Int | PawType::Long | PawType::Float => Ok(ty),
                        _ => Err(PawError::Type {
                            message: format!("Unary '-' not supported for {}", ty),
                        }),
                    },
                    "!" => Ok(PawType::Bool),
                    _ => Err(PawError::Type {
                        message: format!("Unknown unary operator '{}'", op),
                    }),
                }
            }

            Expr::Property { object, name } => {
                let obj_ty = self.check_expr(object)?;
                match obj_ty {
                    PawType::Module => Ok(PawType::Any),
                    PawType::Array(_) if name == "length" => Ok(PawType::Int),
                    PawType::String if name == "length" => Ok(PawType::Int),
                    _ => Err(PawError::Type {
                        message: format!("Cannot access property {:?} on {:?}", obj_ty, name),
                    }),
                }
            }

            Expr::BinaryOp { left, op, right } => {
                let l_ty = self.check_expr(left)?;
                let r_ty = self.check_expr(right)?;

                // 字符串拼接：String + X 或 X + String => String
                if *op == BinaryOp::Add && (l_ty == PawType::String || r_ty == PawType::String) {
                    return Ok(PawType::String);
                }

                // 如果两侧都是数值类型但不相同，则做“提升”：
                let numeric = |t: &PawType| {
                    matches!(
                        t,
                        PawType::Int | PawType::Long | PawType::Float | PawType::Double
                    )
                };
                let (l_ty, r_ty) = if numeric(&l_ty) && numeric(&r_ty) {
                    // 只要有一方是 Double，就提升到 Double；否则有一方是 Float，就提升到 Float；……
                    let promoted = match (&l_ty, &r_ty) {
                        (PawType::Double, _) | (_, PawType::Double) => PawType::Double,
                        (PawType::Float, _) | (_, PawType::Float) => PawType::Float,
                        (PawType::Long, _) | (_, PawType::Long) => PawType::Long,
                        _ => PawType::Int,
                    };
                    (promoted.clone(), promoted)
                } else if l_ty != r_ty {
                    return Err(PawError::Type {
                        message: format!("Type mismatch in binary op: {} vs {}", l_ty, r_ty),
                    });
                } else {
                    (l_ty.clone(), r_ty.clone())
                };

                let result_ty = match op {
                    BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div
                    | BinaryOp::Mod => l_ty,

                    BinaryOp::EqEq
                    | BinaryOp::NotEq
                    | BinaryOp::Lt
                    | BinaryOp::Le
                    | BinaryOp::Gt
                    | BinaryOp::Ge
                    | BinaryOp::And
                    | BinaryOp::Or => PawType::Bool,

                    _ => PawType::Any,
                };
                Ok(result_ty)
            }

            Expr::Var(name) => self
                .scope
                .lookup(name)
                .ok_or_else(|| PawError::UndefinedVariable { name: name.clone() }),

            Expr::Property { object, name: _ } => {
                let obj_ty = self.check_expr(object)?;
                match obj_ty {
                    PawType::String => Ok(PawType::Any),
                    PawType::Array(_) => Ok(PawType::Any), // length 之类
                    PawType::Module => Ok(PawType::Any),   // m.square、m.PI …
                    _ => Err(PawError::Type {
                        message: format!("Type {:?} has no properties", obj_ty),
                    }),
                }
            }

            Expr::Call { name, args } => {
                // 函数调用：先检查参数，再返回预注册的签名
                for arg in args {
                    let _ = self.check_expr(arg)?;
                }
                if let Some((mod_name, member)) = name.split_once('.') {
                    if self.scope.lookup(mod_name) == Some(PawType::Module) {
                        return Ok(PawType::Any);
                    }
                }
                // 否则按原逻辑报未定义
                self.scope
                    .lookup(name)
                    .ok_or_else(|| PawError::UndefinedVariable { name: name.clone() })
            }

            _ => Ok(PawType::Any),
        }
    }
}

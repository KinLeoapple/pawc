// src/semantic/type_checker.rs

use crate::ast::{BinaryOp, Expr, Statement, StatementKind};
use crate::error::PawError;
use crate::semantic::scope::{PawType, Scope};

/// 静态类型检查器
pub struct TypeChecker {
    pub scope: Scope,
}

impl TypeChecker {
    /// 创建一个全新顶层作用域
    pub fn new() -> Self {
        TypeChecker {
            scope: Scope::new(),
        }
    }

    /// 以已有作用域为父，创建子检查器
    pub fn with_parent(parent: &Scope) -> Self {
        TypeChecker {
            scope: Scope::with_parent(&parent.clone()),
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
                name: _,
                params,
                return_type: _,
                body,
            } => {
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
            }

            StatementKind::Block(stmts) => {
                let mut nested = TypeChecker::with_parent(&self.scope);
                nested.check_statements(stmts)?;
            }
        }
        Ok(())
    }

    /// 计算表达式的静态类型
    pub fn check_expr(&mut self, expr: &Expr) -> Result<PawType, PawError> {
        match expr {
            Expr::LiteralInt(_) => Ok(PawType::Int),
            Expr::LiteralLong(_) => Ok(PawType::Long),
            Expr::LiteralFloat(_) => Ok(PawType::Float),
            Expr::LiteralString(_) => Ok(PawType::String),
            Expr::LiteralBool(_) => Ok(PawType::Bool),
            Expr::LiteralChar(_) => Ok(PawType::Char),

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

            Expr::BinaryOp { left, op, right } => {
                let l_ty = self.check_expr(left)?;
                let r_ty = self.check_expr(right)?;

                if *op == BinaryOp::Add && (l_ty == PawType::String || r_ty == PawType::String) {
                    return Ok(PawType::String);
                }

                if l_ty != r_ty {
                    return Err(PawError::Type {
                        message: format!("Type mismatch in binary op: {} vs {}", l_ty, r_ty),
                    });
                }
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

            Expr::Call { name, args } => {
                // 先检查所有参数
                for arg in args {
                    let _ = self.check_expr(arg)?;
                }
                // 再返回函数签名里定义的返回类型
                self.scope
                    .lookup(name)
                    .ok_or_else(|| PawError::UndefinedVariable { name: name.clone() })
            }

            _ => Ok(PawType::Any),
        }
    }
}

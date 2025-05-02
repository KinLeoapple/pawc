// File: src/semantic/type_checker.rs

use crate::ast::expr::ExprKind;
use crate::ast::{BinaryOp, Expr, Statement, StatementKind};
use crate::error::error::PawError;
use crate::semantic::scope::{PawType, Scope};
use std::collections::HashSet;

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
    current_filename: String,
}

impl TypeChecker {
    pub fn new(filename: &str) -> Self {
        TypeChecker {
            scope: Scope::new(),
            throwing_functions: HashSet::new(),
            current_fn: None,
            current_filename: filename.to_string(),
        }
    }

    pub fn with_parent(parent: &Scope, filename: &str) -> Self {
        TypeChecker {
            scope: Scope::with_parent(&parent.clone()),
            throwing_functions: HashSet::new(),
            current_fn: None,
            current_filename: filename.to_string(),
        }
    }

    pub fn check_statements(&mut self, stmts: &[Statement]) -> Result<(), PawError> {
        // 阶段一：预注册函数签名
        for stmt in stmts {
            if let StatementKind::FunDecl {
                name, return_type, ..
            } = &stmt.kind
            {
                let ret_ty = return_type
                    .as_deref()
                    .map(PawType::from_str)
                    .unwrap_or(PawType::Void);
                self.scope
                    .define(name, ret_ty, stmt.line, stmt.col, &*self.current_filename)
                    .map_err(|_| PawError::DuplicateDefinition {
                        file: self.current_filename.clone(),
                        code: "E2005",
                        name: name.clone(),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: Some("Function already defined".into()),
                    })?;
            }
        }
        // 阶段二：检查
        for stmt in stmts {
            self.check_statement(stmt)?;
        }
        Ok(())
    }

    pub fn check_statement(&mut self, stmt: &Statement) -> Result<(), PawError> {
        match &stmt.kind {
            StatementKind::Import { alias, .. } => {
                self.scope
                    .define(
                        alias,
                        PawType::Module,
                        stmt.line,
                        stmt.col,
                        &*self.current_filename,
                    )
                    .map_err(|_| PawError::DuplicateDefinition {
                        file: self.current_filename.clone(),
                        code: "E2005",
                        name: alias.clone(),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: Some("Alias already defined".into()),
                    })?;
            }

            StatementKind::Throw(expr) => {
                let ty = self.check_expr(expr)?;
                if ty != PawType::String && ty != PawType::Any {
                    return Err(PawError::Type {
                        file: self.current_filename.clone(),
                        code: "E3001",
                        message: format!("Cannot bark non-string: {:?}", ty),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: Some("Only String or Any may be thrown".into()),
                    });
                }
            }

            StatementKind::Let { name, ty, value } => {
                let expected = PawType::from_str(ty);

                // 如果是 nopaw literal
                if let Expr {
                    kind: ExprKind::LiteralNopaw,
                    line,
                    col,
                } = *value
                {
                    // 若 expected 是 optional，则允许
                    if matches!(expected, PawType::Optional(_)) {
                        self.scope
                            .define(name, expected, line, col, &*self.current_filename)
                            .map_err(|_| PawError::DuplicateDefinition {
                                file: self.current_filename.clone(),
                                code: "E2005",
                                name: name.clone(),
                                line,
                                column: col,
                                snippet: None,
                                hint: Some("Variable already defined".into()),
                            })?;
                        return Ok(());
                    } else {
                        return Err(PawError::Type {
                            file: self.current_filename.clone(),
                            code: "E3002",
                            message: format!(
                                "Cannot assign `nopaw` to {} in let '{}'",
                                expected, name
                            ),
                            line,
                            column: col,
                            snippet: None,
                            hint: Some("Use an Optional type to accept nopaw".into()),
                        });
                    }
                }

                let actual = self.check_expr(value)?;
                // 扩展协变：除了 Optional(Any) → Optional(T)，还允许 T → Optional<T>
                let covariant =
                                // 1) Optional(Any) 可以赋给任何 Optional<T>
                                matches!((&expected, &actual),
                        (PawType::Optional(_), PawType::Optional(inner)) if **inner == PawType::Any
                    )
                                // 2) 普通 T 也能赋给 Optional<T>
                                || matches!((&expected, &actual),
                        (PawType::Optional(inner), actual_ty) if **inner == *actual_ty
                    );
                if expected != actual && expected != PawType::Any && !covariant {
                    return Err(PawError::Type {
                        file: self.current_filename.clone(),
                        code: "E3003",
                        message: format!(
                            "Type mismatch in let '{}': expected {}, found {}",
                            name, expected, actual
                        ),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: Some("Ensure assigned value matches declared type".into()),
                    });
                }

                self.scope
                    .define(name, expected, stmt.line, stmt.col, &*self.current_filename)
                    .map_err(|_| PawError::DuplicateDefinition {
                        file: self.current_filename.clone(),
                        code: "E2005",
                        name: name.clone(),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: Some("Variable already defined".into()),
                    })?;
            }

            StatementKind::Assign { name, value } => {
                let expected =
                    self.scope
                        .lookup(name)
                        .ok_or_else(|| PawError::UndefinedVariable {
                            file: self.current_filename.clone(),
                            code: "E4001",
                            name: name.clone(),
                            line: stmt.line,
                            column: stmt.col,
                            snippet: None,
                            hint: Some("Did you declare this variable before use?".into()),
                        })?;

                if let Expr {
                    kind: ExprKind::LiteralNopaw,
                    line,
                    col,
                } = *value
                {
                    if !matches!(expected, PawType::Optional(_)) {
                        return Err(PawError::Type {
                            file: self.current_filename.clone(),
                            code: "E3004",
                            message: format!("Cannot assign `nopaw` to {}", expected),
                            line,
                            column: col,
                            snippet: None,
                            hint: Some("Only Optional types accept nopaw".into()),
                        });
                    }
                }

                let actual = self.check_expr(value)?;
                // 扩展协变：Optional(Any) → Optional(T) 或者 T → Optional<T>
                let covariant = matches!((&expected, &actual),
        (PawType::Optional(_), PawType::Optional(inner)) if **inner == PawType::Any
    )
                // 2) 普通 T 也能赋给 Optional<T>
                || matches!((&expected, &actual),
        (PawType::Optional(inner), actual_ty) if **inner == *actual_ty
    );
                if expected != actual && expected != PawType::Any && !covariant {
                    return Err(PawError::Type {
                        file: self.current_filename.clone(),
                        code: "E3005",
                        message: format!(
                            "Type mismatch in assignment to '{}': expected {}, found {}",
                            name, expected, actual
                        ),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: Some("Ensure assigned value matches variable type".into()),
                    });
                }
            }

            StatementKind::Say(expr) => {
                self.check_expr(expr)?;
            }

            StatementKind::Ask { name, ty, .. } => {
                let expected = PawType::from_str(ty);
                self.scope
                    .define(name, expected, stmt.line, stmt.col, &*self.current_filename)
                    .map_err(|_| PawError::DuplicateDefinition {
                        file: self.current_filename.clone(),
                        code: "E2005",
                        name: name.clone(),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: None,
                    })?;
            }

            StatementKind::AskPrompt(_) => {}

            StatementKind::Return(opt) => {
                if let Some(e) = opt {
                    self.check_expr(e)?;
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
                        file: self.current_filename.clone(),
                        code: "E3006",
                        message: "If condition must be Bool".into(),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: None,
                    });
                }
                let mut child = TypeChecker::with_parent(&self.scope, &*self.current_filename);
                child.check_statements(body)?;
                if let Some(else_stmt) = else_branch {
                    child.check_statement(else_stmt)?;
                }
            }

            StatementKind::LoopForever(body) => {
                TypeChecker::with_parent(&self.scope, &*self.current_filename)
                    .check_statements(body)?;
            }

            StatementKind::LoopWhile { condition, body } => {
                let cond_ty = self.check_expr(condition)?;
                if cond_ty != PawType::Bool {
                    return Err(PawError::Type {
                        file: self.current_filename.clone(),
                        code: "E3007",
                        message: "Loop condition must be Bool".into(),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: None,
                    });
                }
                TypeChecker::with_parent(&self.scope, &*self.current_filename)
                    .check_statements(body)?;
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
                        file: self.current_filename.clone(),
                        code: "E3008",
                        message: format!("Range bounds mismatch: {} vs {}", s, e),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: None,
                    });
                }
                let mut child = TypeChecker::with_parent(&self.scope, &*self.current_filename);
                child
                    .scope
                    .define(var, s.clone(), stmt.line, stmt.col, &*self.current_filename)
                    .map_err(|_| PawError::DuplicateDefinition {
                        file: self.current_filename.clone(),
                        code: "E2005",
                        name: var.clone(),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: None,
                    })?;
                child.check_statements(body)?;
            }

            StatementKind::FunDecl {
                name, params, body, ..
            } => {
                let prev = self.current_fn.clone();
                self.current_fn = Some(name.clone());
                let mut sub = TypeChecker::with_parent(&self.scope, &*self.current_filename);
                for p in params {
                    let pty = PawType::from_str(&p.ty);
                    sub.scope
                        .define(&p.name, pty, stmt.line, stmt.col, &*self.current_filename)
                        .map_err(|_| PawError::DuplicateDefinition {
                            file: self.current_filename.clone(),
                            code: "E2005",
                            name: p.name.clone(),
                            line: stmt.line,
                            column: stmt.col,
                            snippet: None,
                            hint: None,
                        })?;
                }
                sub.check_statements(body)?;
                if let Some(fn_name) = &sub.current_fn {
                    if sub.throwing_functions.contains(fn_name) {
                        self.throwing_functions.insert(fn_name.clone());
                    }
                }
                self.throwing_functions.extend(sub.throwing_functions);
                self.current_fn = prev;
            }

            StatementKind::Block(stmts) => {
                TypeChecker::with_parent(&self.scope, &*self.current_filename)
                    .check_statements(stmts)?;
            }

            StatementKind::TryCatchFinally {
                body,
                err_name,
                handler,
                finally,
            } => {
                let _ = TypeChecker::with_parent(&self.scope, &*self.current_filename)
                    .check_statements(body);
                let mut csc = TypeChecker::with_parent(&self.scope, &*self.current_filename);
                csc.scope
                    .define(
                        err_name,
                        PawType::String,
                        stmt.line,
                        stmt.col,
                        &*self.current_filename,
                    )
                    .map_err(|_| PawError::DuplicateDefinition {
                        file: self.current_filename.clone(),
                        code: "E2005",
                        name: err_name.clone(),
                        line: stmt.line,
                        column: stmt.col,
                        snippet: None,
                        hint: None,
                    })?;
                csc.check_statements(handler)?;
                TypeChecker::with_parent(&self.scope, &*self.current_filename)
                    .check_statements(finally)?;
            }
        }
        Ok(())
    }

    pub fn check_expr(&mut self, expr: &Expr) -> Result<PawType, PawError> {
        match &expr.kind {
            ExprKind::LiteralInt(_) => Ok(PawType::Int),
            ExprKind::LiteralLong(_) => Ok(PawType::Long),
            ExprKind::LiteralFloat(_) => Ok(PawType::Float),
            ExprKind::LiteralString(_) => Ok(PawType::String),
            ExprKind::LiteralBool(_) => Ok(PawType::Bool),
            ExprKind::LiteralChar(_) => Ok(PawType::Char),
            ExprKind::LiteralNopaw => Ok(PawType::Optional(Box::new(PawType::Any))),

            ExprKind::Cast { expr: inner, ty } => {
                let ity = self.check_expr(inner)?;
                let tgt = PawType::from_str(ty);
                if tgt == PawType::Any || ity == tgt || (is_numeric(&ity) && is_numeric(&tgt)) {
                    Ok(tgt)
                } else {
                    Err(PawError::Type {
                        file: self.current_filename.clone(),
                        code: "E3009",
                        message: format!("Cannot cast {} to {}", ity, tgt),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: None,
                    })
                }
            }

            ExprKind::ArrayLiteral(elems) => {
                if let Some(first) = elems.first() {
                    let t0 = self.check_expr(first)?;
                    for e in &elems[1..] {
                        let t1 = self.check_expr(e)?;
                        if t0 != t1 {
                            return Err(PawError::Type {
                                file: self.current_filename.clone(),
                                code: "E3010",
                                message: format!("Array elements mismatch: {} vs {}", t0, t1),
                                line: expr.line,
                                column: expr.col,
                                snippet: None,
                                hint: None,
                            });
                        }
                    }
                }
                let et = if let Some(first) = elems.first() {
                    self.check_expr(first)?
                } else {
                    PawType::Any
                };
                Ok(PawType::Array(Box::new(et)))
            }

            ExprKind::Index { array, index } => {
                let at = self.check_expr(array)?;
                let it = self.check_expr(index)?;
                if it != PawType::Int {
                    return Err(PawError::Type {
                        file: self.current_filename.clone(),
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
                        file: self.current_filename.clone(),
                        code: "E3012",
                        message: format!("Cannot index into {}", at),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: None,
                    })
                }
            }

            ExprKind::UnaryOp { op, expr: inner } => {
                let t = self.check_expr(inner)?;
                let valid = (op == "-"
                    && matches!(t, PawType::Int | PawType::Long | PawType::Float))
                    || op == "!";
                if valid {
                    Ok(if op == "!" { PawType::Bool } else { t })
                } else {
                    Err(PawError::Type {
                        file: self.current_filename.clone(),
                        code: "E3013",
                        message: format!("Bad unary `{}` on {}", op, t),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: None,
                    })
                }
            }

            ExprKind::BinaryOp { left, op, right } => {
                let l = self.check_expr(left)?;
                let r = self.check_expr(right)?;
                if *op == BinaryOp::Add && (l == PawType::String || r == PawType::String) {
                    return Ok(PawType::String);
                }
                let numeric = |x: &PawType| {
                    matches!(
                        x,
                        PawType::Int | PawType::Long | PawType::Float | PawType::Double
                    )
                };
                let (lt, _rt) = if numeric(&l) && numeric(&r) {
                    let p = if matches!((&l, &r), (PawType::Double, _) | (_, PawType::Double)) {
                        PawType::Double
                    } else if matches!((&l, &r), (PawType::Float, _) | (_, PawType::Float)) {
                        PawType::Float
                    } else if matches!((&l, &r), (PawType::Long, _) | (_, PawType::Long)) {
                        PawType::Long
                    } else {
                        PawType::Int
                    };
                    (p.clone(), p)
                } else if l != r {
                    return Err(PawError::Type {
                        file: self.current_filename.clone(),
                        code: "E3014",
                        message: format!("Type mismatch {} vs {}", l, r),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: None,
                    });
                } else {
                    (l.clone(), r.clone())
                };
                let res = match op {
                    BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div
                    | BinaryOp::Mod => lt,
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
                Ok(res)
            }

            ExprKind::Var(n) => self
                .scope
                .lookup(n)
                .ok_or_else(|| PawError::UndefinedVariable {
                    file: self.current_filename.clone(),
                    code: "E4001",
                    name: n.clone(),
                    line: expr.line,
                    column: expr.col,
                    snippet: None,
                    hint: Some("Did you declare this variable before use?".into()),
                }),

            ExprKind::Property { object, name } => {
                let ot = self.check_expr(object)?;
                match ot {
                    PawType::Array(_) | PawType::String if name == "length" => Ok(PawType::Int),
                    PawType::Module => Ok(PawType::Any),
                    _ => Err(PawError::Type {
                        file: self.current_filename.clone(),
                        code: "E3015",
                        message: format!("{} has no property {}", ot, name),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: None,
                    }),
                }
            }

            ExprKind::Call { name, args } => {
                for a in args {
                    let _ = self.check_expr(a)?;
                }
                if let Some((m, _)) = name.split_once('.') {
                    if self.scope.lookup(m) == Some(PawType::Module) {
                        return Ok(PawType::Any);
                    }
                }
                self.scope
                    .lookup(name)
                    .ok_or_else(|| PawError::UndefinedVariable {
                        file: self.current_filename.clone(),
                        code: "E4001",
                        name: name.clone(),
                        line: expr.line,
                        column: expr.col,
                        snippet: None,
                        hint: Some("Did you declare this function or module before use?".into()),
                    })
            }
        }
    }
}

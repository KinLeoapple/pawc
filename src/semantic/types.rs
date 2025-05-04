// File: src/semantic/types.rs

use std::fmt;
use crate::ast::expr::BinaryOp;
use crate::ast::expr::BinaryOp::{Add, And, Div, EqEq, Ge, Gt, Le, Lt, Mod, Mul, NotEq, Or, Sub};

/// 支持的类型
#[derive(Clone, Debug, PartialEq)]
pub enum PawType {
    Int,
    Long,
    Float,
    Double,
    Bool,
    Char,
    String,
    Void,
    Any,
    /// 可选类型，如 T? 或者 `Optional(T)`
    Optional(Box<PawType>),
    /// 数组类型，如 Array<T>
    Array(Box<PawType>),
    /// 记录类型，字段名和字段类型列表
    Record(Vec<(String, PawType)>),
    /// 模块类型，用于 import
    Module,
    /// 未知类型，用于错误恢复
    Unknown,
}

impl PawType {
    /// 从脚本里的类型名字符串解析出 PawType
    /// 支持 T?, Array<T>, 以及基础类型名称
    pub fn from_str(s: &str) -> Self {
        // 可选类型后缀 '?'
        if let Some(inner) = s.strip_suffix('?') {
            return PawType::Optional(Box::new(PawType::from_str(inner)));
        }
        // 泛型 Array<T>
        if let Some(inner) = s.strip_prefix("Array<").and_then(|rest| rest.strip_suffix('>')) {
            return PawType::Array(Box::new(PawType::from_str(inner)));
        }
        // 基础类型
        match s {
            "Int" => PawType::Int,
            "Long" => PawType::Long,
            "Float" => PawType::Float,
            "Double" => PawType::Double,
            "Bool" => PawType::Bool,
            "Char" => PawType::Char,
            "String" => PawType::String,
            "Void" => PawType::Void,
            "Any" => PawType::Any,
            "Module" => PawType::Module,
            _ => PawType::Unknown,
        }
    }
}

impl fmt::Display for PawType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PawType::Int => write!(f, "Int"),
            PawType::Long => write!(f, "Long"),
            PawType::Float => write!(f, "Float"),
            PawType::Double => write!(f, "Double"),
            PawType::Bool => write!(f, "Bool"),
            PawType::Char => write!(f, "Char"),
            PawType::String => write!(f, "String"),
            PawType::Void => write!(f, "Void"),
            PawType::Any => write!(f, "Any"),
            PawType::Module => write!(f, "Module"),
            PawType::Unknown => write!(f, "Unknown"),
            PawType::Optional(inner) => write!(f, "{}?", inner),
            PawType::Array(elem) => write!(f, "Array<{}>", elem),
            PawType::Record(fields) => {
                // 打印成 {x: Int, y: String}
                let parts: Vec<String> = fields
                    .iter()
                    .map(|(n, t)| format!("{}: {}", n, t))
                    .collect();
                write!(f, "{{{}}}", parts.join(", "))
            }
        }
    }
}

impl PawType {
    pub(crate) fn is_numeric(&self) -> bool {
        matches!(
            self,
            PawType::Int | PawType::Long | PawType::Float | PawType::Double
        )
    }

    pub(crate) fn binary_result(&self, op: &BinaryOp, rhs: &PawType) -> Result<PawType, String> {
        use crate::ast::expr::BinaryOp::*;
        // 字符串 concat
        if *op == Add && (self == &PawType::String || rhs == &PawType::String) {
            return Ok(PawType::String);
        }
        // 数值运算
        if self.is_numeric() && rhs.is_numeric() {
            let out = if matches!((self, rhs), (PawType::Double, _) | (_, PawType::Double)) {
                PawType::Double
            } else if matches!((self, rhs), (PawType::Float, _) | (_, PawType::Float)) {
                PawType::Float
            } else if matches!((self, rhs), (PawType::Long, _) | (_, PawType::Long)) {
                PawType::Long
            } else {
                PawType::Int
            };
            return match op {
                Add | Sub | Mul | Div | Mod => Ok(out),
                EqEq | NotEq | Lt | Le | Gt | Ge => Ok(PawType::Bool),
                _ => Err(format!("Unsupported operator {:?} for numeric types", op)),
            };
        }
        // 逻辑运算
        if *op == And || *op == Or {
            if self == &PawType::Bool && rhs == &PawType::Bool {
                return Ok(PawType::Bool);
            } else {
                return Err("Logical operators require Bool operands".into());
            }
        }
        // 相等比较
        if *op == EqEq || *op == NotEq {
            if self == rhs {
                return Ok(PawType::Bool);
            }
            return Err(format!("Cannot compare {} vs {}", self, rhs));
        }
        Err(format!(
            "Type mismatch {} vs {} for operator {:?}",
            self, rhs, op
        ))
    }
}


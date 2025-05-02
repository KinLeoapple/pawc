// src/semantic/scope.rs

use crate::error::error::PawError;
use std::collections::HashMap;

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
    Optional(Box<PawType>),
    Any,
    Unknown,
    Module,
    Array(Box<PawType>),
}

impl PawType {
    /// 从脚本里的类型名字符串解析出 PawType
    pub fn from_str(s: &str) -> Self {
        if let Some(inner) = s.strip_suffix('?') {
            let inner_ty = PawType::from_str(inner);
            return PawType::Optional(Box::new(inner_ty));
        }

        if let Some(inner) = s.strip_prefix("Array<").and_then(|t| t.strip_suffix('>')) {
            PawType::Array(Box::new(PawType::from_str(inner)))
        } else {
            match s {
                "Int" => PawType::Int,
                "Long" => PawType::Long,
                "Float" => PawType::Float,
                "Double" => PawType::Double,
                "Bool" => PawType::Bool,
                "Char" => PawType::Char,
                "String" => PawType::String,
                "Void" => PawType::Void,
                "Module" => PawType::Module,
                "Any" => PawType::Any,
                _ => PawType::Unknown,
            }
        }
    }
}

impl std::fmt::Display for PawType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PawType::*;
        match self {
            Int => write!(f, "Int"),
            Long => write!(f, "Long"),
            Float => write!(f, "Float"),
            Double => write!(f, "Double"),
            Bool => write!(f, "Bool"),
            Char => write!(f, "Char"),
            String => write!(f, "String"),
            Void => write!(f, "Void"),
            Any => write!(f, "Any"),
            Optional(ty) => write!(f, "{}?", ty),
            Module => write!(f, "Module"),
            Unknown => write!(f, "Unknown"),
            Array(elem) => write!(f, "Array<{}>", elem),
        }
    }
}

/// 作用域，支持嵌套查找
#[derive(Clone, Debug)]
pub struct Scope {
    symbols: HashMap<String, PawType>,
    parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            symbols: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: &Scope) -> Self {
        Scope {
            symbols: HashMap::new(),
            parent: Some(Box::new(parent.clone())),
        }
    }

    /// 定义新变量，已存在则 Err
    ///
    /// Now takes `line` and `column` so that errors can carry a source span.
    pub fn define(
        &mut self,
        name: &str,
        ty: PawType,
        line: usize,
        column: usize,
    ) -> Result<(), PawError> {
        if self.symbols.contains_key(name) {
            Err(PawError::DuplicateDefinition {
                code: "E2005", // duplicate definition
                name: name.to_string(),
                line,
                column,
                snippet: None,
                hint: Some("Try a different name".into()),
            })
        } else {
            self.symbols.insert(name.to_string(), ty);
            Ok(())
        }
    }

    /// 向上查找
    pub fn lookup(&self, name: &str) -> Option<PawType> {
        if let Some(t) = self.symbols.get(name) {
            Some(t.clone())
        } else if let Some(parent) = &self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }
}

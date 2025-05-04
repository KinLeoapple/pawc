// src/semantic/scope.rs

use crate::error::error::PawError;
use std::collections::HashMap;
use std::sync::Arc;
pub(crate) use crate::semantic::types::PawType;

/// 作用域，支持嵌套查找
#[derive(Clone, Debug)]
pub struct Scope {
    symbols: HashMap<String, PawType>,
    parent: Option<Arc<Scope>>,
}

impl Scope {
    /// 创建一个新的空作用域
    pub fn new() -> Self {
        Scope {
            symbols: HashMap::new(),
            parent: None,
        }
    }

    /// 以现有作用域作为父作用域创建子作用域
    pub fn with_parent(parent: &Scope) -> Self {
        Scope {
            symbols: HashMap::new(),
            parent: Some(Arc::new(parent.clone())),
        }
    }

    /// 在当前作用域中定义一个新符号，若已存在则返回 Err
    pub fn define(
        &mut self,
        name: &str,
        ty: PawType,
        line: usize,
        column: usize,
        filename: &str,
    ) -> Result<(), PawError> {
        if self.symbols.contains_key(name) {
            Err(PawError::DuplicateDefinition {
                file: filename.to_string(),
                code: "E2005",
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

    /// 定义一个模块别名
    pub fn define_module(&mut self, alias: &str, line: usize, col: usize, file: &str) -> Result<(), PawError> {
        self.define(alias, PawType::Module, line, col, file)
    }

    /// 向上查找符号类型，若未找到返回 None
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

// src/interpreter/env.rs

use crate::error::error::PawError;
use crate::interpreter::value::{Value, ValueInner};
use ahash::AHashMap;
use parking_lot::RwLock;
use std::sync::Arc;

/// 对外的环境句柄
#[derive(Clone, Debug)]
pub struct Env(Arc<RwLock<AHashMap<String, Value>>>);

impl Env {
    /// 创建一个全新空环境
    pub fn new() -> Self {
        Env(Arc::new(RwLock::new(AHashMap::new())))
    }

    /// 基于父环境创建一个新环境（浅拷贝所有现有绑定）
    pub fn with_parent(parent: &Env) -> Self {
        // parking_lot::RwLock::read() 直接返回 guard，无需 unwrap
        let map = parent.0.read().clone();
        Env(Arc::new(RwLock::new(map)))
    }

    /// 定义或覆盖一个变量
    pub fn define(&self, key: String, val: Value) {
        let mut w = self.0.write();
        w.insert(key, val);
    }

    /// 导出当前所有绑定
    pub fn bindings(&self) -> AHashMap<String, Value> {
        self.0.read().clone()
    }

    /// 更新已存在变量，否则报错
    pub fn assign(&self, key: &str, val: Value) -> Result<(), PawError> {
        let mut w = self.0.write();
        if w.contains_key(key) {
            w.insert(key.to_string(), val);
            Ok(())
        } else {
            Err(PawError::UndefinedVariable {
                file: "<runtime>".into(),
                code: "E4001",
                name: key.into(),
                line: 0,
                column: 0,
                snippet: None,
                hint: Some("Did you declare this variable before use?".into()),
            })
        }
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        self.0.read().get(key).cloned()
    }

    /// 对单个值执行一元运算
    pub fn unary_op(&self, op: &str, v: Value, file: &str) -> Result<Value, PawError> {
        match v {
            Value(inner) => match op {
                // 负号
                "-" => match &*inner {
                    ValueInner::Int(i) => Ok(Value::Int(-i)),
                    ValueInner::Long(l) => Ok(Value::Long(-l)),
                    ValueInner::Float(f) => Ok(Value::Float(-f)),
                    other => Err(PawError::Runtime {
                        file: file.into(),
                        code: "E3013".into(),
                        message: format!("Bad unary `-` on {:?}", other),
                        line: 0,
                        column: 0,
                        snippet: None,
                        hint: None,
                    }),
                },
                // 逻辑非
                "!" => match &*inner {
                    ValueInner::Bool(b) => Ok(Value::Bool(!b)),
                    other => Err(PawError::Runtime {
                        file: file.into(),
                        code: "E3013".into(),
                        message: format!("Bad unary `!` on {:?}", other),
                        line: 0,
                        column: 0,
                        snippet: None,
                        hint: None,
                    }),
                },
                _ => Err(PawError::Internal {
                    file: file.into(),
                    code: "E6002".into(),
                    message: format!("Unknown unary operator `{}`", op),
                    line: 0,
                    column: 0,
                    snippet: None,
                    hint: None,
                }),
            },
        }
    }
}

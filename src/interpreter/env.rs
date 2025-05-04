// src/interpreter/env.rs

use crate::error::error::PawError;
use crate::interpreter::value::Value;
use ahash::AHashMap;
use std::sync::RwLock;
use std::sync::Arc;

/// 实际存数据的结构
#[derive(Debug)]
struct EnvInner {
    values: AHashMap<String, Value>,
    parent: Option<Env>,
}

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
        let map = parent.0.read().unwrap().clone();
        Env(Arc::new(RwLock::new(map)))
    }

    /// 定义或覆盖一个变量
    pub fn define(&self, key: String, val: Value) {
        let mut w = self.0.write().unwrap();
        w.insert(key, val);
    }

    /// 导出当前所有绑定
    pub fn bindings(&self) -> AHashMap<String, Value> {
        self.0.read().unwrap().clone()
    }
    
    /// 更新已存在变量，否则报错
    pub fn assign(&self, key: &str, val: Value) -> Result<(), PawError> {
        let mut w = self.0.write().unwrap();
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
        let r = self.0.read().unwrap();
        r.get(key).cloned()
    }
}

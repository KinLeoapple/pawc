// src/interpreter/env.rs

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::error::error::PawError;
use crate::interpreter::value::Value;

/// 实际存数据的结构
#[derive(Debug)]
struct EnvInner {
    values: HashMap<String, Value>,
    parent: Option<Env>,
}

/// 对外的环境句柄
#[derive(Clone, Debug)]
pub struct Env(Arc<Mutex<EnvInner>>);

impl Env {
    pub fn new() -> Self {
        Env(Arc::new(Mutex::new(EnvInner {
            values: HashMap::new(),
            parent: None,
        })))
    }

    pub fn with_parent(parent: &Env) -> Self {
        Env(Arc::new(Mutex::new(EnvInner {
            values: HashMap::new(),
            parent: Some(parent.clone()),
        })))
    }

    pub fn keys(&self) -> Vec<String> {
        let guard = self.0.lock().unwrap();
        guard.values.keys().cloned().collect()
    }

    /// 返回当前环境中所有顶层绑定的拷贝
    pub fn bindings(&self) -> HashMap<String, Value> {
        // 锁住内部，然后克隆出 values
        let guard = self.0.lock().unwrap();
        guard.values.clone()
    }

    /// 返回当前层所有定义的 key→value 副本，用于 Import 之后把子环境导出
    pub fn snapshot(&self) -> HashMap<String, Value> {
        // 复制当前层的 values
        let guard = self.0.lock().unwrap();
        guard.values.clone()
    }

    pub fn define(&self, name: String, val: Value) {
        let mut guard = self.0.lock().unwrap();
        guard.values.insert(name, val);
    }

    pub fn assign(&self, name: &str, val: Value) -> Result<(), PawError> {
        // 先在锁内看看是不是自己定义的；如果不是，就把 parent clone 出来
        let parent_opt = {
            let mut guard = self.0.lock().unwrap();
            if guard.values.contains_key(name) {
                guard.values.insert(name.to_string(), val);
                return Ok(());
            }
            guard.parent.clone()
        }; // guard 在这里就 drop 了

        // 锁已经释放，递归调用父环境
        if let Some(parent) = parent_opt {
            parent.assign(name, val)
        } else {
            Err(PawError::UndefinedVariable {
                file: "<runtime>".into(),
                code: "E4001",
                name: name.into(),
                line: 0,
                column: 0,
                snippet: None,
                hint: Some("Did you declare this variable before use?".into()),
            })
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        // 同样，先在锁内查一次本层，拿到 parent 的 clone
        let (found, parent_opt) = {
            let guard = self.0.lock().unwrap();
            let v = guard.values.get(name).cloned();
            (v, guard.parent.clone())
        }; // guard drop

        if let Some(v) = found {
            Some(v)
        } else if let Some(parent) = parent_opt {
            parent.get(name)
        } else {
            None
        }
    }
}

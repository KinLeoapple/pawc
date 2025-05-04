// src/interpreter/value.rs

use crate::ast::param::Param;
use crate::ast::statement::Statement;
use crate::error::error::PawError;
use crate::interpreter::env::Env;
use std::{f64, fmt};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc};
use futures::lock::Mutex;
use ahash::AHashMap;

#[derive(Debug,Clone)]
pub enum ValueInner {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    Char(char),
    String(Arc<String>),
    Array(Arc<Vec<Value>>),
    Record(Arc<AHashMap<String, Value>>),
    Module(Arc<AHashMap<String, Value>>),
    Function {
        name: Arc<String>,
        params: Arc<Vec<Param>>,
        body: Arc<Vec<Statement>>,
        env: Env,
        is_async: bool,
    },
    Future(Arc<Mutex<Pin<Box<dyn Future<Output=Result<Value, PawError>> + Send>>>>),
    Null,
    Optional(Arc<Option<Value>>),
}

impl fmt::Display for ValueInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueInner::Int(i)       => write!(f, "{}", i),
            ValueInner::Long(l)      => write!(f, "{}", l),
            ValueInner::Float(fl)    => write!(f, "{}", fl),
            ValueInner::Double(d)    => write!(f, "{}", d),
            ValueInner::Bool(b)      => write!(f, "{}", b),
            ValueInner::Char(c)      => write!(f, "{}", c),
            ValueInner::String(s)    => write!(f, "{}", s),
            ValueInner::Null         => write!(f, "Nopaw"),
            ValueInner::Optional(o)  => {
                if let Some(v) = &**o {
                    write!(f, "{}", v)
                } else {
                    write!(f, "Nopaw")
                }
            }
            ValueInner::Array(arr)   => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
            ValueInner::Record(r)    => {
                let fields: Vec<String> =
                    r.iter().map(|(k,v)| format!("{}: {}", k, v)).collect();
                write!(f, "{{{}}}", fields.join(", "))
            }
            ValueInner::Module(_)    => write!(f, "<module>"),
            ValueInner::Function {..}=> write!(f, "<function>"),
            ValueInner::Future {..}  => write!(f, "<future>"),
        }
    }
}

/// 对外暴露的 Value 类型，内部引用计数
#[derive(Clone, Debug)]
pub struct Value(pub Arc<ValueInner>);

impl Value {
    /// 从内部枚举构造
    pub fn from_inner(inner: ValueInner) -> Self {
        Value(Arc::new(inner))
    }

    // 常见类型构造器
    pub fn Int(v: i32) -> Self {
        Value::from_inner(ValueInner::Int(v))
    }
    pub fn Long(v: i64) -> Self {
        Value::from_inner(ValueInner::Long(v))
    }
    pub fn Float(v: f32) -> Self {
        Value::from_inner(ValueInner::Float(v))
    }
    pub fn Double(v: f64) -> Self {
        Value::from_inner(ValueInner::Double(v))
    }
    pub fn Bool(v: bool) -> Self {
        Value::from_inner(ValueInner::Bool(v))
    }
    pub fn Char(v: char) -> Self {
        Value::from_inner(ValueInner::Char(v))
    }
    pub fn String<S: Into<String>>(s: S) -> Self {
        Value::from_inner(ValueInner::String(Arc::new(s.into())))
    }
    pub fn Array(v: Vec<Value>) -> Self {
        Value::from_inner(ValueInner::Array(Arc::new(v)))
    }
    pub fn Record(m: AHashMap<String, Value>) -> Self {
        Value::from_inner(ValueInner::Record(Arc::new(m)))
    }
    pub fn Module(m: AHashMap<String, Value>) -> Self {
        Value::from_inner(ValueInner::Module(Arc::new(m)))
    }
    pub fn Null() -> Self {
        Value::from_inner(ValueInner::Null)
    }
    pub fn Optional(o: Option<Value>) -> Self {
        Value::from_inner(ValueInner::Optional(Arc::new(o)))
    }

    /// Function 构造
    pub fn Function(
        name: String,
        params: Vec<Param>,
        body: Vec<Statement>,
        env: Env,
        is_async: bool,
    ) -> Self {
        Value::from_inner(ValueInner::Function {
            name: Arc::new(name),
            params: Arc::new(params),
            body: Arc::new(body),
            env,
            is_async,
        })
    }

    //// Future 构造
    pub fn Future(
        fut: Pin<Box<dyn Future<Output = Result<Value, PawError>> + Send>>
    ) -> Self {
        Value::from_inner(ValueInner::Future(Arc::new(Mutex::new(fut))))
    }
    
}

impl Value {
    /// 如果自己是字符串，就返回 &str，否则返回 None
    pub fn as_str(&self) -> Option<&str> {
        use crate::interpreter::value::ValueInner;
        match &*self.0 {
            ValueInner::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// 如果自己是数组，就返回一个 Vec<Value> 的克隆；否则返回 None
    pub fn into_array(self) -> Option<Vec<Value>> {
        use crate::interpreter::value::ValueInner;
        match Arc::try_unwrap(self.0) {
            Ok(inner) => match inner {
                ValueInner::Array(v) => Some(Arc::try_unwrap(v).unwrap_or_else(|v_arc| (*v_arc).clone())),
                _ => None,
            },
            Err(arc) => match &*arc {
                ValueInner::Array(v) => Some((**v).clone()),
                _ => None,
            }
        }
    }
}

// From<String> 转换
impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}


// From<&str> 转换
impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &*self.0)
    }
}

// PartialEq/Eq 根据内部类型实现，忽略 Function/Future
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use ValueInner::*;
        match (&*self.0, &*other.0) {
            (Int(a), Int(b)) => a == b,
            (Long(a), Long(b)) => a == b,
            (Float(a), Float(b)) => (a - b).abs() < f32::EPSILON,
            (Double(a), Double(b)) => (a - b).abs() < f64::EPSILON,
            (Bool(a), Bool(b)) => a == b,
            (Char(a), Char(b)) => a == b,
            (String(a), String(b)) => a == b,
            (Array(a), Array(b)) => a == b,
            (Record(a), Record(b)) => a == b,
            (Module(a), Module(b)) => a == b,
            (Null, Null) => true,
            (Optional(a), Optional(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Value {}

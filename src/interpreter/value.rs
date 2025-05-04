// src/interpreter/value.rs

use crate::ast::param::Param;
use crate::ast::statement::Statement;
use crate::error::error::PawError;
use crate::interpreter::env::Env;
use std::collections::HashMap;
use std::f64;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub enum Value {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    Char(char),
    String(String),
    Array(Vec<Value>),
    Record(HashMap<String, Value>),
    Module(HashMap<String, Value>),
    Function {
        name: String,
        params: Vec<Param>,
        body: Vec<Statement>,
        env: Env,
        is_async: bool,
    },
    Future(Arc<Mutex<Pin<Box<dyn Future<Output = Result<Value, PawError>> + Send>>>>),
    Null,
    Optional(Box<Option<Value>>),
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "Nopaw"),
            Value::String(s) => write!(f, "{}", s.clone()),
            Value::Int(i) => write!(f, "{}", i.to_string()),
            Value::Long(l) => write!(f, "{}", l.to_string()),
            Value::Float(fl) => write!(f, "{}", fl.to_string()),
            Value::Double(d) => write!(f, "{}", d.to_string()),
            Value::Bool(b) => write!(f, "{}", b.to_string()),
            Value::Char(c) => write!(f, "{}", c.to_string()),
            Value::Array(a) => write!(f, "{}", format!("{:?}", a)),
            Value::Function { name, is_async, .. } => {
                if *is_async {
                    write!(f, "<async fun {}>", name)
                } else {
                    write!(f, "<fun {}>", name)
                }
            }
            Value::Record(m) => write!(f, "{}", format!("{:?}", m)),
            Value::Future(_) => write!(f, "<Future>"),
            Value::Optional(opt) => match opt.as_ref() {
                Some(v) => write!(f, "Optional({:?})", v),
                None => write!(f, "Optional(None)"),
            },
            Value::Module(_) => write!(f, "Module"),
        }
    }
}

// 方便比较，Future、Function 不参与 Eq
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => a == b,
            (Long(a), Long(b)) => a == b,
            (Float(a), Float(b)) => (a - b).abs() < f32::EPSILON,
            (Double(a), Double(b)) => (a - b).abs() < f64::EPSILON,
            (Bool(a), Bool(b)) => a == b,
            (Char(a), Char(b)) => a == b,
            (String(a), String(b)) => a == b,
            (Array(a), Array(b)) => a == b,
            (Record(a), Record(b)) => a == b,
            (Null, Null) => true,
            (Optional(a), Optional(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Value {}

use crate::error::error::PawError;
use crate::interpreter::value::Value;
use ahash::AHashMap;
use once_cell::sync::Lazy;
use std::sync::RwLock;

/// Global interface method tables:
/// interface name -> (method name -> method implementation)
pub static INTERFACES: Lazy<
    RwLock<
        AHashMap<
            String,
            AHashMap<String, Box<dyn Fn(Value) -> Result<Value, PawError> + Send + Sync>>
        >
    >
> = Lazy::new(|| RwLock::new(AHashMap::new()));
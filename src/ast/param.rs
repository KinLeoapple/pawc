// src/ast/param.rs

/// 函数参数
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: String,
}

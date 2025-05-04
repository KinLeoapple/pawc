// src/ast/param.rs

/// 函数参数
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: String,
    pub line: usize,
    pub col: usize,
}

impl Param {
    pub fn new(name: String, ty: String, line: usize, col: usize) -> Self {
        Param { name, ty, line, col }
    }
}

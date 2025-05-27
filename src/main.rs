extern crate pest;

mod parser;
mod ast;

use once_cell::sync::OnceCell;

pub static STACK_SIZE: OnceCell<usize> = OnceCell::with_value(1);

fn main() {
    
}

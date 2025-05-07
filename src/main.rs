use crate::cli::cli::run;
use once_cell::sync::OnceCell;
use tokio::runtime::Builder;

mod ast;
mod cli;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod semantic;

pub static STACK_SIZE: OnceCell<usize> = OnceCell::with_value(1);

fn main() {
    let stack_size_bytes = STACK_SIZE.get_or_init(|| 1) * 1024 * 1024;

    let cpus = num_cpus::get().max(1);
    let rt = Builder::new_multi_thread()
        .worker_threads(cpus)
        .thread_stack_size(stack_size_bytes)
        .enable_all()
        .build();

    match rt {
        Ok(rt) => {
            rt.block_on(async { run().await });
        }
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}

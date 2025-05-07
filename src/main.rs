use once_cell::sync::OnceCell;
use stacker::maybe_grow;
use tokio::runtime::Builder;
use crate::cli::cli::run;

mod semantic;
mod parser;
mod lexer;
mod interpreter;
mod ast;
mod cli;
mod error;

pub static STACK_SIZE: OnceCell<usize> = OnceCell::new();
pub static WORKER_STACK_SIZE: OnceCell<usize> = OnceCell::new();

fn main() {
    let stack_mib = *STACK_SIZE.get().unwrap_or(&1);
    let worker_mib = *WORKER_STACK_SIZE.get().unwrap_or(&1);

    let red_zone = 32 * 1024;
    let stack_bytes = stack_mib * 1024 * 1024;
    let worker_bytes = worker_mib * 1024 * 1024;
    
    maybe_grow(32 * 1024, stack_bytes, || {
        // 在这块大栈闭包里，我们构造并跑 Tokio
        let cpus = num_cpus::get().max(1);
        let rt = Builder::new_multi_thread()
            .worker_threads(cpus)
            .thread_stack_size(worker_bytes)
            .enable_all()
            .build();
        match rt { 
            Ok(rt) => {
                rt.block_on(async {
                    async_main().await;
                });
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    });
}

async fn async_main() {
    run().await
} 
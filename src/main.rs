use crate::cli::cli::run;
use include_dir::{include_dir, Dir, DirEntry};
use once_cell::sync::OnceCell;
use std::path::Path;
use std::{fs, io};
use tokio::runtime::Builder;

mod ast;
mod cli;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod semantic;

pub static STACK_SIZE: OnceCell<usize> = OnceCell::with_value(1);

pub static STD_DIR: Dir<'_> = include_dir!("Std");
pub const PROJECT_ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn main() {
    let dump_result = dump_std();
    match dump_result {
        Ok(_) => {}
        Err(_) => {
            eprintln!("Interpreter is corrupted!");
        }
    }
    
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

fn dump_dir(dir: &Dir, out_root: &Path) -> io::Result<()> {
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(subdir) => {
                // 创建对应的子目录
                let dest = out_root.join(subdir.path());
                fs::create_dir_all(&dest)?;
                // 递归
                dump_dir(subdir, out_root)?;
            }
            DirEntry::File(file) => {
                // 写文件
                let dest = out_root.join(file.path());
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&dest, file.contents_utf8().unwrap())?;
            }
        }
    }
    Ok(())
}

pub fn dump_std() -> io::Result<()> {
    let out_root = Path::new(PROJECT_ROOT).join("../std");
    // 确保根目录存在
    fs::create_dir_all(&out_root)?;
    // 递归写出
    dump_dir(&STD_DIR, &out_root)
}
use std::fs;          // 文件系统模块
use std::io::Write;          // 输入输出模块
use std::io;
use std::env;
use std::path::Path;

use nix::sys::signal;


fn main() {
    prompt();
    pwd();
}

fn prompt() -> Option<()> {
    print!("$ ");
    io::stdout().flush().ok()?;
    Some(())
}

fn tokenize(command:String) ->Vec<String>{
    command.split_whitespace()
    .map(|token|{
        if token.starts_with('~'){
            env::var("HOME").unwrap_or_default()+token.strip_prefix('~').unwrap()
        }else if token.starts_with('$'){
            env::var(token.strip_prefix('$').unwrap()).unwrap_or_default()
        }else{token.to_string()}
    }).collect()
}

fn pwd() -> Option<()>{
    let cwd = env::current_dir().ok()?;
    let path = cwd.to_str()?.to_string();
    print!("{}", &path);
    io::stdout().flush().ok()?;
    Some(())
}
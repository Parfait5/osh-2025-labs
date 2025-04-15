use std::fs;          // 文件系统模块
use std::io;          // 输入输出模块
use std::env;
use std::path::Path;

use nix::sys::signal;


fn main() {
    // to do
}

fn prompt() -> Option<()> {
    let cwd = env::current_dir().ok()?;
    let home = env::var("HOME").unwrap_or_default();
    let home_path = Path::new(&home); // 将字符串转为 Path
    let path = 
        if cwd == home_path {
            '~'.to_string
        }else if cwd.starts_with(&home_path){
            "~/".to_string() + cwd.strip_prefix(&home).ok()?.to_str()?
        }else{
            cwd.to_str?.to_string
        };
    print!("{}>",&path);
    io::stdout().flush().ok()?;
    Some(());
}

fn tokenize(command:String) ->Option<Vec(String)>{
    command.split_whiteapce()
    .map(|token|{
        if token.starts_with('~'){
            env::var("HOME").unwrap_or_default()+token.strip_prefix('~').unwrap()
        }else if token.starts_with('$'){
            env::var(token.strip_prefix('$').unwrap()).unwrap_or_default()
        }else{token.to_string()}
    }).collect()
}
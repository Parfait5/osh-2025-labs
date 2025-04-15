use std::io::Write;          // 输入输出模块
use std::io;
use std::env;
use std::ptr;
use std::ffi::OsString;
use std::sync::atomic::{AtomicBool, Ordering, AtomicPtr};

use nix::sys::signal::{signal, SigHandler, Signal};


static INPUTING: AtomicBool = AtomicBool::new(true);
extern "C" fn handle_sigint(_: libc::c_int) {
    println!();
    if INPUTING.load(Ordering::Relaxed) {
        prompt().expect("error print prompt")
    }
}


fn main() {
    unsafe {nix::sys::signal::signal(
        Signal::SIGINT,
        SigHandler::Handler(handle_sigint)
    ).expect("无法注册信号处理程序");}
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

// 使用静态变量存储上一次的目录
static PREV_DIR: AtomicPtr<OsString> = AtomicPtr::new(std::ptr::null_mut());

fn cd(args: &Vec<String>) ->Option<()>{
    let home = env::var("HOME").unwrap_or_default();

    // 获取目标路径
    let target_path = match args.get(0).map(|s| s.as_str()) {
        Some("-") => {
            // 处理 cd - 的情况
            let ptr = PREV_DIR.load(Ordering::Acquire);
            if ptr.is_null() {
                return None;
            }
            unsafe { (*ptr).to_owned().into_string().ok()? }
        }
        Some(path) => path.to_string(),
        None => env::var("HOME").unwrap_or_default(), // 无参数时默认到家目录
    };
    
    // 保存当前目录作为下一次的 PREV_DIR
    let current_dir = env::current_dir().ok()?;
    unsafe {
        PREV_DIR = Some(current_dir.into_os_string());
    }

    // 处理路径中的 ~ 替换
    let resolved_path = if target_path == "~" {
        home
    } else if target_path.starts_with("~/") {
        format!("{}/{}", home.clone(), &target_path[2..])
    } else {
        target_path
    };

    env::set_current_dir(resolved_path).ok()?;
    Some(())
}
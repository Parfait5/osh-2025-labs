use std::io::Write;          // 输入输出模块
use std::io::{self, BufRead};
use std::env;
use std::ptr;
use std::process::{Command, Stdio};
use std::ffi::OsString;
use std::sync::atomic::{AtomicBool, Ordering, AtomicPtr};

use nix::sys::signal::{SigHandler, Signal};


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

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();

    loop{
        INPUTING.store(true, Ordering::Relaxed);
        prompt();

        let mut line = String::new();
        if stdin_lock.read_line(&mut line).is_err() {
            break; // 读入错误（可能是 EOF），退出
        }
        if line.is_empty() {
            break; // EOF（Ctrl+D）
        }
        INPUTING.store(false, Ordering::Relaxed);

        let tokens = tokenize(line.trim().to_string());
        eval(tokens);
        print!("\n");
    }
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

fn eval(tokens: Vec<String>) {
    if tokens.is_empty() {
        return;
    }
    match tokens[0].as_str() {
        "cd" => {
            let _ = cd(&tokens[1..].to_vec());
        }
        "exit" => {
            std::process::exit(0);
        }
        "pwd" =>{
            let _ = pwd();
        }
        _ => {
            // 外部命令处理
            let child = Command::new(&tokens[0])
                    .args(&tokens[1..])
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .spawn();

                match child {
                    Ok(mut child) => {
                        // 父进程等待子进程结束
                        if let Err(e) = child.wait() {
                            eprintln!("Failed to wait for child: {}", e);
                        }
                    }
                    Err(_) => {
                        eprintln!("command not found: {}", tokens[0]);
                    }
                }
            }
        }
    }

fn pwd() -> Option<()>{
    let cwd = env::current_dir().ok()?;
    let path = cwd.to_str()?.to_string();
    print!("{}", &path);
    io::stdout().flush().ok()?;
    Some(())
}

// 使用静态变量存储上一次的目录
static PREV_DIR: AtomicPtr<OsString> = AtomicPtr::new(ptr::null_mut());

fn cd(args: &Vec<String>) -> Option<()> {
    let home = env::var("HOME").unwrap_or_default();

    // 保存当前目录作为 PREV_DIR（先获取当前目录）
    let current_dir = env::current_dir().ok()?;

    // 获取目标路径
    let target_path = match args.get(0).map(|s| s.as_str()) {
        Some("-") => {
            let ptr = PREV_DIR.load(Ordering::Acquire);
            if ptr.is_null() {
                eprintln!("osh: cd: no previous directory");
                return None;
            }
            unsafe { (*ptr).to_owned().into_string().ok()? }
        }
        Some(path) => path.to_string(),
        None => home.clone(),
    };

    // 处理路径中的 ~ 替换
    let resolved_path = if target_path == "~" {
        home.clone()
    } else if target_path.starts_with("~/") {
        format!("{}/{}", home, &target_path[2..])
    } else {
        target_path
    };

    // 切换目录
    if env::set_current_dir(&resolved_path).is_err() {
        eprintln!("osh: cd: no such file or directory: {}", resolved_path);
        return None;
    }

    // 更新 PREV_DIR（释放旧的指针）
    let old_ptr = PREV_DIR.swap(Box::into_raw(Box::new(current_dir.into_os_string())), Ordering::AcqRel);
    if !old_ptr.is_null() {
        unsafe {
            drop(Box::from_raw(old_ptr)); // 释放旧内存，防止泄漏
        }
    }

    Some(())
}
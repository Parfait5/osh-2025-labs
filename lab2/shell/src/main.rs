use std::io::Write;          // 输入输出模块
use std::io::{self, BufRead};
use std::env;
use std::ptr;
use std::fs::{File, OpenOptions};
use std::cmp::min;
use std::process::{Command, Stdio, Child};
use std::ffi::OsString;
use std::sync::{Mutex, atomic::{AtomicBool, Ordering, AtomicPtr}};

use nix::sys::signal::{SigHandler, Signal};
use lazy_static::lazy_static;

lazy_static! {
    static ref BG_CHILDREN: Mutex<Vec<Child>> = Mutex::new(vec![]);
}

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

    // 检查是否是后台执行
    let mut tokens = tokens;
    let background = if tokens.last().map(|s| s == "&").unwrap_or(false) {
        tokens.pop();
        true
    } else {
        false
    };

    // 拆分管道命令
    let mut commands: Vec<Vec<String>> = vec![];
    let mut current_cmd = vec![];
    for token in tokens {
        if token == "|" {
            commands.push(current_cmd);
            current_cmd = vec![];
        } else {
            current_cmd.push(token);
        }
    }
    commands.push(current_cmd);

    let mut previous_stdout = None;
    let mut bg_children_handles: Vec<Child> = vec![];

    for (i, command) in commands.iter().enumerate() {
        let is_last = i == commands.len() - 1;
        let (stdin, child_opt) = match execute(command, is_last, previous_stdout, background) {
            Some(io) => io,
            None => return,
        };
        if let Some(child) = child_opt {
            bg_children_handles.push(child);
        }
        previous_stdout = Some(stdin);
    }

    // 如果是后台运行，则将所有子进程句柄保存在全局后台列表中并立即返回提示
    if background {
        let mut bg_lock = BG_CHILDREN.lock().unwrap();
        for child in bg_children_handles.iter() {
            println!("[后台执行] PID: {}", child.id());
        }
        bg_lock.extend(bg_children_handles);
    }
}

fn execute(
    command: &[String],
    is_last: bool,
    previous_stdout: Option<Stdio>,
    background:bool,
) -> Option<(Stdio, Option<Child>)> {
    let mut stdin = previous_stdout.unwrap_or(Stdio::inherit());
    let mut stdout = if is_last { Stdio::inherit() } else { Stdio::piped() };

    // 处理重定向
    let mut last_command_index = command.len();
    let mut redirect =
        |token: &str, stdio: &mut Stdio, read: bool, write: bool, append: bool| -> Option<()> {
            if let Some(index) = command.iter().position(|t| t == token) {
                let file_path = command.get(index + 1)?;
                if File::open(file_path).is_err() && (write || append) {
                    File::create(file_path).ok()?;
                }
                let file = OpenOptions::new()
                    .read(read)
                    .write(write)
                    .append(append)
                    .create(true)
                    .open(file_path)
                    .ok()?;
                *stdio = Stdio::from(file);
                last_command_index = min(last_command_index, index);
            }
            Some(())
        };
    redirect("<", &mut stdin, true, false, false)?;
    redirect(">", &mut stdout, false, true, false)?;
    redirect(">>", &mut stdout, false, true, true)?;

    let args = &command[..last_command_index];
    if args.is_empty() {
        return None;
    }

    let prog = &args[0];
    let real_args = &args[1..];

    // 内建命令处理
    if prog == "cd" {
        let _ = cd(&real_args.to_vec());
        return None;
    }
    if prog == "exit" {
        std::process::exit(0);
    }
    if prog == "pwd" {
        let _ = pwd();
        return None;
    }
    if prog == "wait" {
        let _ = wait_bg();
        return None;
    }
    if prog == "fg" {
        let pid = args.get(1).and_then(|s| s.parse::<u32>().ok());
        fg(pid);
        return None;
    }
    if prog == "bg" {
        let pid = args.get(1).and_then(|s| s.parse::<u32>().ok());
        bg(pid);
        return None;
    }

    let mut child_builder = Command::new(prog);
    child_builder.args(real_args)
        .stdin(stdin)
        .stdout(stdout)
        .stderr(Stdio::inherit());

    let mut child = match child_builder.spawn() {
        Ok(child) => child,
        Err(_) => {
            eprintln!("command not found: {}", prog);
            return None;
        }
    };

    // 如果是管道中间命令，返回其 stdout 用于下一步管道
    if is_last {
        if background {
            Some((Stdio::null(), Some(child)))
        }else{
            child.wait().ok()?;
            Some((Stdio::null(), None))
        }
    } else {
        // 中间命令需要返回 stdout 管道
        let next_stdout = child.stdout.take().map(Stdio::from)?;
        Some((next_stdout, Some(child)))
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

/// 等待所有后台进程结束
fn wait_bg() {
    let mut bg_lock = BG_CHILDREN.lock().unwrap();
    for child in bg_lock.iter_mut() {
        println!("Waiting for background process {}", child.id());
        let _ = child.wait();
    }
    bg_lock.clear();
}

/// 将指定后台进程（或最近的）切换到前台执行
fn fg(pid_opt: Option<u32>) {
    let mut bg_lock = BG_CHILDREN.lock().unwrap();
    let index = if let Some(pid) = pid_opt {
        bg_lock.iter().position(|child| child.id() == pid)
    } else {
        bg_lock.len().checked_sub(1)
    };

    if let Some(i) = index {
        let mut child = bg_lock.remove(i);
        println!("Bringing process {} to foreground", child.id());
        // 如果进程处于暂停状态，发送 SIGCONT 使之继续
        unsafe {
            libc::kill(child.id() as i32, libc::SIGCONT);
        }
        let _ = child.wait();
    } else {
        eprintln!("fg: no such process");
    }
}

/// 恢复暂停的后台进程运行（不等待）
fn bg(pid_opt: Option<u32>) {
    let bg_lock = BG_CHILDREN.lock().unwrap();
    let child_opt = if let Some(pid) = pid_opt {
        bg_lock.iter().find(|child| child.id() == pid)
    } else {
        bg_lock.last()
    };

    if let Some(child) = child_opt {
        println!("Resuming background process {}", child.id());
        unsafe {
            libc::kill(child.id() as i32, libc::SIGCONT);
        }
    } else {
        eprintln!("bg: no such process");
    }
}

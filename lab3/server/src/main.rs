use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    sync::{Arc, Mutex},
    thread,
};
use std::sync::mpsc::{self, Sender, Receiver};
use std::time::SystemTime;


// ---------- 线程池定义 ----------
struct ThreadPool {
    workers: Vec<thread::JoinHandle<()>>,
    sender: Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver): (Sender<Job>, Receiver<Job>) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for _ in 0..size {
            let receiver = Arc::clone(&receiver);
            workers.push(thread::spawn(move || loop {
                let job = receiver.lock().unwrap().recv().unwrap();
                job();
            }));
        }

        ThreadPool { workers, sender }
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(f)).unwrap();
    }
}

// ---------- 缓存结构 ----------
struct CacheEntry {
    data: Vec<u8>,
    modified: SystemTime,
}

#[derive(Clone)]
struct Cache {
    map: Arc<Mutex<HashMap<String, CacheEntry>>>,
}

impl Cache {
    fn new() -> Cache {
        Cache {
            map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn get_or_load(&self, path: &str) -> Option<Vec<u8>> {
        let mut cache = self.map.lock().unwrap();
    
        let file_path = Path::new(".").join(&path[1..]);
        let metadata = fs::metadata(&file_path).ok()?;
        let modified = metadata.modified().ok()?;
    
        if let Some(entry) = cache.get(path) {
            if entry.modified == modified {
                return Some(entry.data.clone());
            }
        }
    
        let data = fs::read(&file_path).ok()?;
        cache.insert(
            path.to_string(),
            CacheEntry {
                data: data.clone(),
                modified,
            },
        );
        Some(data)
    }

    // 可选：缓存失效机制（例如手动清除，或定期清理）
}

// ---------- 主函数 ----------
fn main() {
    let listener = TcpListener::bind("0.0.0.0:8000").expect("Failed to bind address");
    println!("Listening on http://0.0.0.0:8000");

    let pool = ThreadPool::new(4); // 可配置线程数量
    let cache = Cache::new();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let cache = cache.clone();
                pool.execute(move || {
                    if let Err(e) = handle_client(stream, &cache) {
                        eprintln!("Client handling error: {}", e);
                    }
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}

// ---------- 响应处理 ----------
fn handle_client(mut stream: TcpStream, cache: &Cache) -> std::io::Result<()> {
    let mut buffer = [0u8; 4096];
    let mut request_data = Vec::new();

    loop {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        request_data.extend_from_slice(&buffer[..bytes_read]);
        if bytes_read < buffer.len() {
            break;
        }
    }

    let request = String::from_utf8_lossy(&request_data);
    let (method, path, version) = match parse_request_line(&request) {
        Ok(parts) => parts,
        Err(status) => {
            respond_with_status(&mut stream, status, None)?;
            return Ok(());
        }
    };

    if method != "GET" || version != "HTTP/1.0" {
        respond_with_status(&mut stream, 500, None)?;
        return Ok(());
    }

    let clean_path = if path == "/" { "/index.html" } else { &path };
    if clean_path.contains("..") {
        respond_with_status(&mut stream, 403, None)?;
        return Ok(());
    }

    // 从缓存中读取（不存在时从磁盘加载）
    match cache.get_or_load(clean_path) {
        Some(content) => {
            let header = format!(
                "HTTP/1.0 200 OK\r\nContent-Length: {}\r\n\r\n",
                content.len()
            );
            stream.write_all(header.as_bytes())?;
            stream.write_all(&content)?;
        }
        None => {
            respond_with_status(&mut stream, 404, None)?;
        }
    }
    Ok(())
}

/// 返回指定状态码和可选消息体
fn respond_with_status(stream: &mut TcpStream, code: u16, body: Option<&[u8]>) -> std::io::Result<()> {
    let (status_text, default_body) = match code {
        200 => ("200 OK", b"" as &[u8]),
        403 => ("403 Forbidden", b"403 Forbidden" as &[u8]),
        404 => ("404 Not Found", b"404 Not Found" as &[u8]),
        500 => ("500 Internal Server Error", b"500 Internal Server Error" as &[u8]),
        _ => ("500 Internal Server Error", b"500 Internal Server Error" as &[u8]),
    };
    let body = body.unwrap_or(default_body);
    let header = format!(
        "HTTP/1.0 {}\r\nContent-Length: {}\r\n\r\n",
        status_text,
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    if !body.is_empty() {
        stream.write_all(body)?;
    }
    Ok(())
}

/// 解析请求行，返回 (method, path, version)
/// 错误时返回对应 HTTP 状态码
fn parse_request_line(request: &str) -> Result<(&str, &str, &str), u16> {
    let mut lines = request.lines();
    let request_line = lines.next().ok_or(500u16)?;
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() != 3 {
        return Err(500);
    }
    Ok((parts[0], parts[1], parts[2]))
}

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

fn main(){
    // 绑定地址并监听
    let listener = TcpListener::bind("0.0.0.0:8000").expect("Failed to bind address");
    println!("Listening on http://0.0.0.0:8000");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // 并发处理：为每个连接创建新线程
                thread::spawn(|| {
                    if let Err(e) = handle_client(stream) {
                        eprintln!("Client handling error: {}", e);
                    }
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}

/// 返回指定状态码和可选消息体
fn respond_with_status(stream: &mut TcpStream, code: u16, body: Option<&[u8]>) -> std::io::Result<()> {
    let (status_text, default_body) = match code {
        200 => ("200 OK", b"" as &[u8]),
        403 => ("403 Forbidden", b"Forbidden" as &[u8]),
        404 => ("404 Not Found", b"Not Found" as &[u8]),
        500 => ("500 Internal Server Error", b"Internal Server Error" as &[u8]),
        _ => ("500 Internal Server Error", b"Internal Server Error" as &[u8]),
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
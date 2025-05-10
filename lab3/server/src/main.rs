use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

fn main(){
    // 绑定地址并监听
    let listener = TcpListener::bind("0.0.0.0:8000").expect("Failed to bind address");
    println!("Listening on http://0.0.0.0:8000");
}
use clap::Parser;
#[allow(unused_imports)]
use std::net::TcpListener;
use std::{
    collections::HashMap,
    fs::{read, write},
    io::{Read, Write},
    path::PathBuf,
    thread,
};
#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long, default_value_t = String::new())]
    /// root directory
    directory: String,
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                thread::spawn(move || {
                    let root_dir = Cli::parse();
                    let mut net_buff = [0; 1024];
                    #[allow(clippy::unused_io_amount)]
                    _stream.read(&mut net_buff).unwrap();
                    let splited_request: Vec<&[u8]> =
                        net_buff.split(|&byte| byte == b'\r').collect();
                    let request_body = splited_request.last().unwrap();

                    // Extracting headers into a hashMap
                    let mut header_hash: HashMap<String, String> = HashMap::new();
                    let mut header_buffer = String::new();
                    for request in &splited_request {
                        for ch in request.utf8_chunks() {
                            for char in ch.valid().chars() {
                                header_buffer.push(char);
                            }
                            header_buffer = header_buffer.replace("\n", "");
                            let key_value: Vec<&str> = header_buffer.split(":").collect();
                            if key_value.len() > 1 {
                                header_hash.insert(
                                    key_value[0].trim().to_string(),
                                    key_value[1].trim().to_string(),
                                );
                            }
                            header_buffer = String::new();
                        }
                    }

                    let request_line: Vec<&[u8]> = splited_request
                        .first()
                        .unwrap()
                        .split(|&byte| byte == b' ')
                        .collect();
                    match request_line[0] {
                        b"GET" => {
                            let url_path: Vec<&[u8]> =
                                request_line[1].split(|&byte| byte == b'/').collect();
                            match url_path[1] {
                                b"echo" => {
                                    let utf8_string = bytes_to_utf8(url_path[2]);
                                    let http_responce = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", utf8_string.len(), utf8_string);
                                    _stream.write_all(http_responce.as_bytes()).unwrap();
                                }
                                b"user-agent" => {
                                    let user_agent = header_hash.get("User-Agent").unwrap();
                                    let http_responce = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", user_agent.len(), user_agent);
                                    _stream.write_all(http_responce.as_bytes()).unwrap();
                                }
                                b"files" => {
                                    let path = PathBuf::from(root_dir.directory);
                                    let path = path.join(bytes_to_utf8(url_path[2]));
                                    if let Ok(file) = read(path) {
                                        let http_responce = format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}", file.len(), bytes_to_utf8(&file));
                                        _stream.write_all(http_responce.as_bytes()).unwrap();
                                    } else {
                                        _stream
                                            .write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
                                            .unwrap();
                                    }
                                }
                                b"" => {
                                    _stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
                                }
                                _ => {
                                    _stream
                                        .write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
                                        .unwrap();
                                }
                            }
                        }
                        b"POST" => {
                            let file_path: Vec<&[u8]> =
                                request_line[1].split(|&byte| byte == b'/').collect();
                            let path = PathBuf::from(root_dir.directory);
                            let path = path.join(bytes_to_utf8(file_path[2]));
                            if write(path, bytes_to_utf8(request_body)).is_ok() {
                                _stream.write_all(b"HTTP/1.1 201 Created\r\n\r\n").unwrap();
                            } else {
                                _stream
                                    .write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
                                    .unwrap();
                            }
                        }
                        _ => {
                            _stream
                                .write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
                                .unwrap();
                        }
                    }
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn bytes_to_utf8(bytes: &[u8]) -> String {
    let mut utf8_string = String::new();
    let content_utf8 = bytes.utf8_chunks();
    for ch in content_utf8 {
        for char in ch.valid().chars() {
            utf8_string.push(char);
        }
    }
    utf8_string.trim().replace("\0", "")
}

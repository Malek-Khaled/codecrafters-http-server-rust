use std::io::{Read, Write};
#[allow(unused_imports)]
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let mut net_buff = [0; 100];
                _stream.read(&mut net_buff);
                let splited_request: Vec<&[u8]> = net_buff.split(|&byte| byte == b'\r').collect();
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
                                let mut utf8_string = String::new();
                                let content_utf8 = url_path[2].utf8_chunks();
                                for ch in content_utf8 {
                                    for char in ch.valid().chars() {
                                        utf8_string.push(char);
                                    }
                                }
                                let http_responce = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", utf8_string.len(), utf8_string);
                                _stream.write_all(http_responce.as_bytes()).unwrap();
                            }
                            _ => {
                                _stream
                                    .write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
                                    .unwrap();
                            }
                        }
                    }
                    _ => {
                        _stream
                            .write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
                            .unwrap();
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

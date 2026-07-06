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
                let http_responce = b"HTTP/1.1 200 OK\r\n\r\n";
                let splited_request: Vec<&[u8]> = net_buff.split(|&byte| byte == b'\r').collect();
                let request_line: Vec<&[u8]> = splited_request
                    .first()
                    .unwrap()
                    .split(|&byte| byte == b' ')
                    .collect();
                let http_responce: &[u8] = if request_line[1] == b"/" {
                    "HTTP/1.1 200 OK\r\n\r\n".as_bytes()
                } else {
                    "HTTP/1.1 404 Not Found\r\n\r\n".as_bytes()
                };
                _stream.write_all(http_responce).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

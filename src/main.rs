use std::io::{Read, Write};
#[allow(unused_imports)]
use std::net::TcpListener;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let mut net_buff = String::new();
                _stream.read_to_string(&mut net_buff).unwrap();
                let splited_request: Vec<&str> = net_buff.split("\r\n").collect();
                let request_line: Vec<&str> = splited_request.first().unwrap().split(" ").collect();
                let http_responce = if request_line.get(1).unwrap().eq(&"/") {
                    "HTTP/1.1 200 OK\r\n\r\n".to_string()
                } else {
                    "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
                };
                _stream.write_all(http_responce.as_bytes()).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

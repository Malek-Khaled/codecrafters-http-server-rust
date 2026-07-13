use clap::Parser;
use flate2::{
    read::{GzDecoder, GzEncoder},
    Compression,
};
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

                    let request_line: Vec<&[u8]> = splited_request
                        .first()
                        .unwrap()
                        .split(|&byte| byte == b' ')
                        .collect();
                    let request_body = splited_request.last().unwrap();
                    // Extracting headers into a hashMap
                    let mut headers_hash: HashMap<String, String> = HashMap::new();
                    let mut header_buffer = String::new();
                    for request in &splited_request {
                        for ch in request.utf8_chunks() {
                            for char in ch.valid().chars() {
                                header_buffer.push(char);
                            }
                            header_buffer = header_buffer.replace("\n", "");
                            let key_value: Vec<&str> = header_buffer.split(":").collect();
                            if key_value.len() > 1 {
                                headers_hash.insert(
                                    key_value[0].trim().to_string(),
                                    key_value[1].trim().to_string(),
                                );
                            }
                            header_buffer = String::new();
                        }
                    }

                    let encoders: Vec<&str> =
                        if let Some(encoders) = headers_hash.get("Accept-Encoding") {
                            encoders.split(",").map(|encoder| encoder.trim()).collect()
                        } else {
                            vec![]
                        };

                    match request_line[0] {
                        b"GET" => {
                            let mut _status = String::from("HTTP/1.1 200 OK\r\n");
                            let mut _content_type = String::new();
                            let mut _content_encoding = String::new();
                            let supported_encoder: &str = "gzip";

                            if encoders.contains(&supported_encoder) {
                                _content_encoding = String::from("Content-Encoding: gzip\r\n");
                            }
                            let mut _content_length = String::new();
                            let mut _content_body: Box<&[u8]> = Box::default();
                            let mut _bff: Vec<u8> = Vec::new();
                            let url_path: Vec<&[u8]> =
                                request_line[1].split(|&byte| byte == b'/').collect();
                            match url_path[1] {
                                b"echo" => {
                                    let utf8_string = bytes_to_utf8(url_path[2]);
                                    _content_length =
                                        format!("Content-Length: {}\r\n", utf8_string.len());
                                    // _content_body = utf8_string;
                                    // _content_body = vec![url_path[2]];
                                    _content_body = Box::from(url_path[2]);
                                    _content_type = String::from("Content-Type: text/plain\r\n");
                                }
                                b"user-agent" => {
                                    let user_agent = headers_hash.get("User-Agent").unwrap();
                                    _content_length =
                                        format!("Content-Length: {}\r\n", user_agent.len());
                                    // _content_body = user_agent.clone();
                                    // _content_body = vec![user_agent.clone().as_bytes()];
                                    _content_body = Box::from(user_agent.as_bytes());
                                    _content_type = String::from("Content-Type: text/plain\r\n");
                                }
                                b"files" => {
                                    let path = PathBuf::from(root_dir.directory);
                                    let path = path.join(bytes_to_utf8(url_path[2]));
                                    if let Ok(file) = read(path) {
                                        _content_length =
                                            format!("Content-Length: {}\r\n", file.len());
                                        _bff = file;
                                        let file_array: &[u8] = &_bff;
                                        _content_body = Box::from(file_array);
                                        _content_type = String::from(
                                            "Content-Type: application/octet-stream\r\n",
                                        );
                                    } else {
                                        _status = String::from("HTTP/1.1 404 Not Found\r\n");
                                    }
                                }
                                b"" => {}
                                _ => {
                                    _status = String::from("HTTP/1.1 404 Not Found\r\n");
                                }
                            }
                            // let mut compressor = GzEncoder::new(Vec::new(), Compression::default());
                            let mut _encoded: Vec<u8> = Vec::new();
                            if !&_content_encoding.is_empty() {
                                // let byte_string = b"hello world";
                                // compressor.write_all(&_content_body).unwrap();
                                // _encoded = compressor.finish().unwrap();
                                let mut gz =
                                    GzEncoder::new(&_content_body[..], Compression::default());
                                gz.read_to_end(&mut _encoded).unwrap();
                            }
                            let mut grg = GzDecoder::new(&_encoded[..]);
                            let mut decoded = String::new();
                            grg.read_to_string(&mut decoded).unwrap();
                            dbg!(&decoded);
                            dbg!(&decoded.len());

                            dbg!(!_encoded.is_empty());
                            if !_encoded.is_empty() {
                                let formated_responce = format!(
                                    "{_status}{_content_type}{_content_encoding}Content-Length: {length}\r\n\r\n",
                                    length = _encoded.len()
                                );
                                let formated_responce: &[u8] = formated_responce.as_bytes();
                                let http_responce: Vec<u8> =
                                    [formated_responce, &_encoded[..]].concat();
                                _stream.write_all(&http_responce).unwrap();
                            } else {
                                let formated_responce = format!(
                                "{_status}{_content_type}{_content_encoding}{_content_length}\r\n",
                            );
                                let formated_responce: &[u8] = formated_responce.as_bytes();
                                let http_responce: Vec<u8> =
                                    [formated_responce, &_content_body].concat();
                                _stream.write_all(&http_responce).unwrap();
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

// fn forming_responce(
//     content_type: &str,
//     length: &str,
//     encoding_method: &str,
//     status: &str,
//     // body: &str,
// ) -> Vec<[u8]> {
//     let formated_responce = format!("{status}{content_type}{encoding_method}{length}\r\n");
//     vec![formated_responce.as_bytes()]
// }

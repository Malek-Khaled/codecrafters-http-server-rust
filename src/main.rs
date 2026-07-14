use clap::Parser;
use flate2::{read::GzEncoder, Compression};
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
                    let root_dir = Cli::parse().directory;
                    let mut net_buff = [0; 8142];
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
                    for request in &splited_request {
                        let mut header_buffer = bytes_to_utf8(request);
                        header_buffer = header_buffer.replace("\n", "");
                        let key_value: Vec<&str> = header_buffer.split(":").collect();
                        if key_value.len() > 1 {
                            headers_hash.insert(
                                key_value[0].trim().to_string(),
                                key_value[1].trim().to_string(),
                            );
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
                            let mut status: Vec<u8> = Vec::from(b"HTTP/1.1 200 OK\r\n");
                            let mut content_type: Vec<u8> = Vec::new();
                            let mut content_encoding: Vec<u8> = Vec::new();
                            let supported_encoder = "gzip";

                            if encoders.contains(&supported_encoder) {
                                content_encoding =
                                    [b"Content-Encoding: ", supported_encoder.as_bytes(), b"\r\n"]
                                        .concat();
                            }
                            let mut _content_body: Vec<u8> = Vec::new();
                            let url_path: Vec<&[u8]> =
                                request_line[1].split(|&byte| byte == b'/').collect();
                            match url_path[1] {
                                b"echo" => {
                                    _content_body = url_path[2].to_vec();
                                    content_type = b"Content-Type: text/plain\r\n".to_vec();
                                }
                                b"user-agent" => {
                                    let user_agent = headers_hash.get("User-Agent").unwrap();
                                    _content_body = user_agent.as_bytes().to_vec();
                                    content_type = b"Content-Type: text/plain\r\n".to_vec();
                                }
                                b"files" => {
                                    let path = PathBuf::from(root_dir);
                                    let path = path.join(bytes_to_utf8(url_path[2]));
                                    if let Ok(file) = read(path) {
                                        _content_body = file.to_vec();
                                        content_type =
                                            b"Content-Type: application/octet-stream\r\n".to_vec();
                                    } else {
                                        status = b"HTTP/1.1 404 Not Found\r\n".to_vec();
                                    }
                                }
                                b"" => {}
                                _ => {
                                    status = b"HTTP/1.1 404 Not Found\r\n".to_vec();
                                }
                            }
                            if !&content_encoding.is_empty() {
                                let buffer = _content_body.clone();
                                _content_body.clear();
                                let mut gz = GzEncoder::new(&buffer[..], Compression::default());
                                gz.read_to_end(&mut _content_body).unwrap();
                            }

                            let formatted_responce = forming_responce(
                                &content_type,
                                _content_body.len(),
                                &content_encoding,
                                &status,
                                &_content_body,
                            );
                            _stream.write_all(&formatted_responce).unwrap();
                        }
                        b"POST" => {
                            let file_path: Vec<&[u8]> =
                                request_line[1].split(|&byte| byte == b'/').collect();
                            let path = PathBuf::from(root_dir);
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

fn forming_responce(
    content_type: &[u8],
    length: usize,
    encoding_method: &[u8],
    status: &[u8],
    body: &[u8],
) -> Vec<u8> {
    let formatted = [
        status,
        content_type,
        encoding_method,
        b"Content-Length: ",
        length.to_string().as_bytes(),
        b"\r\n\r\n",
        body,
    ]
    .concat();
    formatted
}

mod file_handler;
mod handlers;
mod http_request;
mod http_response;

use std::{io::Read, net::TcpListener};
use std::{str, thread};

use crate::handlers::*;
use crate::http_request::HttpRequest;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        thread::spawn(|| match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let mut buf = vec![0; 1024];

                match stream.read(&mut buf) {
                    Ok(len) => {
                        let val = str::from_utf8(&buf[..len]).unwrap();
                        let http = HttpRequest::parse(val);

                        if http.path == "/" {
                            handle_root_path(&mut stream, &http);
                        } else if http.path == "/user-agent" {
                            handle_user_agent(&mut stream, &http);
                        } else if http.path.starts_with("/echo/") {
                            handle_echo(&mut stream, &http);
                        } else if http.method == "GET" && http.path.starts_with("/files/") {
                            handle_get_file(&mut stream, &http);
                        } else if http.method == "POST" && http.path.starts_with("/files/") {
                            handle_post_file(&mut stream, &http);
                        } else {
                            handle_not_found(&mut stream, &http);
                        }
                    }
                    Err(e) => panic!("encountered IO error: {e}"),
                };
            }
            Err(e) => {
                println!("error: {}", e);
            }
        });
    }
}

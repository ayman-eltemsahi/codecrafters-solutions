use std::net::TcpStream;

use crate::file_handler::{read_file_content, write_file_content};
use crate::http_request::HttpRequest;
use crate::http_response::{
    write_http_response, write_status_code, HttpContentType, HttpResponse, HttpStatusCode,
};

pub fn handle_root_path(stream: &mut TcpStream, _http: &HttpRequest) {
    write_status_code(stream, HttpStatusCode::Ok);
}

pub fn handle_not_found(stream: &mut TcpStream, _http: &HttpRequest) {
    write_status_code(stream, HttpStatusCode::NotFound);
}

pub fn handle_user_agent(stream: &mut TcpStream, http: &HttpRequest) {
    let user_agent = http
        .get_header("User-Agent")
        .expect("Could not find a user agent in the headers");

    let response = HttpResponse {
        status_code: HttpStatusCode::Ok,
        content_type: HttpContentType::PlainText,
        content: Some(user_agent),
    };

    write_http_response(stream, &response);
}

pub fn handle_echo(stream: &mut TcpStream, http: &HttpRequest) {
    let (_, content) = http.path.split_once("/echo/").unwrap();

    let response = HttpResponse {
        status_code: HttpStatusCode::Ok,
        content_type: HttpContentType::PlainText,
        content: Some(content.to_string()),
    };

    write_http_response(stream, &response);
}

pub fn handle_get_file(stream: &mut TcpStream, http: &HttpRequest) {
    let (_, filename) = http.path.split_once("/files/").unwrap();
    match read_file_content(filename) {
        Some(content) => {
            let response = HttpResponse {
                status_code: HttpStatusCode::Ok,
                content_type: HttpContentType::ApplicationOctetStream,
                content: Some(content.to_string()),
            };

            write_http_response(stream, &response);
        }
        None => write_status_code(stream, HttpStatusCode::NotFound),
    }
}

pub fn handle_post_file(stream: &mut TcpStream, http: &HttpRequest) {
    let (_, filename) = http.path.split_once("/files/").unwrap();

    match &http.body {
        Some(body) => match write_file_content(filename, &body) {
            Ok(_) => write_status_code(stream, HttpStatusCode::Created),
            Err(e) => {
                panic!("Could not write the file: {}", e);
            }
        },
        None => {
            panic!("Could not find a request body");
        }
    }
}

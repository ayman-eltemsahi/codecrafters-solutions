use std::io::Write;
use std::net::TcpStream;

const CRLF: &str = "\r\n";

#[derive(Debug, Clone)]
pub enum HttpStatusCode {
    Ok,
    Created,
    NotFound,
}

impl HttpStatusCode {
    fn as_str(&self) -> &'static str {
        match self {
            HttpStatusCode::Ok => "HTTP/1.1 200 OK",
            HttpStatusCode::Created => "HTTP/1.1 201 Created",
            HttpStatusCode::NotFound => "HTTP/1.1 404 Not Found",
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum HttpContentType {
    ApplicationJson,
    PlainText,
    ApplicationOctetStream,
}

impl HttpContentType {
    fn as_str(&self) -> &'static str {
        match self {
            HttpContentType::ApplicationJson => "application/json",
            HttpContentType::PlainText => "text/plain",
            HttpContentType::ApplicationOctetStream => "application/octet-stream",
        }
    }
}

pub fn write_status_code(stream: &mut TcpStream, status_code: HttpStatusCode) {
    let buf = format!("{}{}{}", status_code.as_str(), CRLF, CRLF);
    match stream.write(&buf.as_bytes()) {
        Ok(_res) => {}
        Err(err) => eprintln!("Failed while writing: {}", err),
    }
    stream.flush().expect("failed to flush the stream");
}

pub fn write_http_response(stream: &mut TcpStream, response: &HttpResponse) {
    let buf = match &response.content {
        Some(content) => format!(
            "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n",
            response.status_code.as_str(),
            response.content_type.as_str(),
            content.len(),
            content
        ),
        None => format!(
            "{}\r\nContent-Type: {}\r\n\r\n\r\n",
            response.status_code.as_str(),
            response.content_type.as_str(),
        ),
    };

    match stream.write(&buf.as_bytes()) {
        Ok(_res) => {}
        Err(err) => eprintln!("Failed while writing: {}", err),
    }
    stream.flush().expect("failed to flush the stream");
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: HttpStatusCode,
    pub content_type: HttpContentType,
    pub content: Option<String>,
}

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub body: Option<String>,
    headers: HashMap<String, String>,
}

impl HttpRequest {
    pub fn parse(val: &str) -> HttpRequest {
        let request_lines: Vec<&str> = val.split("\r\n").collect();
        let request_first_line: Vec<&str> = request_lines[0].split(" ").collect();
        let method = request_first_line[0].to_owned();
        let path = if request_first_line.len() > 1 {
            request_first_line[1].to_owned()
        } else {
            "".to_owned()
        };
        let version = if request_first_line.len() > 2 {
            request_first_line[2].to_owned()
        } else {
            "".to_owned()
        };

        let mut headers = HashMap::new();

        for seg in request_lines.iter().skip(1).take_while(|l| l.len() > 0) {
            let (name, value) = seg.split_once(':').expect("invalid header value");
            headers.insert(name.to_lowercase(), value.trim_start().to_owned());
        }

        let body = match val.split_once("\r\n\r\n") {
            Some((_, body)) => Some(body.to_owned()),
            None => None,
        };

        HttpRequest {
            method,
            path,
            version,
            headers,
            body,
        }
    }

    pub fn get_header(&self, val: &str) -> Option<&String> {
        self.headers.get(&val.to_lowercase())
    }
}

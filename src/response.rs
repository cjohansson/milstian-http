use std::collections::HashMap;
use std::str;

pub struct Message {
    protocol: String,
    pub status: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Message {
    pub fn new(
        protocol: String,
        status: String,
        headers: HashMap<String, String>,
        body: Vec<u8>,
    ) -> Message {
        Message {
            protocol,
            status,
            headers,
            body,
        }
    }

    pub fn _header_to_string(&self) -> String {
        let mut response = format!("{} {}\r\n", &self.protocol, &self.status);

        if !&self.headers.is_empty() {
            let mut headers: Vec<(&String, &String)> = self.headers.iter().collect();
            headers.sort_by(|a, b| a.cmp(b));
            for (key, value) in headers {
                response.push_str(&format!("{}: {}\r\n", &key, &value));
            }
            response.push_str("\r\n");
        }

        response
    }

    pub fn _to_string(&self) -> String {
        let mut response = format!("{} {}\r\n", &self.protocol, &self.status);

        if !&self.headers.is_empty() {
            let mut headers: Vec<(&String, &String)> = self.headers.iter().collect();
            headers.sort_by(|a, b| a.cmp(b));
            for (key, value) in headers {
                response.push_str(&format!("{}: {}\r\n", &key, &value));
            }
        }
        response.push_str("\r\n");

        if !&self.body.is_empty() {
            if let Ok(body_string) = str::from_utf8(&self.body) {
                response.push_str(body_string);
            }
        }

        response
    }

    pub fn to_bytes(&mut self) -> Vec<u8> {
        let mut response = format!("{} {}\r\n", &self.protocol, &self.status).into_bytes();

        if !&self.headers.is_empty() {
            let mut headers: Vec<(&String, &String)> = self.headers.iter().collect();
            headers.sort_by(|a, b| a.cmp(b));
            for (key, value) in headers {
                response.append(&mut format!("{}: {}\r\n", &key, &value).into_bytes());
            }
        }
        response.append(&mut "\r\n".to_string().into_bytes());

        if !&self.body.is_empty() {
            response.append(&mut self.body);
        }

        response
    }
}

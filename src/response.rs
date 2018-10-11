//! # Handles everything related to HTTP responses.

use std::collections::HashMap;
use std::str;

/// # A request message
pub struct Message {
    pub protocol: String,
    pub status: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Message {
    /// # Create a new HTTP Message
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

    /// # Get the HTTP header as a new string
    /// ```rust
    /// use milstian_http::response::Message;
    /// use std::collections::HashMap;
    /// assert_eq!(
    ///     Message::new(
    ///         "HTTP/1.0".to_string(),
    ///         "200 OK".to_string(),
    ///         HashMap::new(),
    ///         b"<html><body>Nothing here</body></html>".to_vec()
    ///     ).header_to_string(),
    ///     "HTTP/1.0 200 OK\r\n\r\n".to_string()
    /// );    
    /// ```
    pub fn header_to_string(&self) -> String {
        let mut response = format!("{} {}\r\n", &self.protocol, &self.status);

        if !&self.headers.is_empty() {
            let mut headers: Vec<(&String, &String)> = self.headers.iter().collect();
            headers.sort_by(|a, b| a.cmp(b));
            for (key, value) in headers {
                response.push_str(&format!("{}: {}\r\n", &key, &value));
            }
        }
        response.push_str("\r\n");

        response
    }

    /// # Convert response message into a string
    /// ```rust
    /// use milstian_http::response::Message;
    /// use std::collections::HashMap;
    /// assert_eq!(
    ///     Message::new(
    ///         "HTTP/1.0".to_string(),
    ///         "200 OK".to_string(),
    ///         HashMap::new(),
    ///         b"<html><body>Nothing here</body></html>".to_vec()
    ///     ).to_string(),
    ///     "HTTP/1.0 200 OK\r\n\r\n<html><body>Nothing here</body></html>".to_string()
    /// );
    /// ```
    pub fn to_string(&mut self) -> String {
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

    /// # Convert message into bytes
    /// ```rust
    /// use milstian_http::response::Message;
    /// use std::collections::HashMap;
    /// assert_eq!(
    ///     Message::new(
    ///         "HTTP/1.0".to_string(),
    ///         "200 OK".to_string(),
    ///         HashMap::new(),
    ///         b"<html><body>Nothing here</body></html>".to_vec()
    ///     ).to_bytes(),
    ///     b"HTTP/1.0 200 OK\r\n\r\n<html><body>Nothing here</body></html>".to_vec()
    /// );
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_string() {
        let mut message = Message::new(
            "HTTP/1.0".to_string(),
            "200 OK".to_string(),
            HashMap::new(),
            b"<html><body>Nothing here</body></html>".to_vec(),
        );
        assert_eq!(
            message.to_string(),
            "HTTP/1.0 200 OK\r\n\r\n<html><body>Nothing here</body></html>".to_string()
        );
    }

    #[test]
    fn test_to_bytes() {
        let mut message = Message::new(
            "HTTP/1.0".to_string(),
            "200 OK".to_string(),
            HashMap::new(),
            b"<html><body>Nothing here</body></html>".to_vec(),
        );
        assert_eq!(
            message.to_bytes(),
            b"HTTP/1.0 200 OK\r\n\r\n<html><body>Nothing here</body></html>".to_vec()
        );
    }

}

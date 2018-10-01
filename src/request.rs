//! # Handles everything related to HTTP requests.

use std::collections::HashMap;
use std::str;

#[derive(Debug)]
pub enum BodyContentType {
    SinglePart(HashMap<String, String>),
    MultiPart(HashMap<String, MultiPartValue>),
}

#[derive(Debug)]
pub struct Message {
    pub body: BodyContentType,
    pub headers: HashMap<String, HeaderValueParts>,
    pub request_line: Line,
}

#[derive(Debug)]
pub struct Line {
    pub method: Method,
    pub protocol: Protocol,
    pub request_uri: String,
    pub request_uri_base: String,
    pub query_arguments: HashMap<String, String>,
    pub query_string: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Method {
    Connect,
    Delete,
    Get,
    Head,
    Invalid,
    Options,
    Patch,
    Post,
    Put,
    Trace,
}

#[derive(Debug)]
pub enum HeaderContentType {
    MultiPart(String), // String is multi-part boundary string
    SinglePart,
}

#[derive(Debug)]
pub enum HeaderValuePart {
    Single(String),
    KeyValue(String, String),
}

#[derive(Debug)]
pub struct HeaderValueParts {
    pub parts: Vec<Vec<HeaderValuePart>>,
}

impl HeaderValueParts {
    pub fn get_key_value(&self, key: &str) -> Option<String> {
        for params_block in self.parts.iter() {
            for params_subblock in params_block.iter() {
                if let HeaderValuePart::KeyValue(key_value_key, key_value_value) = params_subblock {
                    if key_value_key == key {
                        return Some(key_value_value.to_string());
                    }
                }
            }
        }
        None
    }

    pub fn to_string(&self) -> String {
        let mut output = String::new();
        let mut params_block_count = 0;
        for params_block in self.parts.iter() {
            if params_block_count > 0 {
                output.push_str("; ");
            }
            let mut params_subblock_count = 0;
            for params_subblock in params_block.iter() {
                if params_subblock_count > 0 {
                    output.push_str(", ");
                }
                match params_subblock {
                    HeaderValuePart::Single(string) => {
                        output.push_str(&string);
                    }
                    HeaderValuePart::KeyValue(key, value) => {
                        output.push_str(&format!("{}={}", key, value).to_string());
                    }
                }
                params_subblock_count = params_subblock_count + 1;
            }
            params_block_count = params_block_count + 1;
        }
        output
    }
}

#[derive(Debug)]
pub struct MultiPartValue {
    pub body: Vec<u8>,
    pub headers: HashMap<String, HeaderValueParts>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Protocol {
    Invalid,
    V1_0,
    V1_1,
    V2_0,
    V0_9,
}

enum ParserSection {
    Line,
    HeaderFields,
    MessageBody,
}

enum MultiPartSection {
    End,
    EndBoundary,
    Skipping,
    Start,
    StartSuffix,
}

enum ParserMode {
    Boundaries(Vec<u8>),
    Lines,
}

#[derive(Debug, Eq, PartialEq)]
enum SettingValence {
    Optional,
    No,
    Yes,
}

impl Message {
    fn method_has_request_body(method: &Method) -> SettingValence {
        match method {
            Method::Connect => SettingValence::Yes,
            Method::Delete => SettingValence::No,
            Method::Get => SettingValence::Optional,
            Method::Head => SettingValence::No,
            Method::Options => SettingValence::Optional,
            Method::Patch => SettingValence::Yes,
            Method::Post => SettingValence::Yes,
            Method::Put => SettingValence::Yes,
            Method::Trace => SettingValence::Yes,
            Method::Invalid => SettingValence::Optional,
        }
    }

    fn _method_has_response_body(method: &Method) -> bool {
        match method {
            Method::Connect => true,
            Method::Delete => true,
            Method::Get => true,
            Method::Head => false,
            Method::Options => true,
            Method::Patch => true,
            Method::Post => true,
            Method::Put => true,
            Method::Trace => true,
            Method::Invalid => true,
        }
    }

    fn _method_is_safe(method: &Method) -> bool {
        match method {
            Method::Connect => false,
            Method::Delete => false,
            Method::Get => true,
            Method::Head => true,
            Method::Options => true,
            Method::Patch => false,
            Method::Post => false,
            Method::Put => false,
            Method::Trace => true,
            Method::Invalid => true,
        }
    }

    fn _method_is_idempotent(method: &Method) -> bool {
        match method {
            Method::Connect => false,
            Method::Delete => true,
            Method::Get => true,
            Method::Head => true,
            Method::Options => true,
            Method::Patch => false,
            Method::Post => false,
            Method::Put => true,
            Method::Trace => true,
            Method::Invalid => true,
        }
    }

    fn _method_is_cacheable(method: &Method) -> bool {
        match method {
            Method::Connect => false,
            Method::Delete => false,
            Method::Get => true,
            Method::Head => true,
            Method::Options => false,
            Method::Patch => false,
            Method::Post => true,
            Method::Put => false,
            Method::Trace => false,
            Method::Invalid => false,
        }
    }

    fn get_query_args_from_multipart_blob(data: &[u8]) -> Option<(String, MultiPartValue)> {
        let mut headers: HashMap<String, HeaderValueParts> = HashMap::new();
        let mut last_was_carriage_return = false;
        let mut index = 0;
        let mut start = 0;
        for byte in data.iter() {
            if byte == &10 && last_was_carriage_return {
                last_was_carriage_return = false;
                if let Ok(utf8_line) = str::from_utf8(&data[start..index]) {
                    if utf8_line.trim().is_empty() {
                        start = index + 1;
                        break;
                    } else {
                        if let Some((header_key, header_value)) =
                            Message::get_header_field(utf8_line)
                        {
                            headers.insert(header_key, header_value);
                        }
                    }
                    start = index + 1;
                }
            } else if byte == &13 {
                last_was_carriage_return = true;
            } else {
                last_was_carriage_return = false;
            }
            index = index + 1;
        }

        // Did we find a name within the content-disposition header?
        let mut name = String::new();
        if let Some(content_disposition) = headers.get("Content-Disposition") {
            if let Some(content_disposition_name) = content_disposition.get_key_value("name") {
                name = content_disposition_name.trim_matches('"').to_string();
            }
        }
        if !name.is_empty() {
            let body = data[start..].to_vec();
            if !body.is_empty() {
                return Some((name, MultiPartValue { body, headers }));
            }
        }
        None
    }

    fn get_query_args_from_string(subject: &str) -> Option<HashMap<String, String>> {
        let mut args: HashMap<String, String> = HashMap::new();
        if !subject.is_empty() {
            let subject_arguments: Vec<&str> = subject.split("&").collect();
            for item in subject_arguments {
                let query_arg: Vec<&str> = item.split("=").collect();
                if query_arg.len() == 2 {
                    args.insert(query_arg.get(0)?.to_string(), query_arg.get(1)?.to_string());
                } else {
                    args.insert(query_arg.get(0)?.to_string(), String::from("1"));
                }
            }
        }
        if args.len() > 0 {
            return Some(args);
        }
        None
    }

    pub fn get_protocol_text(protocol: &Protocol) -> String {
        match protocol {
            Protocol::V0_9 => String::from("HTTP/0.9"),
            Protocol::V1_0 => String::from("HTTP/1.0"),
            Protocol::V1_1 => String::from("HTTP/1.1"),
            Protocol::V2_0 => String::from("HTTP/2.0"),
            Protocol::Invalid => String::from("INVALID"),
        }
    }

    pub fn get_message_body(body: &str) -> Option<BodyContentType> {
        if let Some(body) = Message::get_query_args_from_string(body) {
            return Some(BodyContentType::SinglePart(body));
        }
        None
    }

    // TODO Maybe change keys to Camel-Case to improve parsing
    pub fn get_header_field(line: &str) -> Option<(String, HeaderValueParts)> {
        let line = line.trim();
        if !line.is_empty() {
            let parts: Vec<&str> = line.splitn(2, ":").collect();
            if parts.len() == 2 {
                let header_key = parts.get(0)?.trim().to_string();
                let header_value = parts.get(1)?.trim().to_string();
                let mut header_parts: Vec<Vec<HeaderValuePart>> = Vec::new();

                let params_blocks: Vec<&str> = header_value.split(";").collect();
                for params_block in params_blocks.iter() {
                    let mut header_value_part: Vec<HeaderValuePart> = Vec::new();
                    let params_subblocks: Vec<&str> = params_block.split(",").collect();
                    for params_subblock in params_subblocks.iter() {
                        let params_subblock_clone = params_subblock.clone();
                        let params_key_pair: Vec<&str> =
                            params_subblock_clone.splitn(2, "=").collect();
                        if params_key_pair.len() == 2 {
                            let param_key = params_key_pair.get(0)?.trim().to_string();
                            let param_value = params_key_pair.get(1)?.trim().to_string();
                            header_value_part
                                .push(HeaderValuePart::KeyValue(param_key, param_value));
                        } else {
                            header_value_part
                                .push(HeaderValuePart::Single(params_subblock.trim().to_string()));
                        }
                    }
                    header_parts.push(header_value_part);
                }

                return Some((
                    header_key,
                    HeaderValueParts {
                        parts: header_parts,
                    },
                ));
            }
        }
        None
    }

    pub fn get_request_line(line: &str) -> Option<Line> {
        let line = line.trim();
        let parts: Vec<&str> = line.split(" ").collect();
        if parts.len() == 3 {
            let method = match parts.get(0)?.as_ref() {
                "CONNECT" => Method::Connect,
                "DELETE" => Method::Delete,
                "GET" => Method::Get,
                "HEAD" => Method::Head,
                "OPTIONS" => Method::Options,
                "PATCH" => Method::Patch,
                "PUT" => Method::Put,
                "POST" => Method::Post,
                "TRACE" => Method::Trace,
                __ => Method::Invalid,
            };

            let request_uri = parts.get(1)?.to_string();
            let request_uri_copy = request_uri.clone();
            let mut request_uri_base = request_uri.clone();
            let mut query_string = String::new();
            let mut query_arguments: HashMap<String, String> = HashMap::new();
            let uri_parts: Vec<&str> = request_uri_copy.splitn(2, "?").collect();
            if uri_parts.len() == 2 {
                request_uri_base = uri_parts.get(0)?.to_string();
                query_string = uri_parts.get(1)?.to_string();
                if let Some(query_args) = Message::get_query_args_from_string(&query_string) {
                    query_arguments = query_args;
                }
            }

            let protocol = match parts.get(2)?.as_ref() {
                "HTTP/0.9" => Protocol::V0_9,
                "HTTP/1.0" => Protocol::V1_0,
                "HTTP/1.1" => Protocol::V1_1,
                "HTTP/2.0" => Protocol::V2_0,
                _ => Protocol::Invalid,
            };

            if method != Method::Invalid && protocol != Protocol::Invalid {
                return Some(Line {
                    method,
                    protocol,
                    request_uri,
                    request_uri_base,
                    query_arguments,
                    query_string,
                });
            }
        } else if parts.len() == 1 {
            // Support for a request line containing only the path name is accepted by servers to
            // maintain compatibility with  clients before the HTTP/1.0 specification.
            let method = Method::Get;
            let request_uri = parts.get(0)?.trim_matches(char::from(0)).to_string();
            if !request_uri.is_empty() {
                let protocol = Protocol::V0_9;

                let request_uri_copy = request_uri.clone();
                let mut request_uri_base = request_uri.clone();
                let mut query_string = String::new();
                let mut query_arguments: HashMap<String, String> = HashMap::new();

                let uri_parts: Vec<&str> = request_uri_copy.splitn(2, "?").collect();
                if uri_parts.len() == 2 {
                    request_uri_base = uri_parts.get(0)?.to_string();
                    query_string = uri_parts.get(1)?.to_string();
                    if let Some(query_args) = Message::get_query_args_from_string(&query_string) {
                        query_arguments = query_args;
                    }
                }

                return Some(Line {
                    method,
                    protocol,
                    request_uri,
                    request_uri_base,
                    query_arguments,
                    query_string,
                });
            }
        }
        None
    }

    /// Try to decode a byte stream into a HTTP Message
    /// ## Usage
    /// ```rust
    /// extern crate milstian_http;
    /// use milstian_http::request::{Message, Method, Protocol};
    /// let response = Message::from_tcp_stream(b"GET / HTTP/2.0\r\n");
    /// let response_unwrapped = response.expect("A decoded HTTP Message");
    /// assert_eq!(response_unwrapped.request_line.method, Method::Get);
    /// assert_eq!(response_unwrapped.request_line.request_uri, "/".to_string());
    /// assert_eq!(response_unwrapped.request_line.protocol, Protocol::V2_0);
    /// ```
    pub fn from_tcp_stream(request: &[u8]) -> Option<Message> {
        // Temporary message
        let mut message = Message {
            body: BodyContentType::SinglePart(HashMap::new()),
            headers: HashMap::new(),
            request_line: Line {
                method: Method::Invalid,
                protocol: Protocol::Invalid,
                request_uri: String::new(),
                request_uri_base: String::new(),
                query_arguments: HashMap::new(),
                query_string: String::new(),
            },
        };

        // Parsing variables
        let mut start = 0;
        let mut start_boundary = 0;
        let mut start_data = 0;
        let mut section = ParserSection::Line;
        let mut end = 0;
        let mut end_data = 0;
        let last_index = match request.len() {
            0 => 0,
            _ => request.len() - 1,
        };
        let mut last_was_carriage_return = false;
        let mut parser_mode = ParserMode::Lines;
        let mut multipart_section = MultiPartSection::Start;

        for byte in request.iter() {
            match parser_mode {
                ParserMode::Boundaries(ref boundary) => {
                    match multipart_section {
                        MultiPartSection::Skipping => {
                            if byte == &13 {
                                last_was_carriage_return = true;
                            } else if byte == &10 && last_was_carriage_return {
                                multipart_section = MultiPartSection::Start;
                                start_boundary = end + 1;
                                last_was_carriage_return = false;
                            } else if byte == &0 {
                                break;
                            } else {
                                last_was_carriage_return = false;
                            }
                        }
                        MultiPartSection::Start => {
                            // Does byte match next byte in boundary?
                            if let Some(boundary_byte) = boundary.get(end - start_boundary) {
                                if boundary_byte == byte {
                                    // Was it the last character of boundary?
                                    if end - start_boundary + 1 == boundary.len() {
                                        multipart_section = MultiPartSection::StartSuffix;
                                    }
                                } else if byte == &45 && start_boundary < end {
                                    if let Some(boundary_byte) =
                                        boundary.get(end - start_boundary - 1)
                                    {
                                        if boundary_byte == byte {
                                            start_boundary = start_boundary + 1;
                                        } else {
                                            multipart_section = MultiPartSection::Skipping;
                                        }
                                    } else {
                                        multipart_section = MultiPartSection::Skipping;
                                    }
                                } else {
                                    multipart_section = MultiPartSection::Skipping;
                                }
                            } else if byte == &0 {
                                break;
                            } else {
                                multipart_section = MultiPartSection::Skipping;
                            }
                        }
                        MultiPartSection::StartSuffix => {
                            if byte == &13 {
                                last_was_carriage_return = true;
                            } else if byte == &10 && last_was_carriage_return {
                                multipart_section = MultiPartSection::End;
                                last_was_carriage_return = false;
                                start_data = end;
                            } else if byte == &0 {
                                break;
                            } else {
                                last_was_carriage_return = false;
                                multipart_section = MultiPartSection::Skipping;
                            }
                        }
                        MultiPartSection::End => {
                            if byte == &13 {
                                last_was_carriage_return = true;
                            } else if byte == &10 && last_was_carriage_return {
                                multipart_section = MultiPartSection::EndBoundary;
                                last_was_carriage_return = false;
                                end_data = end - 1;
                                start_boundary = end + 1;
                            } else if byte == &0 {
                                break;
                            }
                        }
                        MultiPartSection::EndBoundary => {
                            // Does byte match next byte in boundary?
                            if let Some(boundary_byte) = boundary.get(end - start_boundary) {
                                if boundary_byte == byte {
                                    // Was it the last character of boundary?
                                    if end - start_boundary + 1 == boundary.len() {
                                        multipart_section = MultiPartSection::Skipping;

                                        if start_data > 0
                                            && start_data < end_data
                                            && end_data < request.len()
                                        {
                                            let data = &request[start_data..end_data];
                                            if let Some((query_key, query_value)) =
                                                Message::get_query_args_from_multipart_blob(&data)
                                            {
                                                if let BodyContentType::MultiPart(ref mut values) =
                                                    message.body
                                                {
                                                    values.insert(query_key, query_value);
                                                }
                                            }
                                        }
                                    }
                                } else if byte == &45 && start_boundary < end {
                                    if let Some(boundary_byte) =
                                        boundary.get(end - start_boundary - 1)
                                    {
                                        if boundary_byte == byte {
                                            start_boundary = start_boundary + 1;
                                        } else {
                                            multipart_section = MultiPartSection::End;
                                        }
                                    } else {
                                        multipart_section = MultiPartSection::End;
                                    }
                                } else {
                                    multipart_section = MultiPartSection::End;
                                }
                            } else if byte == &0 {
                                break;
                            } else {
                                multipart_section = MultiPartSection::End;
                            }
                        }
                    }
                }
                ParserMode::Lines => {
                    if byte == &13 {
                        last_was_carriage_return = true;
                    } else if byte == &10 && last_was_carriage_return {
                        let clean_end = end - 1;
                        if let Ok(utf8_line) = str::from_utf8(&request[start..clean_end]) {
                            Message::parse_line(
                                &utf8_line,
                                &mut section,
                                &mut message,
                                &mut parser_mode,
                            );
                            start = end + 1;
                            start_boundary = end + 1;
                        }
                        last_was_carriage_return = false;

                    // When we get null bytes we are done or if we reach last index
                    } else if byte == &0 || end == last_index {
                        let clean_end = match byte {
                            &0 => end,
                            _ => end + 1,
                        };
                        if let Ok(utf8_line) = str::from_utf8(&request[start..clean_end]) {
                            Message::parse_line(
                                &utf8_line,
                                &mut section,
                                &mut message,
                                &mut parser_mode,
                            );
                        }
                        break;
                    } else {
                        last_was_carriage_return = false;
                    }
                }
            }

            // Increment byte position
            end = end + 1;
        }

        if message.request_line.method != Method::Invalid
            && message.request_line.protocol != Protocol::Invalid
        {
            return Some(message);
        }

        None
    }

    fn parse_line(
        line: &str,
        section: &mut ParserSection,
        message: &mut Message,
        parser_mode: &mut ParserMode,
    ) {
        match section {
            ParserSection::Line => {
                if let Some(request_line_temp) = Message::get_request_line(line) {
                    message.request_line = request_line_temp;
                    *section = ParserSection::HeaderFields;
                }
            }
            ParserSection::HeaderFields => {
                // Is it the last line of the headers?
                if line.trim().is_empty() {
                    // Check if we have a multi-part body
                    if let Some(content_type_header) = message.headers.get("Content-Type") {
                        if let Some(boundary) = content_type_header.get_key_value("boundary") {
                            *parser_mode = ParserMode::Boundaries(boundary.as_bytes().to_vec());
                            message.body = BodyContentType::MultiPart(HashMap::new());
                        }
                    }

                    if Message::method_has_request_body(&message.request_line.method)
                        != SettingValence::No
                    {
                        *section = ParserSection::MessageBody;
                    }
                } else {
                    if let Some((header_key, header_value)) = Message::get_header_field(line) {
                        message.headers.insert(header_key, header_value);
                    }
                }
            }
            ParserSection::MessageBody => {
                if !line.is_empty() {
                    if let Some(body_args) = Message::get_message_body(line) {
                        message.body = body_args;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_message_body_single_part() {
        let response = Message::get_message_body("random=abc&hej=def&def");
        assert!(response.is_some());

        let response_unwrapped = response.unwrap();
        if let BodyContentType::SinglePart(response_unwrapped) = response_unwrapped {
            assert_eq!(
                response_unwrapped
                    .get(&"random".to_string())
                    .unwrap()
                    .to_string(),
                "abc".to_string()
            );
            assert_eq!(
                response_unwrapped
                    .get(&"hej".to_string())
                    .unwrap()
                    .to_string(),
                "def".to_string()
            );
            assert_eq!(
                response_unwrapped
                    .get(&"def".to_string())
                    .unwrap()
                    .to_string(),
                "1".to_string()
            );
            assert!(response_unwrapped.get(&"defs".to_string()).is_none());
        }

        let response = Message::get_message_body("");
        assert!(response.is_none());
    }

    #[test]
    fn test_get_query_args_from_multipart_blob() {
        let response = Message::get_query_args_from_multipart_blob(
            b"Content-Disposition: form-data; name=\"losen\"\r\n\r\nabc\n123",
        );
        assert!(response.is_some());
        if let Some((query_key, query_value)) = response {
            assert_eq!(query_key, "losen".to_string());
            assert_eq!(
                query_value
                    .headers
                    .get("Content-Disposition")
                    .unwrap()
                    .to_string(),
                "form-data; name=\"losen\""
            );
            assert_eq!(query_value.body, b"abc\n123");
        } else {
            panic!("Expected multipart body but received: {:?}", response);
        }

        let response = Message::get_query_args_from_multipart_blob(
            b"Content-Disposition: form-data; name=\"file\"; filename=\"KeePassXC-2.3.1.dmg.sig\"\r\nContent-Type: application/octet-stream\r\n\r\n
-----BEGIN PGP SIGNATURE-----

iQEzBAABCAAdFiEEweTLo61406/YlPngt6ZvA7WQdqgFAlqfE5MACgkQt6ZvA7WQ
dqgnEAgAjtdbsMPaULGXKX6H+fcsYeGEN8OjiUTNz+StwNDkDxhxB4MT0N0lYZ4L
xUv86kwMdWAaxp8pvVWo6gWXTEM5gWmN302bBxkpbhBl9fnq6WdcCCDGs4GM5vHX
lOrHXWTsK+8ayLNZ0dCcP054srAtMmJHscPiuUYPfvKSgLxl+JxkPC147EktCCzv
5O+2AtQPwIEPuaMewFqP9KjaGOhWgAc0nauIKa0ASt9FXXrexq1EoZnoZ3ZQ0p/w
/otAB2D27yQ4kv+X2Rn94Ky9W0lMT2MYEF+/tQH4aEKsdMBQ7REQtfLGFlEzTMB/
BNUI5YCF3PV9MKr3N53vEVYvkbXLbw==
=LO1E
-----END PGP SIGNATURE-----
");

        assert!(response.is_some());
        if let Some((query_key, query_value)) = response {
            assert_eq!(query_key, "file".to_string());
            assert_eq!(
                query_value
                    .headers
                    .get("Content-Disposition")
                    .unwrap()
                    .to_string(),
                "form-data; name=\"file\"; filename=\"KeePassXC-2.3.1.dmg.sig\""
            );
        } else {
            panic!("Expected multipart body but received: {:?}", response);
        }

        let response = Message::get_query_args_from_multipart_blob(
            b"okasdokadsokasd oa skoasdk\r\nokadsokasdokoadskods\r\n123123",
        );
        assert!(response.is_none());
    }

    #[test]
    fn test_get_header_field() {
        let response = Message::get_header_field(
            "User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:12.0) Gecko/20100101 Firefox/12.0\r\n",
        );
        assert!(response.is_some());

        let (key, value) = response.unwrap();
        assert_eq!(key, "User-Agent".to_string());
        assert_eq!(
            value.to_string(),
            "Mozilla/5.0 (X11; Linux x86_64; rv:12.0) Gecko/20100101 Firefox/12.0".to_string()
        );

        let response = Message::get_header_field("Cache-Control: no-cache \r\n");
        assert!(response.is_some());

        let (key, value) = response.unwrap();
        assert_eq!(key, "Cache-Control".to_string());
        assert_eq!(value.to_string(), "no-cache".to_string());

        let response = Message::get_header_field("Just various text here\r\n");
        assert!(response.is_none());

        let response = Message::get_header_field("");
        assert!(response.is_none());

        let response = Message::get_header_field(
            "Content-Type: multipart/form-data; boundary=---------------------------208201381313076108731815782760\r\n",
        );
        assert!(response.is_some());
        let (key, value) = response.unwrap();
        assert_eq!(key, "Content-Type".to_string());
        assert_eq!(value.to_string(), "multipart/form-data; boundary=---------------------------208201381313076108731815782760".to_string());
        assert_eq!(
            value.get_key_value("boundary").unwrap(),
            "---------------------------208201381313076108731815782760".to_string()
        );
    }

    #[test]
    fn test_get_request_line() {
        let response = Message::get_request_line("POST /random?abc=test HTTP/0.9\r\n");
        assert!(response.is_some());

        let response_unpacked = response.unwrap();
        assert_eq!(response_unpacked.method, Method::Post);
        assert_eq!(
            response_unpacked.request_uri,
            String::from("/random?abc=test")
        );
        assert_eq!(response_unpacked.request_uri_base, String::from("/random"));
        assert_eq!(response_unpacked.query_string, String::from("abc=test"));
        assert_eq!(
            response_unpacked
                .query_arguments
                .get(&"abc".to_string())
                .unwrap()
                .to_string(),
            String::from("test")
        );
        assert_eq!(response_unpacked.protocol, Protocol::V0_9);

        let response = Message::get_request_line("GET / HTTP/1.0\r\n");
        assert!(response.is_some());

        let response_unpacked = response.unwrap();
        assert_eq!(response_unpacked.method, Method::Get);
        assert_eq!(response_unpacked.request_uri, String::from("/"));
        assert_eq!(response_unpacked.request_uri_base, String::from("/"));
        assert_eq!(response_unpacked.query_string, String::from(""));
        assert_eq!(response_unpacked.protocol, Protocol::V1_0);

        let response = Message::get_request_line("HEAD /moradish.html?test&abc=def HTTP/1.1\r\n");
        assert!(response.is_some());

        let response_unpacked = response.unwrap();
        assert_eq!(response_unpacked.method, Method::Head);
        assert_eq!(
            response_unpacked.request_uri,
            String::from("/moradish.html?test&abc=def")
        );
        assert_eq!(
            response_unpacked.request_uri_base,
            String::from("/moradish.html")
        );
        assert_eq!(response_unpacked.query_string, String::from("test&abc=def"));
        assert_eq!(
            response_unpacked
                .query_arguments
                .get(&"test".to_string())
                .unwrap()
                .to_string(),
            String::from("1")
        );
        assert_eq!(
            response_unpacked
                .query_arguments
                .get(&"abc".to_string())
                .unwrap()
                .to_string(),
            String::from("def")
        );
        assert_eq!(response_unpacked.protocol, Protocol::V1_1);

        let response = Message::get_request_line("OPTIONS /random/random2.txt HTTP/2.0\r\n");
        assert!(response.is_some());

        let response_unpacked = response.unwrap();
        assert_eq!(response_unpacked.method, Method::Options);
        assert_eq!(
            response_unpacked.request_uri,
            String::from("/random/random2.txt")
        );
        assert_eq!(response_unpacked.protocol, Protocol::V2_0);

        let response = Message::get_request_line("GET / HTTP/2.2\r\n");
        assert!(response.is_none());
    }

    #[test]
    fn test_from_tcp_stream() {
        // GET request with no headers or body
        let response = Message::from_tcp_stream(b"GET / HTTP/2.0\r\n");
        assert!(response.is_some());
        let response_unwrapped = response.expect("GET HTTP2");
        assert_eq!(response_unwrapped.request_line.method, Method::Get);
        assert_eq!(response_unwrapped.request_line.request_uri, "/".to_string());
        assert_eq!(response_unwrapped.request_line.protocol, Protocol::V2_0);

        // POST request with random header and null bytes
        let mut request: Vec<u8> =
            b"POST /random HTTP/1.0\r\nAgent: Random browser\r\n\r\ntest=abc".to_vec();
        request.push(0);
        request.push(0);
        let response = Message::from_tcp_stream(&request);
        assert!(response.is_some());
        assert_eq!(
            "/random".to_string(),
            response.expect("/random").request_line.request_uri
        );

        // POST request with random header
        let response =
            Message::from_tcp_stream(b"POST / HTTP/1.0\r\nAgent: Random browser\r\n\r\ntest=abc");
        assert!(response.is_some());
        let response_unwrapped = response.expect("POST HTTP1");
        assert_eq!(response_unwrapped.request_line.method, Method::Post);
        assert_eq!(response_unwrapped.request_line.protocol, Protocol::V1_0);
        assert_eq!(
            response_unwrapped
                .headers
                .get(&"Agent".to_string())
                .expect("Agent")
                .to_string(),
            "Random browser".to_string()
        );
        if let BodyContentType::SinglePart(body) = response_unwrapped.body {
            assert_eq!(
                body.get(&"test".to_string()).expect("test-abc").to_string(),
                "abc".to_string()
            );
        }

        // Two invalid  requests
        let response = Message::from_tcp_stream(b"RANDOM /stuff HTTP/2.5\r\n");
        assert!(response.is_none());
        let response = Message::from_tcp_stream(b"");
        assert!(response.is_none());

        // Multi-part with form-data
        let response = Message::from_tcp_stream(b"POST /?test=abcdef HTTP/1.1\r\nHost: localhost:8888\r\nUser-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.13; rv:63.0) Gecko/20100101 Firefox/63.0\r\nAccept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\nAccept-Language: en-US,en;q=0.5\r\nAccept-Encoding: gzip, deflate\r\nReferer: http://localhost:8888/?test=abcdef\r\nContent-Type: multipart/form-data; boundary=---------------------------5072966556248019951999579782\r\nContent-Length: 733\r\nDNT: 1\r\nConnection: keep-alive\r\nUpgrade-Insecure-Requests: 1\r\nPragma: no-cache\r\nCache-Control: no-cache\r\n\r\n-----------------------------5072966556248019951999579782\r\nContent-Disposition: form-data; name=\"file\"; filename=\"KeePassXC-2.3.1.dmg.sig\"\r\nContent-Type: application/octet-stream\r\n\r\n-----BEGIN PGP SIGNATURE-----\n\niQEzBAABCAAdFiEEweTLo61406/YlPngt6ZvA7WQdqgFAlqfE5MACgkQt6ZvA7WQ\ndqgnEAgAjtdbsMPaULGXKX6H+fcsYeGEN8OjiUTNz+StwNDkDxhxB4MT0N0lYZ4L\nxUv86kwMdWAaxp8pvVWo6gWXTEM5gWmN302bBxkpbhBl9fnq6WdcCCDGs4GM5vHX\nlOrHXWTsK+8ayLNZ0dCcP054srAtMmJHscPiuUYPfvKSgLxl+JxkPC147EktCCzv\n5O+2AtQPwIEPuaMewFqP9KjaGOhWgAc0nauIKa0ASt9FXXrexq1EoZnoZ3ZQ0p/w\n/otAB2D27yQ4kv+X2Rn94Ky9W0lMT2MYEF+/tQH4aEKsdMBQ7REQtfLGFlEzTMB/\nBNUI5YCF3PV9MKr3N53vEVYvkbXLbw==\n=LO1E\n-----END PGP SIGNATURE-----\n\r\n-----------------------------5072966556248019951999579782--\r\nCAAdFiEEweTLo61406/YlPngt6ZvA7WQdqgFAlqfE5MACgkQt6ZvA7WQ\ndqgnEAgAjtdbsMPaULGXKX6H+fcsYeGEN8OjiUTNz+StwNDkDxhxB4MT0N0lYZ4L\nxUv86kwMdWAaxp8pvVWo6gWXTEM5gWmN302bBxkpbhBl9fnq6WdcCCDGs4GM5vHX\nlOrHXWTsK+8ayLNZ0dCcP054srAtMmJHscPiuUYPfvKSgLxl+JxkPC147");
        assert!(response.is_some());
        let response_unwrapped = response.expect("multipart");
        if let BodyContentType::MultiPart(body) = response_unwrapped.body {
            assert_eq!(
                String::from_utf8(body.get(&"file".to_string()).expect("expecting file data").body.clone()).expect("expecting utf-8 file data"),
                "-----BEGIN PGP SIGNATURE-----\n\niQEzBAABCAAdFiEEweTLo61406/YlPngt6ZvA7WQdqgFAlqfE5MACgkQt6ZvA7WQ\ndqgnEAgAjtdbsMPaULGXKX6H+fcsYeGEN8OjiUTNz+StwNDkDxhxB4MT0N0lYZ4L\nxUv86kwMdWAaxp8pvVWo6gWXTEM5gWmN302bBxkpbhBl9fnq6WdcCCDGs4GM5vHX\nlOrHXWTsK+8ayLNZ0dCcP054srAtMmJHscPiuUYPfvKSgLxl+JxkPC147EktCCzv\n5O+2AtQPwIEPuaMewFqP9KjaGOhWgAc0nauIKa0ASt9FXXrexq1EoZnoZ3ZQ0p/w\n/otAB2D27yQ4kv+X2Rn94Ky9W0lMT2MYEF+/tQH4aEKsdMBQ7REQtfLGFlEzTMB/\nBNUI5YCF3PV9MKr3N53vEVYvkbXLbw==\n=LO1E\n-----END PGP SIGNATURE-----\n".to_string()
            );
        } else {
            eprintln!(
                "Boundary header: {:?}",
                response_unwrapped
                    .headers
                    .get("Content-Type")
                    .expect("A content-type header")
                    .get_key_value("boundary")
                    .expect("A boundary")
            );
            panic!(
                "Expected multipart content but got: {:?}",
                response_unwrapped
            );
        }

        // Get requests should get their message body parsed
        let response = Message::from_tcp_stream(b"GET / HTTP/2.0\r\n\r\nabc=123");
        assert!(response.is_some());
        let response_unwrapped = response.unwrap();
        if let BodyContentType::SinglePart(body) = response_unwrapped.body {
            assert_eq!(
                body.get(&"abc".to_string()).unwrap().to_string(),
                "123".to_string()
            );
        }

        // HEAD requests should not get their message body parsed
        let response = Message::from_tcp_stream(b"HEAD / HTTP/2.0\r\n\r\nabc=123");
        assert!(response.is_some());
        let response_unwrapped = response.unwrap();
        if let BodyContentType::SinglePart(body) = response_unwrapped.body {
            assert!(body.get(&"abc".to_string()).is_none());
        }

        let response = Message::from_tcp_stream(b"html/index.html\r\n");
        assert!(response.is_some());
        let response_unwrapped = response.unwrap();
        assert_eq!(response_unwrapped.request_line.method, Method::Get);
        assert_eq!(
            response_unwrapped.request_line.request_uri,
            "html/index.html".to_string()
        );
        assert_eq!(response_unwrapped.request_line.protocol, Protocol::V0_9);

        let response = Message::from_tcp_stream(&[0; 100]);
        assert!(response.is_none());
    }

}

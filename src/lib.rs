//! # Milstian HTTP
//!
//! ![Milstian Logo](https://raw.githubusercontent.com/cjohansson/milstian-rust-internet-framework/master/html/img/logo1-modified.jpg)
//!
//! In progress, primarily used for learning Rust programming.
//!
//! This project is used by the milstian-internet-framework to parse and build HTTP requests and responses.
//!
//! ## Major goal
//! * Easy and fast way to decode and encode HTTP requests and responses
//!
//! ## Usage
//! ### Include in project
//! This crate is on [crates.io](https://crates.io/crates/milstian-http) and can be used by adding time to the dependencies in your project's `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! milstian_http = "0.1.*"
//! ```
//! And this in your crate root:
//! ```rust,dont_run
//! extern crate milstian_http;
//! ```
//!
//! ### Decoding a TCP stream into a HTTP request
//! ```rust
//! extern crate milstian_http;
//! use milstian_http::request::{Message, Method, Protocol};
//! 
//! let request =
//!     Message::from_tcp_stream(b"POST / HTTP/1.0\r\nAgent: Random browser\r\n\r\ntest=abc");
//! assert!(request.is_some());
//! let request_unwrapped = request.expect("POST HTTP1");
//! assert_eq!(request_unwrapped.request_line.method, Method::Post);
//! assert_eq!(request_unwrapped.request_line.protocol, Protocol::V1_0);
//! ```
//!
//! ### Encoding protocol, status, headers and body into a HTTP response
//! ```rust
//! extern crate milstian_http;
//! use milstian_http::response::Message;
//! use std::collections::HashMap;
//! assert_eq!(
//!     Message::new(
//!         "HTTP/1.0".to_string(),
//!         "200 OK".to_string(),
//!         HashMap::new(),
//!         b"<html><body>Nothing here</body></html>".to_vec()
//!     ).to_bytes(),
//!     b"HTTP/1.0 200 OK\r\n\r\n<html><body>Nothing here</body></html>".to_vec()
//! );
//! ```

pub mod request;
pub mod response;

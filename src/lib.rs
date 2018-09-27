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

// TODO Code usage example here

pub mod request;
pub mod response;

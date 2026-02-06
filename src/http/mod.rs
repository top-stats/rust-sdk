//! HTTP client abstraction layer.
//!
//! This module provides a trait-based abstraction over HTTP clients,
//! allowing users to choose between different backends (reqwest, ureq).

use std::collections::HashMap;

use crate::error::Result;

mod traits;

#[cfg(not(feature = "blocking"))]
pub use traits::HttpClient;

#[cfg(feature = "blocking")]
pub use traits::BlockingHttpClient;

#[cfg(all(feature = "reqwest-client", not(feature = "blocking")))]
mod reqwest_client;

#[cfg(all(feature = "reqwest-client", not(feature = "blocking")))]
pub use reqwest_client::ReqwestClient;

#[cfg(all(feature = "ureq-client", feature = "blocking"))]
mod ureq_client;

#[cfg(all(feature = "ureq-client", feature = "blocking"))]
pub use ureq_client::UreqClient;

/// HTTP request method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// HTTP GET method.
    Get,
    /// HTTP POST method.
    Post,
}

/// An HTTP request.
#[derive(Debug, Clone)]
pub struct Request {
    /// The HTTP method.
    pub method: Method,
    /// The full URL.
    pub url: String,
    /// Query parameters.
    pub query: HashMap<String, String>,
    /// Request headers.
    pub headers: HashMap<String, String>,
    /// Request body (for POST requests).
    pub body: Option<String>,
}

impl Request {
    /// Creates a new GET request.
    #[must_use]
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            method: Method::Get,
            url: url.into(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Creates a new POST request.
    #[must_use]
    pub fn post(url: impl Into<String>) -> Self {
        Self {
            method: Method::Post,
            url: url.into(),
            query: HashMap::new(),
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Adds a query parameter.
    #[must_use]
    pub fn query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.insert(key.into(), value.into());
        self
    }

    /// Adds a header.
    #[must_use]
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Sets the request body.
    #[must_use]
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }
}

/// An HTTP response.
#[derive(Debug, Clone)]
pub struct Response {
    /// The HTTP status code.
    pub status: u16,
    /// Response headers.
    pub headers: HashMap<String, String>,
    /// Response body.
    pub body: String,
}

impl Response {
    /// Returns `true` if the response status is successful (2xx).
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// Returns `true` if the response status indicates a rate limit (429).
    #[must_use]
    pub const fn is_rate_limited(&self) -> bool {
        self.status == 429
    }

    /// Returns `true` if the response status indicates not found (404).
    #[must_use]
    pub const fn is_not_found(&self) -> bool {
        self.status == 404
    }

    /// Parses the response body as JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if the body cannot be parsed as the expected type.
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        Ok(serde_json::from_str(&self.body)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_builder() {
        let request = Request::get("https://api.example.com/test")
            .query("foo", "bar")
            .query("baz", "qux")
            .header("Authorization", "token123");

        assert_eq!(request.method, Method::Get);
        assert_eq!(request.url, "https://api.example.com/test");
        assert_eq!(request.query.get("foo"), Some(&"bar".to_string()));
        assert_eq!(request.query.get("baz"), Some(&"qux".to_string()));
        assert_eq!(
            request.headers.get("Authorization"),
            Some(&"token123".to_string())
        );
    }

    #[test]
    fn test_response_status_checks() {
        let success = Response {
            status: 200,
            headers: HashMap::new(),
            body: String::new(),
        };
        assert!(success.is_success());
        assert!(!success.is_rate_limited());
        assert!(!success.is_not_found());

        let rate_limited = Response {
            status: 429,
            headers: HashMap::new(),
            body: String::new(),
        };
        assert!(!rate_limited.is_success());
        assert!(rate_limited.is_rate_limited());

        let not_found = Response {
            status: 404,
            headers: HashMap::new(),
            body: String::new(),
        };
        assert!(!not_found.is_success());
        assert!(not_found.is_not_found());
    }

    #[test]
    fn test_response_json_parsing() {
        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct TestData {
            name: String,
            value: i32,
        }

        let response = Response {
            status: 200,
            headers: HashMap::new(),
            body: r#"{"name": "test", "value": 42}"#.to_string(),
        };

        let data: TestData = response.json().unwrap();
        assert_eq!(data.name, "test");
        assert_eq!(data.value, 42);
    }
}

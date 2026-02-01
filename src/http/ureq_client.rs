//! Ureq HTTP client implementation (blocking only).

use std::collections::HashMap;

use super::{BlockingHttpClient, Method, Request, Response};
use crate::error::{Error, Result};
use crate::user_agent;

/// Blocking HTTP client implementation using ureq.
#[derive(Debug, Clone)]
pub struct UreqClient {
    agent: ureq::Agent,
}

impl UreqClient {
    /// Creates a new ureq client with default settings.
    ///
    /// # Errors
    ///
    /// This function is infallible but returns `Result` for API consistency
    /// with other HTTP client implementations.
    #[allow(clippy::unnecessary_wraps)]
    pub fn new() -> Result<Self> {
        // Configure agent to not treat HTTP errors as Rust errors
        // so we can handle them ourselves
        let config = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .build();
        let agent = ureq::Agent::new_with_config(config);
        Ok(Self { agent })
    }

    /// Creates a new ureq client with a custom agent.
    #[must_use]
    pub const fn with_agent(agent: ureq::Agent) -> Self {
        Self { agent }
    }
}

impl Default for UreqClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default ureq client")
    }
}

impl BlockingHttpClient for UreqClient {
    fn send(&self, request: Request) -> Result<Response> {
        let mut url = request.url.clone();

        // Add query parameters
        if !request.query.is_empty() {
            let query_string: String = request
                .query
                .iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                .collect::<Vec<_>>()
                .join("&");

            if url.contains('?') {
                url.push('&');
            } else {
                url.push('?');
            }
            url.push_str(&query_string);
        }

        // Build the request based on method
        let result = match request.method {
            Method::Get => {
                let mut builder = self.agent.get(&url);

                // Add headers
                for (key, value) in &request.headers {
                    builder = builder.header(key.as_str(), value.as_str());
                }
                builder = builder.header("User-Agent", &user_agent());

                builder.call()
            }
            Method::Post => {
                let mut builder = self.agent.post(&url);

                // Add headers
                for (key, value) in &request.headers {
                    builder = builder.header(key.as_str(), value.as_str());
                }
                builder = builder.header("User-Agent", &user_agent());

                if let Some(body) = request.body {
                    builder.send(body.as_bytes())
                } else {
                    builder.send(&[] as &[u8])
                }
            }
        };

        match result {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let headers = extract_headers(&resp);
                let body = resp
                    .into_body()
                    .read_to_string()
                    .map_err(|e| Error::Network(e.to_string()))?;

                Ok(Response {
                    status,
                    headers,
                    body,
                })
            }
            Err(e) => Err(Error::Network(e.to_string())),
        }
    }
}

fn extract_headers(resp: &ureq::http::Response<ureq::Body>) -> HashMap<String, String> {
    resp.headers()
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|v| (name.as_str().to_string(), v.to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ureq_client_creation() {
        let client = UreqClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_ureq_client_clone() {
        let client = UreqClient::new().unwrap();
        let cloned = client.clone();
        let _ = cloned;
    }
}

//! Ureq HTTP client implementation (blocking only).

use std::collections::HashMap;

use super::{BlockingHttpClient, Method, Request, Response};
use crate::error::{Error, Result};
use crate::user_agent;

/// Blocking HTTP client implementation using ureq.
#[derive(Debug)]
pub struct UreqClient {
    agent: ureq::Agent,
}

impl UreqClient {
    /// Creates a new ureq client with default settings.
    #[must_use]
    pub fn new() -> Self {
        let agent = ureq::AgentBuilder::new().user_agent(&user_agent()).build();

        Self { agent }
    }

    /// Creates a new ureq client with a custom agent.
    #[must_use]
    pub const fn with_agent(agent: ureq::Agent) -> Self {
        Self { agent }
    }
}

impl Default for UreqClient {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for UreqClient {
    fn clone(&self) -> Self {
        // ureq::Agent doesn't implement Clone, so we create a new one
        Self::new()
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

        let mut req = match request.method {
            Method::Get => self.agent.get(&url),
            Method::Post => self.agent.post(&url),
        };

        // Add headers
        for (key, value) in &request.headers {
            req = req.set(key, value);
        }

        // Send request
        let response = if let Some(body) = request.body {
            req.send_string(&body)
        } else {
            req.call()
        };

        match response {
            Ok(resp) => {
                let status = resp.status();
                let headers = extract_headers(&resp);
                let body = resp
                    .into_string()
                    .map_err(|e| Error::Network(e.to_string()))?;

                Ok(Response {
                    status,
                    headers,
                    body,
                })
            }
            Err(ureq::Error::Status(status, resp)) => {
                let headers = extract_headers(&resp);
                let body = resp
                    .into_string()
                    .map_err(|e| Error::Network(e.to_string()))?;

                Ok(Response {
                    status,
                    headers,
                    body,
                })
            }
            Err(e) => Err(Error::Ureq(Box::new(e))),
        }
    }
}

fn extract_headers(resp: &ureq::Response) -> HashMap<String, String> {
    resp.headers_names()
        .iter()
        .filter_map(|name| {
            resp.header(name)
                .map(|value| (name.to_string(), value.to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ureq_client_creation() {
        let client = UreqClient::new();
        let _ = client; // Just verify it doesn't panic
    }

    #[test]
    fn test_ureq_client_clone() {
        let client = UreqClient::new();
        let cloned = client.clone();
        let _ = cloned;
    }
}

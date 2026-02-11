//! Reqwest HTTP client implementation.

use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

use super::{HttpClient, Method, Request, Response};
use crate::error::Result;
use crate::USER_AGENT;

/// HTTP client implementation using reqwest.
#[derive(Debug, Clone)]
pub struct ReqwestClient {
    client: Client,
}

impl ReqwestClient {
    /// Creates a new reqwest client with default settings.
    ///
    /// # Errors
    ///
    /// Returns an error if the client cannot be created.
    pub fn new() -> Result<Self> {
        let client = Client::builder().user_agent(USER_AGENT).build()?;

        Ok(Self { client })
    }

    /// Creates a new reqwest client wrapping an existing `reqwest::Client`.
    pub const fn with_client(client: Client) -> Self {
        Self { client }
    }
}

impl Default for ReqwestClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default reqwest client")
    }
}

#[async_trait]
impl HttpClient for ReqwestClient {
    async fn send(&self, request: Request) -> Result<Response> {
        let mut builder = match request.method {
            Method::Get => self.client.get(&request.url),
            Method::Post => self.client.post(&request.url),
        };

        // Add query parameters
        if !request.query.is_empty() {
            builder = builder.query(&request.query.iter().collect::<Vec<(&String, &String)>>());
        }

        // Add headers
        for (key, value) in &request.headers {
            builder = builder.header(key, value);
        }

        // Add body if present
        if let Some(body) = request.body {
            builder = builder.body(body);
        }

        let response = builder.send().await?;
        let status = response.status().as_u16();

        // Extract headers
        let headers: HashMap<String, String> = response
            .headers()
            .iter()
            .filter_map(|(k, v)| {
                v.to_str()
                    .ok()
                    .map(|v| (k.as_str().to_string(), v.to_string()))
            })
            .collect();

        let body = response.text().await?;

        Ok(Response {
            status,
            headers,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reqwest_client_creation() {
        let client = ReqwestClient::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_reqwest_client_default() {
        let client = ReqwestClient::default();
        // Just verify it doesn't panic
        let _ = client;
    }
}

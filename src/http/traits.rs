//! HTTP client traits.
//!
//! This module defines the traits that HTTP client implementations must implement.

#[cfg(not(feature = "blocking"))]
use async_trait::async_trait;

use super::{Request, Response};
use crate::error::Result;

/// Trait for async HTTP clients.
#[cfg(not(feature = "blocking"))]
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// Sends an HTTP request and returns the response.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    async fn send(&self, request: Request) -> Result<Response>;
}

/// Trait for blocking HTTP clients.
#[cfg(feature = "blocking")]
#[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
pub trait BlockingHttpClient: Send + Sync {
    /// Sends an HTTP request and returns the response.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    fn send(&self, request: Request) -> Result<Response>;
}

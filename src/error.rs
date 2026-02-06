//! Error types for the `TopStats` SDK.

use thiserror::Error;

/// A specialized Result type for `TopStats` operations.
pub type Result<T> = std::result::Result<T, Error>;

/// The main error type for the `TopStats` SDK.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// The API returned a rate limit response.
    /// Contains the number of seconds until the rate limit expires.
    #[error("Rate limited. Retry after {retry_after} seconds")]
    RateLimited {
        /// Seconds until the rate limit expires.
        retry_after: f64,
        /// The error message from the API.
        message: String,
    },

    /// The requested resource was not found.
    #[error("Not found: {message}")]
    NotFound {
        /// The error message from the API.
        message: String,
    },

    /// The request was forbidden (e.g., banned from the API).
    #[error("Forbidden: {message}")]
    Forbidden {
        /// The error message from the API.
        message: String,
    },

    /// An HTTP error occurred.
    #[error("HTTP error {status}: {message}")]
    Http {
        /// The HTTP status code.
        status: u16,
        /// The error message.
        message: String,
    },

    /// A network or connection error occurred.
    #[error("Network error: {0}")]
    Network(String),

    /// Failed to parse the API response.
    #[error("Failed to parse response: {0}")]
    Parse(#[from] serde_json::Error),

    /// Invalid configuration or input.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Invalid Discord bot ID format.
    #[error("Invalid Discord bot ID format: {0}. Expected 17-19 digit snowflake.")]
    InvalidBotId(String),

    /// The API token is missing or invalid.
    #[error("Missing or invalid API token")]
    InvalidToken,

    /// An error from the reqwest HTTP client.
    #[cfg(feature = "reqwest-client")]
    #[cfg_attr(docsrs, doc(cfg(feature = "reqwest-client")))]
    #[error("Request error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// An error from the ureq HTTP client.
    #[cfg(feature = "ureq-client")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ureq-client")))]
    #[error("Request error: {0}")]
    Ureq(#[from] Box<ureq::Error>),
}

impl Error {
    /// Returns `true` if this error is a rate limit error.
    #[must_use]
    pub const fn is_rate_limited(&self) -> bool {
        matches!(self, Self::RateLimited { .. })
    }

    /// Returns `true` if this error is a not found error.
    #[must_use]
    pub const fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound { .. })
    }

    /// Returns the retry-after duration in seconds if this is a rate limit error.
    #[must_use]
    pub const fn retry_after(&self) -> Option<f64> {
        match self {
            Self::RateLimited { retry_after, .. } => Some(*retry_after),
            _ => None,
        }
    }
}

/// API error response structure from the `TopStats` API.
#[derive(Debug, serde::Deserialize)]
pub(crate) struct ApiErrorResponse {
    pub code: u16,
    pub message: String,
    #[serde(rename = "expiresIn")]
    pub expires_in: Option<f64>,
}

impl From<ApiErrorResponse> for Error {
    fn from(response: ApiErrorResponse) -> Self {
        match response.code {
            429 => Self::RateLimited {
                retry_after: response.expires_in.unwrap_or(0.0),
                message: response.message,
            },
            404 => Self::NotFound {
                message: response.message,
            },
            403 => Self::Forbidden {
                message: response.message,
            },
            status => Self::Http {
                status,
                message: response.message,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_error() {
        let err = Error::RateLimited {
            retry_after: 26.0,
            message: "You are being rate limited".to_string(),
        };
        assert!(err.is_rate_limited());
        assert_eq!(err.retry_after(), Some(26.0));
        assert!(err.to_string().contains("26"));
    }

    #[test]
    fn test_not_found_error() {
        let err = Error::NotFound {
            message: "Bot not found".to_string(),
        };
        assert!(err.is_not_found());
        assert!(!err.is_rate_limited());
        assert_eq!(err.retry_after(), None);
    }

    #[test]
    fn test_api_error_response_parsing() {
        let json = r#"{"code": 429, "message": "Rate limited", "expiresIn": 30}"#;
        let response: ApiErrorResponse = serde_json::from_str(json).unwrap();
        let err: Error = response.into();
        assert!(err.is_rate_limited());
        assert_eq!(err.retry_after(), Some(30.0));
    }

    #[test]
    fn test_api_error_404() {
        let json = r#"{"code": 404, "message": "Not found"}"#;
        let response: ApiErrorResponse = serde_json::from_str(json).unwrap();
        let err: Error = response.into();
        assert!(err.is_not_found());
    }

    #[test]
    fn test_invalid_bot_id_error() {
        let err = Error::InvalidBotId("abc".to_string());
        assert!(err.to_string().contains("abc"));
        assert!(err.to_string().contains("17-19 digit"));
    }
}

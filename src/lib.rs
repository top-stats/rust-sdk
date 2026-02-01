//! # `TopStats` Rust SDK
//!
//! A Rust SDK for interacting with the [TopStats.gg API](https://topstats.gg),
//! which provides statistics for Discord bots listed on Top.gg.
//!
//! ## Features
//!
//! - **Async-first** design with runtime-agnostic implementation
//! - **Multiple HTTP backends**: reqwest (default) or ureq
//! - **Blocking API** available via the `blocking` feature
//! - **Built-in rate limiting** with automatic retry for short delays
//! - **Type-safe** models with serde serialization
//! - **Tracing** support for logging (optional)
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use topstats::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), topstats::Error> {
//!     let client = Client::new("your-api-token")?;
//!     
//!     // Get bot information
//!     let bot = client.get_bot("432610292342587392").await?;
//!     println!("Bot: {} has {} monthly votes", bot.name, bot.monthly_votes);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Feature Flags
//!
//! - `reqwest-client` (default): Use reqwest as the HTTP backend
//! - `ureq-client`: Use ureq as the HTTP backend (enables `blocking`)
//! - `blocking`: Enable the blocking API
//! - `rustls-tls` (default): Use rustls for TLS
//! - `native-tls`: Use native TLS implementation
//! - `tracing`: Enable tracing/logging support

#![cfg_attr(docsrs, feature(doc_cfg))]
// Clippy lints
#![warn(clippy::all)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![deny(unsafe_code)]
// Allow some overly strict pedantic lints
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::similar_names)]
#![allow(clippy::module_name_repetitions)]

mod client;
#[doc(hidden)]
pub mod endpoints;
pub mod error;
mod http;
pub mod models;
mod rate_limiter;

#[cfg(feature = "blocking")]
#[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
pub mod blocking;

// Re-exports
pub use client::{Client, ClientBuilder, ClientConfig};
pub use error::{Error, Result};
pub use models::*;

/// The default base URL for the `TopStats` API.
pub const DEFAULT_BASE_URL: &str = "https://api.topstats.gg";

/// The SDK version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The User-Agent string sent with requests.
#[must_use]
pub fn user_agent() -> String {
    format!(
        "topstats-rs/{VERSION} (https://github.com/top-stats/rust-sdk)"
    )
}

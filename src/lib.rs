//! # `TopStats` Rust SDK
//!
//! A Rust SDK for interacting with the [TopStats.gg API](https://topstats.gg),
//! which provides statistics for Discord bots listed on Top.gg.
//!
//! ## Features
//!
//! - **Async-first** design with optional blocking mode
//! - **Multiple HTTP backends**: reqwest (async) or ureq (blocking)
//! - **Built-in rate limiting** with automatic retry for short delays
//! - **Type-safe** models with serde serialization
//! - **Tracing** support for logging (optional)
//!
//! ## Quick Start (Async)
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
//! ## Blocking Mode
//!
//! For synchronous code, disable default features and enable `blocking`:
//!
//! ```toml
//! [dependencies]
//! topstats = { version = "0.1", default-features = false, features = ["blocking", "ureq-client"] }
//! ```
//!
//! ```rust,ignore
//! use topstats::Client;
//!
//! fn main() -> Result<(), topstats::Error> {
//!     let client = Client::new("your-api-token")?;
//!     let bot = client.get_bot("432610292342587392")?;
//!     println!("Bot: {}", bot.name);
//!     Ok(())
//! }
//! ```
//!
//! ## Feature Flags
//!
//! - `async` (default): Enable async mode
//! - `blocking`: Enable blocking/sync mode (mutually exclusive with `async`)
//! - `reqwest-client` (default): Use reqwest as the HTTP backend (async)
//! - `ureq-client`: Use ureq as the HTTP backend (blocking)
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

// Compile-time check: async and blocking are mutually exclusive
#[cfg(all(feature = "async", feature = "blocking"))]
compile_error!(
    "Features `async` and `blocking` are mutually exclusive. \
     Use `default-features = false` and enable only one."
);

// Compile-time check: at least one mode must be enabled
#[cfg(not(any(feature = "async", feature = "blocking")))]
compile_error!(
    "Either `async` or `blocking` feature must be enabled. \
     The `async` feature is enabled by default."
);

mod client;
#[doc(hidden)]
pub mod endpoints;
pub mod error;
pub(crate) mod http;
pub mod models;

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
    format!("topstats-rust-sdk/{VERSION} (https://github.com/top-stats/rust-sdk)")
}

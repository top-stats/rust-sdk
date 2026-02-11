//! Main client implementation for the `TopStats` API.
//!
//! This module provides the [`Client`] for interacting with the `TopStats` API.
//! The client can operate in either async or blocking mode depending on the
//! `blocking` feature flag.
//!
//! # Async Mode (default)
//!
//! ```rust,no_run
//! use topstats::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), topstats::Error> {
//!     let client = Client::new("your-api-token")?;
//!     let bot = client.get_bot("432610292342587392").await?;
//!     println!("Bot: {}", bot.name);
//!     Ok(())
//! }
//! ```
//!
//! # Blocking Mode
//!
//! Enable the `blocking` feature and disable default features:
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

use std::sync::Arc;
use std::time::Duration;

use crate::endpoints;
use crate::error::{ApiErrorResponse, Error, Result};
use crate::http::{Request, Response};
use crate::models::{
    Bot, CompareHistoricalResponse, DataType, HistoricalDataResponse, RankedBot, RankingsQuery,
    RankingsResponse, RecentDataResponse, TimeFrame, UserBotsResponse,
};
use crate::{DEFAULT_BASE_URL, USER_AGENT};

/// Default maximum delay threshold before returning an error (in seconds).
pub const MAX_DELAY_THRESHOLD: f64 = 10.0;

/// Default maximum number of retry attempts for rate-limited requests.
pub const DEFAULT_MAX_RETRIES: u32 = 3;

// Type alias for the default HTTP client based on features
#[cfg(all(feature = "reqwest-client", not(feature = "blocking")))]
type DefaultHttpClient = crate::http::ReqwestClient;

#[cfg(all(feature = "ureq-client", feature = "blocking"))]
type DefaultHttpClient = crate::http::UreqClient;

/// Sleep for the given number of milliseconds (async version).
#[cfg(not(feature = "blocking"))]
async fn sleep_ms(ms: u64) {
    futures_timer::Delay::new(Duration::from_millis(ms)).await;
}

/// Sleep for the given number of milliseconds (blocking version).
#[cfg(feature = "blocking")]
fn sleep_ms(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

/// Configuration options for the `TopStats` client.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// API token for authentication.
    pub token: String,
    /// Base URL for the API.
    pub base_url: String,
    /// Whether to enable automatic rate limit handling.
    pub auto_retry: bool,
    /// Maximum delay threshold before throwing an error (in seconds).
    pub max_delay_threshold: f64,
    /// Maximum number of retry attempts for rate-limited requests.
    pub max_retries: u32,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            token: String::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            auto_retry: true,
            max_delay_threshold: MAX_DELAY_THRESHOLD,
            max_retries: DEFAULT_MAX_RETRIES,
        }
    }
}

/// Builder for creating a [`Client`].
#[derive(Debug, Default)]
pub struct ClientBuilder {
    config: ClientConfig,
}

impl ClientBuilder {
    /// Creates a new client builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the API token.
    #[must_use]
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.config.token = token.into();
        self
    }

    /// Sets the base URL for the API.
    #[must_use]
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.config.base_url = base_url.into();
        self
    }

    /// Enables or disables automatic rate limit handling.
    #[must_use]
    pub const fn auto_retry(mut self, enabled: bool) -> Self {
        self.config.auto_retry = enabled;
        self
    }

    /// Sets the maximum delay threshold before throwing an error.
    #[must_use]
    pub const fn max_delay_threshold(mut self, seconds: f64) -> Self {
        self.config.max_delay_threshold = seconds;
        self
    }

    /// Sets the maximum number of retry attempts for rate-limited requests.
    ///
    /// Default is 3. Set to 0 to disable retries entirely (equivalent to `auto_retry(false)`).
    #[must_use]
    pub const fn max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }

    /// Builds the client with the default HTTP backend.
    ///
    /// In async mode, this uses reqwest. In blocking mode, this uses ureq.
    ///
    /// # Errors
    ///
    /// Returns an error if the token is empty or if the HTTP client cannot be created.
    #[cfg(any(
        all(feature = "reqwest-client", not(feature = "blocking")),
        all(feature = "ureq-client", feature = "blocking")
    ))]
    pub fn build(self) -> Result<Client<DefaultHttpClient>> {
        self.build_with_client(DefaultHttpClient::new()?)
    }

    /// Builds the client with a custom HTTP client.
    ///
    /// # Errors
    ///
    /// Returns an error if the token is empty.
    pub fn build_with_client<H>(self, http_client: H) -> Result<Client<H>> {
        if self.config.token.is_empty() {
            return Err(Error::InvalidToken);
        }

        Ok(Client {
            config: self.config,
            http_client: Arc::new(http_client),
        })
    }
}

/// The main client for interacting with the `TopStats` API.
///
/// In async mode (default), methods return futures that must be `.await`ed.
/// In blocking mode (with `blocking` feature), methods return results directly.
#[derive(Debug)]
pub struct Client<H> {
    config: ClientConfig,
    http_client: Arc<H>,
}

impl<H> Clone for Client<H> {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            http_client: self.http_client.clone(),
        }
    }
}

// Default client constructors (work for both async and blocking modes)
#[cfg(any(
    all(feature = "reqwest-client", not(feature = "blocking")),
    all(feature = "ureq-client", feature = "blocking")
))]
impl Client<DefaultHttpClient> {
    /// Creates a new client with the given API token.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(token: impl Into<String>) -> Result<Self> {
        ClientBuilder::new().token(token).build()
    }

    /// Creates a new client builder.
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }
}

/// Trait abstracting over async and sync HTTP clients.
///
/// This allows the endpoint implementations to be generic over the HTTP client type.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to allow the client to be shared across
/// threads. This is required because [`Client`] wraps the HTTP client in an `Arc`
/// and may be cloned and used from multiple threads or async tasks concurrently.
#[maybe_async::maybe_async]
pub trait MaybeHttpClient: Send + Sync {
    /// Sends an HTTP request and returns the response.
    async fn send_request(&self, request: Request) -> Result<Response>;
}

#[maybe_async::async_impl]
impl<H: crate::http::HttpClient + Send + Sync> MaybeHttpClient for Arc<H> {
    async fn send_request(&self, request: Request) -> Result<Response> {
        self.send(request).await
    }
}

#[maybe_async::sync_impl]
impl<H: crate::http::BlockingHttpClient + Send + Sync> MaybeHttpClient for Arc<H> {
    fn send_request(&self, request: Request) -> Result<Response> {
        self.send(request)
    }
}

// Core implementation using maybe_async
impl<H> Client<H>
where
    Arc<H>: MaybeHttpClient,
{
    /// Returns the current configuration.
    #[must_use]
    pub const fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Makes an authenticated request to the API.
    #[maybe_async::maybe_async]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    async fn request(&self, endpoint: &str, query: &[(&str, &str)]) -> Result<Response> {
        self.request_with_retries(endpoint, query, self.config.max_retries)
            .await
    }

    /// Makes an authenticated request with retry tracking.
    #[maybe_async::maybe_async]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    async fn request_with_retries(
        &self,
        endpoint: &str,
        query: &[(&str, &str)],
        retries_remaining: u32,
    ) -> Result<Response> {
        let url = format!("{}{endpoint}", self.config.base_url);

        let mut request = Request::get(&url)
            .header("Authorization", &self.config.token)
            .header("Content-Type", "application/json")
            .header("User-Agent", USER_AGENT);

        for (key, value) in query {
            request = request.query(*key, *value);
        }

        #[cfg(feature = "tracing")]
        tracing::debug!("Making request to {url}");

        let response = self.http_client.send_request(request).await?;

        // Handle error responses
        if !response.is_success() {
            let error_response: ApiErrorResponse = response.json()?;

            // Auto-retry for short rate limit delays
            if let Some(expires_in) = error_response
                .expires_in
                .filter(|_| response.is_rate_limited())
                .filter(|_| self.config.auto_retry && retries_remaining > 0)
                .filter(|&e| e <= self.config.max_delay_threshold)
            {
                #[cfg(feature = "tracing")]
                tracing::debug!(
                    "Rate limited, auto-retrying after {expires_in}s ({} retries remaining)",
                    retries_remaining - 1
                );

                // Sleep and retry
                sleep_ms((expires_in * 1000.0) as u64).await;

                #[cfg(not(feature = "blocking"))]
                return Box::pin(self.request_with_retries(endpoint, query, retries_remaining - 1))
                    .await;
                #[cfg(feature = "blocking")]
                return self.request_with_retries(endpoint, query, retries_remaining - 1);
            }

            return Err(error_response.into());
        }

        Ok(response)
    }

    // ==================== Bot Endpoints ====================

    /// Gets information about a bot.
    ///
    /// # Arguments
    ///
    /// * `bot_id` - The Discord bot ID (17-19 digit snowflake).
    ///
    /// # Errors
    ///
    /// Returns an error if the bot ID is invalid or the request fails.
    #[maybe_async::maybe_async]
    pub async fn get_bot(&self, bot_id: &str) -> Result<Bot> {
        endpoints::validate_bot_id(bot_id)?;
        let endpoint = format!("/discord/bots/{bot_id}");
        let response = self.request(&endpoint, &[]).await?;
        response.json()
    }

    /// Gets historical data for a bot.
    ///
    /// # Arguments
    ///
    /// * `bot_id` - The Discord bot ID.
    /// * `time_frame` - The time period to query.
    /// * `data_type` - The type of data to retrieve.
    ///
    /// # Errors
    ///
    /// Returns an error if the bot ID is invalid or the request fails.
    #[maybe_async::maybe_async]
    pub async fn get_bot_historical(
        &self,
        bot_id: &str,
        time_frame: TimeFrame,
        data_type: DataType,
    ) -> Result<HistoricalDataResponse> {
        endpoints::validate_bot_id(bot_id)?;
        let endpoint = format!("/discord/bots/{bot_id}/historical");
        let response = self
            .request(
                &endpoint,
                &[
                    ("timeFrame", time_frame.as_str()),
                    ("type", data_type.as_str()),
                ],
            )
            .await?;
        response.json()
    }

    /// Gets recent statistics for a bot.
    ///
    /// Returns hourly data for the past 30 hours and daily data for the past month.
    ///
    /// # Arguments
    ///
    /// * `bot_id` - The Discord bot ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the bot ID is invalid or the request fails.
    #[maybe_async::maybe_async]
    pub async fn get_bot_recent(&self, bot_id: &str) -> Result<RecentDataResponse> {
        endpoints::validate_bot_id(bot_id)?;
        let endpoint = format!("/discord/bots/{bot_id}/recent");
        let response = self.request(&endpoint, &[]).await?;
        response.json()
    }

    // ==================== Rankings Endpoints ====================

    /// Gets the bot rankings.
    ///
    /// # Arguments
    ///
    /// * `query` - Query parameters for filtering and sorting.
    ///
    /// # Errors
    ///
    /// Returns an error if the query is invalid or the request fails.
    #[maybe_async::maybe_async]
    #[allow(clippy::needless_pass_by_value)]
    pub async fn get_rankings(&self, query: RankingsQuery) -> Result<RankingsResponse> {
        query.validate()?;
        let params = endpoints::build_rankings_params(&query);
        let query_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let response = self.request("/discord/rankings/bots", &query_refs).await?;
        response.json()
    }

    // ==================== Search Endpoints ====================

    /// Searches for bots by name.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query.
    /// * `limit` - Maximum number of results (default: 50, max: 100).
    /// * `offset` - Offset for pagination.
    /// * `include_deleted` - Whether to include bots that have been removed from Top.gg.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    #[maybe_async::maybe_async]
    pub async fn search_bots(
        &self,
        query: &str,
        limit: Option<u32>,
        offset: Option<u32>,
        include_deleted: Option<bool>,
    ) -> Result<Vec<Bot>> {
        let params = endpoints::build_search_params(query, limit, offset, include_deleted);
        let query_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let response = self.request("/search", &query_refs).await?;
        response.json()
    }

    /// Searches for bots by tag.
    ///
    /// # Arguments
    ///
    /// * `tag` - The tag to search for.
    /// * `limit` - Maximum number of results (default: 50, max: 50).
    /// * `offset` - Offset for pagination.
    /// * `include_deleted` - Whether to include bots that have been removed from Top.gg.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    #[maybe_async::maybe_async]
    pub async fn search_by_tag(
        &self,
        tag: &str,
        limit: Option<u32>,
        offset: Option<u32>,
        include_deleted: Option<bool>,
    ) -> Result<Vec<Bot>> {
        let params = endpoints::build_search_params(tag, limit, offset, include_deleted);
        let query_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let response = self.request("/discord/tags", &query_refs).await?;
        let tag_response: endpoints::TagResponse = response.json()?;
        Ok(tag_response.data.results)
    }

    // ==================== Compare Endpoints ====================

    /// Compares one or more bots.
    ///
    /// # Arguments
    ///
    /// * `bot_ids` - Array of bot IDs to compare (up to 8 for premium users).
    ///
    /// # Errors
    ///
    /// Returns an error if any bot ID is invalid or the request fails.
    #[maybe_async::maybe_async]
    pub async fn compare_bots(&self, bot_ids: &[&str]) -> Result<Vec<RankedBot>> {
        for id in bot_ids {
            endpoints::validate_bot_id(id)?;
        }

        let path = bot_ids.join("/");
        let endpoint = format!("/discord/compare/{path}");
        let response = self.request(&endpoint, &[]).await?;
        let compare_response: endpoints::CompareResponse = response.json()?;
        Ok(compare_response.data)
    }

    /// Compares historical data for one or more bots.
    ///
    /// # Arguments
    ///
    /// * `bot_ids` - Array of bot IDs to compare (up to 8 for premium users).
    /// * `time_frame` - The time period to query.
    /// * `data_type` - The type of data to retrieve.
    ///
    /// # Errors
    ///
    /// Returns an error if any bot ID is invalid or the request fails.
    #[maybe_async::maybe_async]
    pub async fn compare_bots_historical(
        &self,
        bot_ids: &[&str],
        time_frame: TimeFrame,
        data_type: DataType,
    ) -> Result<CompareHistoricalResponse> {
        for id in bot_ids {
            endpoints::validate_bot_id(id)?;
        }

        let path = bot_ids.join("/");
        let endpoint = format!("/discord/compare/historical/{path}");
        let response = self
            .request(
                &endpoint,
                &[
                    ("timeFrame", time_frame.as_str()),
                    ("type", data_type.as_str()),
                ],
            )
            .await?;
        response.json()
    }

    // ==================== User Endpoints ====================

    /// Gets all bots owned by a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The Discord user ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the user ID is invalid or the request fails.
    ///
    /// # Note
    ///
    /// Data may be inaccurate as bots transferred to teams still appear
    /// on the original owner's account.
    #[maybe_async::maybe_async]
    pub async fn get_user_bots(&self, user_id: &str) -> Result<UserBotsResponse> {
        endpoints::validate_bot_id(user_id)?; // User IDs are also snowflakes
        let endpoint = format!("/discord/users/{user_id}/bots");
        let response = self.request(&endpoint, &[]).await?;
        response.json()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_builder() {
        let builder = ClientBuilder::new()
            .token("test-token")
            .base_url("https://custom.api.com")
            .auto_retry(false)
            .max_delay_threshold(10.0);

        assert_eq!(builder.config.token, "test-token");
        assert_eq!(builder.config.base_url, "https://custom.api.com");
        assert!(!builder.config.auto_retry);
        assert!((builder.config.max_delay_threshold - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_validate_bot_id() {
        assert!(endpoints::validate_bot_id("432610292342587392").is_ok());
        assert!(endpoints::validate_bot_id("123").is_err());
        assert!(endpoints::validate_bot_id("abc").is_err());
    }

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.base_url, DEFAULT_BASE_URL);
        assert!(config.auto_retry);
        assert!((config.max_delay_threshold - MAX_DELAY_THRESHOLD).abs() < f64::EPSILON);
        assert_eq!(config.max_retries, DEFAULT_MAX_RETRIES);
    }

    #[test]
    fn test_client_builder_max_retries() {
        let builder = ClientBuilder::new().token("test").max_retries(5);
        assert_eq!(builder.config.max_retries, 5);
    }
}

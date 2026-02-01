//! Blocking API for the `TopStats` SDK.
//!
//! This module provides a synchronous client for use in non-async contexts.
//!
//! # Example
//!
//! ```rust,no_run
//! use topstats::blocking::Client;
//!
//! fn main() -> Result<(), topstats::Error> {
//!     let client = Client::new("your-api-token")?;
//!     
//!     let bot = client.get_bot("432610292342587392")?;
//!     println!("Bot: {} has {} monthly votes", bot.name, bot.monthly_votes);
//!     
//!     Ok(())
//! }
//! ```

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::error::{ApiErrorResponse, Error, Result};
use crate::http::{BlockingHttpClient, Request};
use crate::models::{
    Bot, CompareHistoricalResponse, DataType, HistoricalDataResponse, RankedBot, RankingsQuery,
    RankingsResponse, RecentDataResponse, TimeFrame, UserBotsResponse,
};
use crate::rate_limiter::MAX_DELAY_THRESHOLD;
use crate::{user_agent, DEFAULT_BASE_URL};

#[cfg(feature = "ureq-client")]
use crate::http::UreqClient;

/// Configuration options for the blocking `TopStats` client.
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
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            token: String::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            auto_retry: true,
            max_delay_threshold: MAX_DELAY_THRESHOLD,
        }
    }
}

/// Builder for creating a blocking [`Client`].
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

    /// Builds the client with the ureq HTTP backend.
    ///
    /// # Errors
    ///
    /// Returns an error if the token is empty.
    #[cfg(feature = "ureq-client")]
    pub fn build(self) -> Result<Client<UreqClient>> {
        if self.config.token.is_empty() {
            return Err(Error::InvalidToken);
        }

        let http_client = UreqClient::new();
        Ok(Client {
            config: self.config,
            http_client: Arc::new(http_client),
        })
    }

    /// Builds the client with a custom blocking HTTP client.
    ///
    /// # Errors
    ///
    /// Returns an error if the token is empty.
    pub fn build_with_client<H: BlockingHttpClient>(self, http_client: H) -> Result<Client<H>> {
        if self.config.token.is_empty() {
            return Err(Error::InvalidToken);
        }

        Ok(Client {
            config: self.config,
            http_client: Arc::new(http_client),
        })
    }
}

/// Blocking client for interacting with the `TopStats` API.
#[derive(Debug)]
#[allow(clippy::struct_field_names)]
pub struct Client<H: BlockingHttpClient> {
    config: ClientConfig,
    http_client: Arc<H>,
}

impl<H: BlockingHttpClient> Clone for Client<H> {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            http_client: Arc::clone(&self.http_client),
        }
    }
}

#[cfg(feature = "ureq-client")]
impl Client<UreqClient> {
    /// Creates a new blocking client with the given API token.
    ///
    /// # Errors
    ///
    /// Returns an error if the token is empty.
    pub fn new(token: impl Into<String>) -> Result<Self> {
        ClientBuilder::new().token(token).build()
    }

    /// Creates a new client builder.
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }
}

impl<H: BlockingHttpClient> Client<H> {
    /// Returns the current configuration.
    #[must_use]
    pub const fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Validates a Discord bot ID format.
    fn validate_bot_id(id: &str) -> Result<()> {
        if Bot::validate_id(id) {
            Ok(())
        } else {
            Err(Error::InvalidBotId(id.to_string()))
        }
    }

    /// Makes an authenticated request to the API.
    fn request(&self, endpoint: &str, query: &[(&str, &str)]) -> Result<crate::http::Response> {
        let url = format!("{}{}", self.config.base_url, endpoint);

        let mut request = Request::get(&url)
            .header("Authorization", &self.config.token)
            .header("Content-Type", "application/json")
            .header("User-Agent", user_agent());

        for (key, value) in query {
            request = request.query(*key, *value);
        }

        let response = self.http_client.send(request)?;

        // Handle error responses
        if !response.is_success() {
            let error_response: ApiErrorResponse = response.json()?;

            // Auto-retry for short rate limit delays
            if response.is_rate_limited() {
                if let Some(expires_in) = error_response.expires_in {
                    if self.config.auto_retry && expires_in <= self.config.max_delay_threshold {
                        thread::sleep(Duration::from_secs_f64(expires_in));
                        return self.request(endpoint, query);
                    }
                }
            }

            return Err(error_response.into());
        }

        Ok(response)
    }

    // ==================== Bot Endpoints ====================

    /// Gets information about a bot.
    ///
    /// # Errors
    ///
    /// Returns an error if the bot ID is invalid or the request fails.
    pub fn get_bot(&self, bot_id: &str) -> Result<Bot> {
        Self::validate_bot_id(bot_id)?;
        let endpoint = format!("/discord/bots/{bot_id}");
        let response = self.request(&endpoint, &[])?;
        response.json()
    }

    /// Gets historical data for a bot.
    ///
    /// # Errors
    ///
    /// Returns an error if the bot ID is invalid or the request fails.
    pub fn get_bot_historical(
        &self,
        bot_id: &str,
        time_frame: TimeFrame,
        data_type: DataType,
    ) -> Result<HistoricalDataResponse> {
        Self::validate_bot_id(bot_id)?;
        let endpoint = format!("/discord/bots/{bot_id}/historical");
        let response = self.request(
            &endpoint,
            &[
                ("timeFrame", time_frame.as_str()),
                ("type", data_type.as_str()),
            ],
        )?;
        response.json()
    }

    /// Gets recent statistics for a bot.
    ///
    /// # Errors
    ///
    /// Returns an error if the bot ID is invalid or the request fails.
    pub fn get_bot_recent(&self, bot_id: &str) -> Result<RecentDataResponse> {
        Self::validate_bot_id(bot_id)?;
        let endpoint = format!("/discord/bots/{bot_id}/recent");
        let response = self.request(&endpoint, &[])?;
        response.json()
    }

    // ==================== Rankings Endpoints ====================

    /// Gets the bot rankings.
    ///
    /// # Errors
    ///
    /// Returns an error if the query is invalid or the request fails.
    #[allow(clippy::needless_pass_by_value)]
    pub fn get_rankings(&self, query: RankingsQuery) -> Result<RankingsResponse> {
        query.validate()?;

        let mut params: Vec<(&str, String)> = vec![
            ("sortBy", query.sort_by.to_string()),
            ("sortMethod", query.sort_order.to_string()),
        ];

        if let Some(limit) = query.limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(offset) = query.offset {
            params.push(("offset", offset.to_string()));
        }

        let query_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let response = self.request("/discord/rankings/bots", &query_refs)?;
        response.json()
    }

    // ==================== Search Endpoints ====================

    /// Searches for bots by name.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub fn search_bots(
        &self,
        query: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Bot>> {
        let mut params: Vec<(&str, String)> = vec![("query", query.to_string())];

        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(offset) = offset {
            params.push(("offset", offset.to_string()));
        }

        let query_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let response = self.request("/search", &query_refs)?;
        response.json()
    }

    /// Searches for bots by tag.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    #[allow(clippy::items_after_statements)]
    pub fn search_by_tag(
        &self,
        tag: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<Bot>> {
        let mut params: Vec<(&str, String)> = vec![("query", tag.to_string())];

        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(offset) = offset {
            params.push(("offset", offset.to_string()));
        }

        let query_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        #[derive(serde::Deserialize)]
        struct TagResponse {
            data: TagData,
        }

        #[derive(serde::Deserialize)]
        struct TagData {
            results: Vec<Bot>,
        }

        let response = self.request("/discord/tags", &query_refs)?;
        let tag_response: TagResponse = response.json()?;
        Ok(tag_response.data.results)
    }

    // ==================== Compare Endpoints ====================

    /// Compares multiple bots.
    ///
    /// # Errors
    ///
    /// Returns an error if the number of IDs is invalid or the request fails.
    #[allow(clippy::items_after_statements)]
    pub fn compare_bots(&self, bot_ids: &[&str]) -> Result<Vec<RankedBot>> {
        let count = bot_ids.len();
        if !(2..=4).contains(&count) {
            return Err(Error::InvalidCompareCount(count));
        }

        for id in bot_ids {
            Self::validate_bot_id(id)?;
        }

        let path = bot_ids.join("/");
        let endpoint = format!("/discord/compare/{path}");

        #[derive(serde::Deserialize)]
        struct CompareResponse {
            data: Vec<RankedBot>,
        }

        let response = self.request(&endpoint, &[])?;
        let compare_response: CompareResponse = response.json()?;
        Ok(compare_response.data)
    }

    /// Compares historical data for multiple bots.
    ///
    /// # Errors
    ///
    /// Returns an error if the number of IDs is invalid or the request fails.
    pub fn compare_bots_historical(
        &self,
        bot_ids: &[&str],
        time_frame: TimeFrame,
        data_type: DataType,
    ) -> Result<CompareHistoricalResponse> {
        let count = bot_ids.len();
        if !(2..=4).contains(&count) {
            return Err(Error::InvalidCompareCount(count));
        }

        for id in bot_ids {
            Self::validate_bot_id(id)?;
        }

        let path = bot_ids.join("/");
        let endpoint = format!("/discord/compare/historical/{path}");

        let response = self.request(
            &endpoint,
            &[
                ("timeFrame", time_frame.as_str()),
                ("type", data_type.as_str()),
            ],
        )?;
        response.json()
    }

    // ==================== User Endpoints ====================

    /// Gets all bots owned by a user.
    ///
    /// # Errors
    ///
    /// Returns an error if the user ID is invalid or the request fails.
    pub fn get_user_bots(&self, user_id: &str) -> Result<UserBotsResponse> {
        Self::validate_bot_id(user_id)?;
        let endpoint = format!("/discord/users/{user_id}/bots");
        let response = self.request(&endpoint, &[])?;
        response.json()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocking_client_builder() {
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
    fn test_blocking_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.base_url, DEFAULT_BASE_URL);
        assert!(config.auto_retry);
        assert!((config.max_delay_threshold - MAX_DELAY_THRESHOLD).abs() < f64::EPSILON);
    }
}

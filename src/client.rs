//! Main client implementation for the `TopStats` API.

use std::sync::Arc;
use std::time::Duration;

use crate::error::{ApiErrorResponse, Error, Result};
use crate::http::{HttpClient, Request, Response};
use crate::models::{
    Bot, CompareHistoricalResponse, DataType, HistoricalDataResponse, RankedBot,
    RankingsQuery, RankingsResponse, RecentDataResponse, TimeFrame, UserBotsResponse,
};
use crate::rate_limiter::{RateLimiterManager, MAX_DELAY_THRESHOLD};
use crate::{DEFAULT_BASE_URL, user_agent};

#[cfg(feature = "reqwest-client")]
use crate::http::ReqwestClient;

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

    /// Builds the client with the default HTTP backend (reqwest).
    ///
    /// # Errors
    ///
    /// Returns an error if the token is empty or if the HTTP client cannot be created.
    #[cfg(feature = "reqwest-client")]
    pub fn build(self) -> Result<Client<ReqwestClient>> {
        if self.config.token.is_empty() {
            return Err(Error::InvalidToken);
        }

        let http_client = ReqwestClient::new()?;
        Ok(Client {
            config: self.config,
            http_client: Arc::new(http_client),
            rate_limiter: RateLimiterManager::new(),
        })
    }

    /// Builds the client with a custom HTTP client.
    ///
    /// # Errors
    ///
    /// Returns an error if the token is empty.
    pub fn build_with_client<H: HttpClient>(self, http_client: H) -> Result<Client<H>> {
        if self.config.token.is_empty() {
            return Err(Error::InvalidToken);
        }

        Ok(Client {
            config: self.config,
            http_client: Arc::new(http_client),
            rate_limiter: RateLimiterManager::new(),
        })
    }
}

/// The main client for interacting with the `TopStats` API.
#[derive(Debug)]
#[allow(clippy::struct_field_names)]
pub struct Client<H: HttpClient> {
    config: ClientConfig,
    http_client: Arc<H>,
    rate_limiter: RateLimiterManager,
}

impl<H: HttpClient> Clone for Client<H> {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            http_client: Arc::clone(&self.http_client),
            rate_limiter: self.rate_limiter.clone(),
        }
    }
}

#[cfg(feature = "reqwest-client")]
impl Client<ReqwestClient> {
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

impl<H: HttpClient> Client<H> {
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
    async fn request(&self, endpoint: &str, query: &[(&str, &str)]) -> Result<Response> {
        let url = format!("{}{}", self.config.base_url, endpoint);

        // Check rate limiter
        if self.config.auto_retry {
            if let Some(wait_time) = self.rate_limiter.check(endpoint).await {
                if wait_time.as_secs_f64() > self.config.max_delay_threshold {
                    return Err(Error::RateLimited {
                        retry_after: wait_time.as_secs_f64(),
                        message: "Rate limit exceeded".to_string(),
                    });
                }
                // Auto-wait for short delays
                #[cfg(feature = "tracing")]
                tracing::debug!("Rate limited, waiting {:?}", wait_time);
                futures_timer::Delay::new(wait_time).await;
            }
        }

        let mut request = Request::get(&url)
            .header("Authorization", &self.config.token)
            .header("Content-Type", "application/json")
            .header("User-Agent", user_agent());

        for (key, value) in query {
            request = request.query(*key, *value);
        }

        #[cfg(feature = "tracing")]
        tracing::debug!("Making request to {}", url);

        let response = self.http_client.send(request).await?;

        // Handle error responses
        if !response.is_success() {
            let error_response: ApiErrorResponse = response.json()?;

            // Record rate limit if applicable
            if response.is_rate_limited() {
                if let Some(expires_in) = error_response.expires_in {
                    self.rate_limiter
                        .record_rate_limit(endpoint, Duration::from_secs_f64(expires_in))
                        .await;

                    // Auto-retry for short delays
                    if self.config.auto_retry && expires_in <= self.config.max_delay_threshold {
                        #[cfg(feature = "tracing")]
                        tracing::debug!("Rate limited, auto-retrying after {}s", expires_in);
                        futures_timer::Delay::new(Duration::from_secs_f64(expires_in)).await;
                        return Box::pin(self.request(endpoint, query)).await;
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
    /// # Arguments
    ///
    /// * `bot_id` - The Discord bot ID (17-19 digit snowflake).
    ///
    /// # Errors
    ///
    /// Returns an error if the bot ID is invalid or the request fails.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), topstats::Error> {
    /// # let client = topstats::Client::new("token")?;
    /// let bot = client.get_bot("432610292342587392").await?;
    /// println!("Bot: {} has {} monthly votes", bot.name, bot.monthly_votes);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_bot(&self, bot_id: &str) -> Result<Bot> {
        Self::validate_bot_id(bot_id)?;
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
    pub async fn get_bot_historical(
        &self,
        bot_id: &str,
        time_frame: TimeFrame,
        data_type: DataType,
    ) -> Result<HistoricalDataResponse> {
        Self::validate_bot_id(bot_id)?;
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
    pub async fn get_bot_recent(&self, bot_id: &str) -> Result<RecentDataResponse> {
        Self::validate_bot_id(bot_id)?;
        let endpoint = format!("/discord/bots/{bot_id}/recent");
        let response = self.request(&endpoint, &[]).await?;
        response.json()
    }

    // ==================== Rankings Endpoints ====================

    /// Gets the bot rankings.
    ///
    /// # Arguments
    ///
    /// * `query` - Optional query parameters for filtering and sorting.
    ///
    /// # Errors
    ///
    /// Returns an error if the query is invalid or the request fails.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), topstats::Error> {
    /// # let client = topstats::Client::new("token")?;
    /// use topstats::{RankingsQuery, SortBy};
    ///
    /// let rankings = client.get_rankings(
    ///     RankingsQuery::new()
    ///         .sort_by(SortBy::MonthlyVotes)
    ///         .limit(100)
    /// ).await?;
    ///
    /// for bot in &rankings.data {
    ///     println!("#{}: {} ({} votes)", bot.monthly_votes_rank, bot.name, bot.monthly_votes);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_rankings(&self, query: RankingsQuery) -> Result<RankingsResponse> {
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
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn search_bots(
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
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    #[allow(clippy::items_after_statements)]
    pub async fn search_by_tag(
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

        let response = self.request("/discord/tags", &query_refs).await?;
        let tag_response: TagResponse = response.json()?;
        Ok(tag_response.data.results)
    }

    // ==================== Compare Endpoints ====================

    /// Compares multiple bots.
    ///
    /// # Arguments
    ///
    /// * `bot_ids` - Array of 2-4 bot IDs to compare.
    ///
    /// # Errors
    ///
    /// Returns an error if the number of IDs is invalid or the request fails.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # async fn example() -> Result<(), topstats::Error> {
    /// # let client = topstats::Client::new("token")?;
    /// let bots = client.compare_bots(&[
    ///     "432610292342587392",
    ///     "646937666251915264"
    /// ]).await?;
    ///
    /// for bot in &bots {
    ///     println!("{}: {} monthly votes", bot.name, bot.monthly_votes);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::items_after_statements)]
    pub async fn compare_bots(&self, bot_ids: &[&str]) -> Result<Vec<RankedBot>> {
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

        let response = self.request(&endpoint, &[]).await?;
        let compare_response: CompareResponse = response.json()?;
        Ok(compare_response.data)
    }

    /// Compares historical data for multiple bots.
    ///
    /// # Arguments
    ///
    /// * `bot_ids` - Array of 2-4 bot IDs to compare.
    /// * `time_frame` - The time period to query.
    /// * `data_type` - The type of data to retrieve.
    ///
    /// # Errors
    ///
    /// Returns an error if the number of IDs is invalid or the request fails.
    pub async fn compare_bots_historical(
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
    pub async fn get_user_bots(&self, user_id: &str) -> Result<UserBotsResponse> {
        Self::validate_bot_id(user_id)?; // User IDs are also snowflakes
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
        assert!(Client::<ReqwestClient>::validate_bot_id("432610292342587392").is_ok());
        assert!(Client::<ReqwestClient>::validate_bot_id("123").is_err());
        assert!(Client::<ReqwestClient>::validate_bot_id("abc").is_err());
    }

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.base_url, DEFAULT_BASE_URL);
        assert!(config.auto_retry);
        assert!((config.max_delay_threshold - MAX_DELAY_THRESHOLD).abs() < f64::EPSILON);
    }
}

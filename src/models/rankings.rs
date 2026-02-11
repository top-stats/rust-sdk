//! Rankings data models.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Sort criteria for rankings queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum SortBy {
    /// Sort by monthly votes rank.
    #[default]
    MonthlyVotes,
    /// Sort by total votes rank.
    TotalVotes,
    /// Sort by server count rank.
    ServerCount,
}

impl SortBy {
    /// Returns the API query parameter value for this sort criteria.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MonthlyVotes => "monthly_votes_rank",
            Self::TotalVotes => "total_votes_rank",
            Self::ServerCount => "server_count_rank",
        }
    }
}

impl fmt::Display for SortBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Sort order for rankings queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum SortOrder {
    /// Ascending order (lowest rank first = best).
    #[default]
    Ascending,
    /// Descending order (highest rank first = worst).
    Descending,
}

impl SortOrder {
    /// Returns the API query parameter value for this sort order.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ascending => "asc",
            Self::Descending => "desc",
        }
    }
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A bot entry in the rankings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RankedBot {
    /// The bot's Discord ID.
    pub id: String,

    /// The bot's display name.
    pub name: String,

    /// Current monthly vote count.
    pub monthly_votes: i64,

    /// Rank by monthly votes.
    pub monthly_votes_rank: i64,

    /// Number of servers the bot is in.
    pub server_count: Option<i64>,

    /// Rank by server count.
    pub server_count_rank: Option<i64>,

    /// Total vote count.
    pub total_votes: i64,

    /// Rank by total votes.
    pub total_votes_rank: i64,

    /// Number of reviews.
    pub review_count: Option<i64>,

    /// Change in monthly votes rank since last update.
    pub monthly_votes_rank_change: Option<i64>,

    /// Change in server count rank since last update.
    pub server_count_rank_change: Option<i64>,

    /// Change in total votes rank since last update.
    pub total_votes_rank_change: Option<i64>,
}

/// Response from the rankings endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RankingsResponse {
    /// Total number of bots tracked.
    #[serde(rename = "totalBotCount")]
    pub total_bot_count: i64,

    /// Array of ranked bots.
    pub data: Vec<RankedBot>,
}

/// Builder for rankings query parameters.
#[derive(Debug, Clone, Default)]
pub struct RankingsQuery {
    /// Maximum number of results (1-500).
    pub limit: Option<u16>,
    /// Offset for pagination.
    pub offset: Option<u32>,
    /// Sort criteria.
    pub sort_by: SortBy,
    /// Sort order.
    pub sort_order: SortOrder,
}

impl RankingsQuery {
    /// Creates a new rankings query with default values.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            limit: None,
            offset: None,
            sort_by: SortBy::MonthlyVotes,
            sort_order: SortOrder::Ascending,
        }
    }

    /// Sets the maximum number of results (1-500).
    #[must_use]
    pub const fn limit(mut self, limit: u16) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the offset for pagination.
    #[must_use]
    pub const fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Sets the sort criteria.
    #[must_use]
    pub const fn sort_by(mut self, sort_by: SortBy) -> Self {
        self.sort_by = sort_by;
        self
    }

    /// Sets the sort order.
    #[must_use]
    pub const fn sort_order(mut self, sort_order: SortOrder) -> Self {
        self.sort_order = sort_order;
        self
    }

    /// Validates the query parameters.
    ///
    /// # Errors
    ///
    /// Returns an error if the limit is outside the valid range (1-500).
    pub fn validate(&self) -> Result<(), crate::Error> {
        if let Some(limit) = self.limit.filter(|&l| l == 0 || l > 500) {
            return Err(crate::Error::InvalidInput(format!(
                "Rankings limit must be between 1 and 500, got {limit}"
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_by_as_str() {
        assert_eq!(SortBy::MonthlyVotes.as_str(), "monthly_votes_rank");
        assert_eq!(SortBy::TotalVotes.as_str(), "total_votes_rank");
        assert_eq!(SortBy::ServerCount.as_str(), "server_count_rank");
    }

    #[test]
    fn test_sort_order_as_str() {
        assert_eq!(SortOrder::Ascending.as_str(), "asc");
        assert_eq!(SortOrder::Descending.as_str(), "desc");
    }

    #[test]
    fn test_ranked_bot_deserialization() {
        let json = r#"{
            "id": "432610292342587392",
            "name": "Mudae",
            "monthly_votes": 2312207,
            "monthly_votes_rank": 1,
            "server_count": 3371839,
            "server_count_rank": 13,
            "total_votes": 205265098,
            "total_votes_rank": 1,
            "review_count": 1050,
            "monthly_votes_rank_change": 1,
            "server_count_rank_change": 2,
            "total_votes_rank_change": 3
        }"#;

        let bot: RankedBot = serde_json::from_str(json).unwrap();
        assert_eq!(bot.id, "432610292342587392");
        assert_eq!(bot.name, "Mudae");
        assert_eq!(bot.monthly_votes_rank, 1);
        assert_eq!(bot.server_count, Some(3_371_839));
        assert_eq!(bot.monthly_votes_rank_change, Some(1));
    }

    #[test]
    fn test_rankings_response_deserialization() {
        let json = r#"{
            "totalBotCount": 43990,
            "data": [
                {
                    "id": "432610292342587392",
                    "name": "Mudae",
                    "monthly_votes": 2312207,
                    "monthly_votes_rank": 1,
                    "total_votes": 205265098,
                    "total_votes_rank": 1
                }
            ]
        }"#;

        let response: RankingsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.total_bot_count, 43990);
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].name, "Mudae");
    }

    #[test]
    fn test_rankings_query_builder() {
        let query = RankingsQuery::new()
            .limit(100)
            .offset(50)
            .sort_by(SortBy::TotalVotes)
            .sort_order(SortOrder::Descending);

        assert_eq!(query.limit, Some(100));
        assert_eq!(query.offset, Some(50));
        assert_eq!(query.sort_by, SortBy::TotalVotes);
        assert_eq!(query.sort_order, SortOrder::Descending);
    }

    #[test]
    fn test_rankings_query_validation() {
        let valid_query = RankingsQuery::new().limit(100);
        assert!(valid_query.validate().is_ok());

        let invalid_query_zero = RankingsQuery::new().limit(0);
        assert!(invalid_query_zero.validate().is_err());

        let invalid_query_too_large = RankingsQuery::new().limit(501);
        assert!(invalid_query_too_large.validate().is_err());
    }

    #[test]
    fn test_defaults() {
        assert_eq!(SortBy::default(), SortBy::MonthlyVotes);
        assert_eq!(SortOrder::default(), SortOrder::Ascending);
    }
}

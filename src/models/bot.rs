//! Bot-related models.

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::snowflake;

/// Full bot data from the `TopStats` API.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Bot {
    /// The bot's Discord ID.
    #[serde(deserialize_with = "snowflake::as_string::deserialize")]
    pub id: u64,

    /// The bot's Top.gg ID (may differ from Discord ID).
    #[serde(
        deserialize_with = "snowflake::option_as_string::deserialize",
        rename = "topGGId"
    )]
    pub topgg_id: Option<u64>,

    /// Array of owner Discord IDs.
    #[serde(deserialize_with = "snowflake::vec_as_string::deserialize")]
    pub owners: Vec<u64>,

    /// Whether the bot has been deleted from Top.gg.
    pub deleted: bool,

    /// The bot's display name.
    pub name: String,

    /// Default avatar identifier.
    #[serde(rename = "def_avatar")]
    pub default_avatar: Option<String>,

    /// Avatar URL.
    pub avatar: Option<String>,

    /// Short description of the bot.
    #[serde(rename = "short_desc")]
    pub short_description: String,

    /// Command prefix.
    pub prefix: String,

    /// Bot's website URL.
    pub website: Option<String>,

    /// When the bot was approved on Top.gg.
    #[serde(rename = "approved_at")]
    pub approved_at: DateTime<Utc>,

    /// Bot's tags/categories.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Current monthly vote count.
    pub monthly_votes: i64,

    /// Total vote count across all time.
    pub total_votes: i64,

    /// Number of servers the bot is in.
    pub server_count: Option<i64>,

    /// Number of reviews.
    pub review_count: Option<i64>,

    /// Average review rating.
    pub avg_review_rating: Option<f64>,

    /// Rank by monthly votes.
    pub monthly_votes_rank: i64,

    /// Rank by server count.
    pub server_count_rank: Option<i64>,

    /// Rank by total votes.
    pub total_votes_rank: i64,

    /// Last update timestamp.
    pub timestamp: DateTime<Utc>,

    /// Unix timestamp of last update (as string).
    pub unix_timestamp: Option<String>,

    /// Percentage changes in statistics.
    #[serde(rename = "percentage_changes")]
    pub percentage_changes: Option<PercentageChanges>,
}

/// Percentage changes in bot statistics.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PercentageChanges {
    /// Daily percentage change.
    pub daily: Option<f64>,
    /// Monthly percentage change.
    pub monthly: Option<f64>,
}

/// A partial bot with basic information (used in rankings and comparisons).
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PartialBot {
    /// The bot's Discord ID.
    #[serde(deserialize_with = "snowflake::as_string::deserialize")]
    pub id: u64,

    /// The bot's display name.
    pub name: String,

    /// Current monthly vote count.
    pub monthly_votes: i64,

    /// Total vote count.
    pub total_votes: i64,

    /// Number of servers.
    pub server_count: Option<i64>,

    /// Number of reviews.
    pub review_count: Option<i64>,

    /// Rank by monthly votes.
    pub monthly_votes_rank: i64,

    /// Rank by server count.
    pub server_count_rank: Option<i64>,

    /// Rank by total votes.
    pub total_votes_rank: i64,
}

impl Bot {
    /// Validates that the given value is a valid Discord snowflake ID.
    ///
    /// Discord snowflakes are 17-19 digit integers.
    #[must_use]
    pub const fn validate_id(id: u64) -> bool {
        // 17 digits minimum: 10^16 = 10_000_000_000_000_000
        id >= 10_000_000_000_000_000
    }

    /// Returns the creation timestamp of this bot based on its Discord snowflake ID.
    ///
    /// Discord snowflakes encode the creation timestamp in the first 42 bits.
    #[must_use]
    pub const fn created_at(&self) -> Option<DateTime<Utc>> {
        snowflake_to_datetime(self.id)
    }
}

impl PartialBot {
    /// Returns the creation timestamp of this bot based on its Discord snowflake ID.
    #[must_use]
    pub const fn created_at(&self) -> Option<DateTime<Utc>> {
        snowflake_to_datetime(self.id)
    }
}

/// Discord epoch: 2015-01-01T00:00:00Z in milliseconds.
const DISCORD_EPOCH: i64 = 1_420_070_400_000;

/// Converts a Discord snowflake ID to a `DateTime`.
const fn snowflake_to_datetime(id: u64) -> Option<DateTime<Utc>> {
    #[allow(clippy::cast_possible_wrap)]
    let timestamp_ms = ((id >> 22) as i64) + DISCORD_EPOCH;
    DateTime::from_timestamp_millis(timestamp_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_bot_id() {
        // Valid IDs
        assert!(Bot::validate_id(432_610_292_342_587_392)); // 18 digits
        assert!(Bot::validate_id(10_000_000_000_000_000)); // 17 digits (minimum)
        assert!(Bot::validate_id(1_234_567_890_123_456_789)); // 19 digits

        // Invalid IDs
        assert!(!Bot::validate_id(123)); // Too small
        assert!(!Bot::validate_id(0)); // Zero
        assert!(!Bot::validate_id(9_999_999_999_999_999)); // 16 digits
    }

    #[test]
    fn test_snowflake_to_datetime() {
        // Known snowflake: 432610292342587392 created around 2018-04-08
        let dt = snowflake_to_datetime(432_610_292_342_587_392).unwrap();
        assert_eq!(dt.year(), 2018);
        assert_eq!(dt.month(), 4);
    }

    #[test]
    fn test_bot_deserialization() {
        let json = r#"{
            "id": "583807014896140293",
            "topGGId": "583807014896140293",
            "owners": ["321714991050784770"],
            "deleted": false,
            "name": "TopStats",
            "def_avatar": "1.png",
            "short_desc": "Get statistics of every bot",
            "prefix": "/",
            "website": "https://topstats.gg",
            "approved_at": "2019-05-30T00:00:00.000Z",
            "tags": ["bot-statistics", "rankings"],
            "monthly_votes": 4,
            "total_votes": 278,
            "server_count": 364,
            "review_count": 0,
            "monthly_votes_rank": 1649,
            "server_count_rank": 3420,
            "total_votes_rank": 4491,
            "timestamp": "2024-10-28T18:00:00.000Z"
        }"#;

        let bot: Bot = serde_json::from_str(json).unwrap();
        assert_eq!(bot.id, 583_807_014_896_140_293);
        assert_eq!(bot.name, "TopStats");
        assert_eq!(bot.monthly_votes, 4);
        assert_eq!(bot.owners.len(), 1);
        assert_eq!(bot.owners[0], 321_714_991_050_784_770);
        assert!(!bot.deleted);
        assert_eq!(bot.tags.len(), 2);
    }

    #[test]
    fn test_partial_bot_deserialization() {
        let json = r#"{
            "id": "432610292342587392",
            "name": "Mudae",
            "monthly_votes": 2312207,
            "monthly_votes_rank": 1,
            "server_count": 3371839,
            "server_count_rank": 13,
            "total_votes": 205265098,
            "total_votes_rank": 1
        }"#;

        let bot: PartialBot = serde_json::from_str(json).unwrap();
        assert_eq!(bot.id, 432_610_292_342_587_392);
        assert_eq!(bot.name, "Mudae");
        assert_eq!(bot.monthly_votes_rank, 1);
    }

    use chrono::Datelike;
}

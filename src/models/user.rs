//! User-related models.

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::snowflake;

/// A bot owned by a user.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct UserBot {
    /// The bot's Discord ID.
    #[serde(deserialize_with = "snowflake::as_string::deserialize")]
    pub id: u64,

    /// Array of owner Discord IDs.
    #[serde(deserialize_with = "snowflake::vec_as_string::deserialize")]
    pub owners: Vec<u64>,

    /// Whether the bot has been deleted from Top.gg.
    pub deleted: bool,

    /// The bot's display name.
    pub name: String,

    /// Avatar URL.
    pub avatar: Option<String>,

    /// Short description of the bot.
    #[serde(rename = "short_desc")]
    pub short_description: String,

    /// Library used (deprecated).
    pub lib: Option<String>,

    /// Command prefix.
    pub prefix: String,

    /// Bot's website URL.
    pub website: Option<String>,

    /// When the bot was approved on Top.gg.
    #[serde(rename = "approved_at")]
    pub approved_at: Option<DateTime<Utc>>,

    /// Current monthly vote count.
    pub monthly_votes: i64,

    /// Number of servers the bot is in.
    pub server_count: Option<i64>,

    /// Total vote count.
    pub total_votes: i64,

    /// Rank by monthly votes.
    pub monthly_votes_rank: i64,

    /// Rank by server count.
    pub server_count_rank: Option<i64>,

    /// Rank by total votes.
    pub total_votes_rank: i64,

    /// Last update timestamp.
    pub timestamp: Option<DateTime<Utc>>,

    /// Unix timestamp of last update (as string).
    pub unix_timestamp: Option<String>,
}

/// Response from the user bots endpoint.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct UserBotsResponse {
    /// List of bots owned by the user.
    pub bots: Vec<UserBot>,
}

impl UserBotsResponse {
    /// Returns the number of bots owned by the user.
    #[must_use]
    pub fn count(&self) -> usize {
        self.bots.len()
    }

    /// Returns `true` if the user has no bots.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bots.is_empty()
    }

    /// Returns an iterator over the bots.
    pub fn iter(&self) -> impl Iterator<Item = &UserBot> {
        self.bots.iter()
    }

    /// Returns the total monthly votes across all bots.
    #[must_use]
    pub fn total_monthly_votes(&self) -> i64 {
        self.bots.iter().map(|b| b.monthly_votes).sum()
    }

    /// Returns the total votes across all bots.
    #[must_use]
    pub fn total_votes(&self) -> i64 {
        self.bots.iter().map(|b| b.total_votes).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_bot_deserialization() {
        let json = r#"{
            "id": "461521980492087297",
            "owners": ["205680187394752512"],
            "deleted": false,
            "name": "Shiro",
            "avatar": "https://cdn.discordapp.com/avatars/461521980492087297/abc.png",
            "short_desc": "A multipurpose bot",
            "lib": "discord.js",
            "prefix": "s!",
            "website": "https://shirobot.org",
            "approved_at": "2018-09-22T11:25:10.962Z",
            "monthly_votes": 3,
            "server_count": 17762,
            "total_votes": 61387,
            "monthly_votes_rank": 7556,
            "server_count_rank": 501,
            "total_votes_rank": 282,
            "timestamp": "2024-10-28T18:00:00.000Z",
            "unix_timestamp": "1730138400000"
        }"#;

        let bot: UserBot = serde_json::from_str(json).unwrap();
        assert_eq!(bot.id, 461_521_980_492_087_297);
        assert_eq!(bot.name, "Shiro");
        assert_eq!(bot.owners.len(), 1);
        assert_eq!(bot.owners[0], 205_680_187_394_752_512);
        assert!(!bot.deleted);
        assert_eq!(bot.server_count, Some(17762));
    }

    #[test]
    fn test_user_bots_response_deserialization() {
        let json = r#"{
            "bots": [
                {
                    "id": "461521980492087297",
                    "owners": ["205680187394752512"],
                    "deleted": false,
                    "name": "Shiro",
                    "short_desc": "A multipurpose bot",
                    "prefix": "s!",
                    "monthly_votes": 3,
                    "total_votes": 61387,
                    "monthly_votes_rank": 7556,
                    "total_votes_rank": 282
                },
                {
                    "id": "583807014896140293",
                    "owners": ["205680187394752512"],
                    "deleted": false,
                    "name": "TopStats",
                    "short_desc": "Statistics bot",
                    "prefix": "/",
                    "monthly_votes": 10,
                    "total_votes": 500,
                    "monthly_votes_rank": 5000,
                    "total_votes_rank": 4000
                }
            ]
        }"#;

        let response: UserBotsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.count(), 2);
        assert!(!response.is_empty());
        assert_eq!(response.total_monthly_votes(), 13);
        assert_eq!(response.total_votes(), 61887);
    }

    #[test]
    fn test_empty_user_bots_response() {
        let response = UserBotsResponse { bots: vec![] };
        assert!(response.is_empty());
        assert_eq!(response.count(), 0);
        assert_eq!(response.total_monthly_votes(), 0);
    }
}

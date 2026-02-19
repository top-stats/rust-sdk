//! Shared endpoint logic and helper functions.
//!
//! This module contains validation functions and response types shared
//! between the async and blocking clients.

use crate::error::{Error, Result};
use crate::models::{Bot, RankedBot};

/// Validates a Discord bot ID format (17-19 digit snowflake).
pub fn validate_bot_id(id: &str) -> Result<()> {
    if Bot::validate_id(id) {
        Ok(())
    } else {
        Err(Error::InvalidBotId(id.to_string()))
    }
}

/// Validates a Discord user ID format (17-19 digit snowflake).
pub fn validate_user_id(id: &str) -> Result<()> {
    if Bot::validate_id(id) {
        Ok(())
    } else {
        Err(Error::InvalidUserId(id.to_string()))
    }
}

/// Response wrapper for tag search.
#[derive(serde::Deserialize)]
pub struct TagResponse {
    /// Tag data containing results.
    pub data: TagData,
}

/// Tag data containing bot results.
#[derive(serde::Deserialize)]
pub struct TagData {
    /// List of bots matching the tag.
    pub results: Vec<Bot>,
}

/// Response wrapper for bot comparison.
#[derive(serde::Deserialize)]
pub struct CompareResponse {
    /// List of compared bots.
    pub data: Vec<RankedBot>,
}

/// Builds query parameters for rankings requests.
#[must_use]
pub fn build_rankings_params(query: &crate::RankingsQuery) -> Vec<(&'static str, String)> {
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

    params
}

/// Builds query parameters for search requests.
#[must_use]
pub fn build_search_params(
    query: &str,
    limit: Option<u32>,
    offset: Option<u32>,
    include_deleted: Option<bool>,
) -> Vec<(&'static str, String)> {
    let mut params: Vec<(&str, String)> = vec![("query", query.to_string())];

    if let Some(limit) = limit {
        params.push(("limit", limit.to_string()));
    }
    if let Some(offset) = offset {
        params.push(("offset", offset.to_string()));
    }
    if let Some(include_deleted) = include_deleted {
        params.push(("includeDeleted", include_deleted.to_string()));
    }

    params
}

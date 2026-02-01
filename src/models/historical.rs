//! Historical data models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Time frame for historical data queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TimeFrame {
    /// All available historical data.
    #[default]
    #[serde(rename = "alltime")]
    AllTime,
    /// Last 5 years.
    #[serde(rename = "5y")]
    FiveYears,
    /// Last 3 years.
    #[serde(rename = "3y")]
    ThreeYears,
    /// Last 1 year.
    #[serde(rename = "1y")]
    OneYear,
    /// Last 270 days (9 months).
    #[serde(rename = "270d")]
    NineMonths,
    /// Last 180 days (6 months).
    #[serde(rename = "180d")]
    SixMonths,
    /// Last 90 days (3 months).
    #[serde(rename = "90d")]
    NinetyDays,
    /// Last 30 days.
    #[serde(rename = "30d")]
    ThirtyDays,
    /// Last 7 days.
    #[serde(rename = "7d")]
    SevenDays,
    /// Last 3 days.
    #[serde(rename = "3d")]
    ThreeDays,
    /// Last 1 day.
    #[serde(rename = "1d")]
    OneDay,
    /// Last 12 hours.
    #[serde(rename = "12h")]
    TwelveHours,
    /// Last 6 hours.
    #[serde(rename = "6h")]
    SixHours,
}

impl TimeFrame {
    /// Returns the API query parameter value for this time frame.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AllTime => "alltime",
            Self::FiveYears => "5y",
            Self::ThreeYears => "3y",
            Self::OneYear => "1y",
            Self::NineMonths => "270d",
            Self::SixMonths => "180d",
            Self::NinetyDays => "90d",
            Self::ThirtyDays => "30d",
            Self::SevenDays => "7d",
            Self::ThreeDays => "3d",
            Self::OneDay => "1d",
            Self::TwelveHours => "12h",
            Self::SixHours => "6h",
        }
    }
}

impl fmt::Display for TimeFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Type of historical data to query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    /// Monthly vote count.
    #[default]
    MonthlyVotes,
    /// Total vote count.
    TotalVotes,
    /// Server count.
    ServerCount,
    /// Review count.
    ReviewCount,
}

impl DataType {
    /// Returns the API query parameter value for this data type.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MonthlyVotes => "monthly_votes",
            Self::TotalVotes => "total_votes",
            Self::ServerCount => "server_count",
            Self::ReviewCount => "review_count",
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A single historical data point.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoricalDataPoint {
    /// Timestamp of this data point.
    pub time: DateTime<Utc>,

    /// Bot ID this data belongs to.
    pub id: String,

    /// Monthly votes at this time (if requested).
    pub monthly_votes: Option<i64>,

    /// Total votes at this time (if requested).
    pub total_votes: Option<i64>,

    /// Server count at this time (if requested).
    pub server_count: Option<i64>,

    /// Review count at this time (if requested).
    pub review_count: Option<i64>,
}

impl HistoricalDataPoint {
    /// Returns the value for the specified data type.
    #[must_use]
    pub const fn value_for(&self, data_type: DataType) -> Option<i64> {
        match data_type {
            DataType::MonthlyVotes => self.monthly_votes,
            DataType::TotalVotes => self.total_votes,
            DataType::ServerCount => self.server_count,
            DataType::ReviewCount => self.review_count,
        }
    }
}

/// Response from the historical data endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoricalDataResponse {
    /// Array of historical data points.
    pub data: Vec<HistoricalDataPoint>,
}

/// Response from the compare historical endpoint.
/// Maps bot IDs to their historical data points.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompareHistoricalResponse {
    /// Map of bot ID to historical data points.
    pub data: std::collections::HashMap<String, Vec<HistoricalDataPoint>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_frame_as_str() {
        assert_eq!(TimeFrame::AllTime.as_str(), "alltime");
        assert_eq!(TimeFrame::ThirtyDays.as_str(), "30d");
        assert_eq!(TimeFrame::TwelveHours.as_str(), "12h");
    }

    #[test]
    fn test_data_type_as_str() {
        assert_eq!(DataType::MonthlyVotes.as_str(), "monthly_votes");
        assert_eq!(DataType::ServerCount.as_str(), "server_count");
    }

    #[test]
    fn test_historical_data_point_deserialization() {
        let json = r#"{
            "time": "2025-03-31T00:00:00.000Z",
            "id": "583807014896140293",
            "monthly_votes": 265,
            "total_votes": 1000,
            "server_count": 500
        }"#;

        let point: HistoricalDataPoint = serde_json::from_str(json).unwrap();
        assert_eq!(point.id, "583807014896140293");
        assert_eq!(point.monthly_votes, Some(265));
        assert_eq!(point.total_votes, Some(1000));
        assert_eq!(point.server_count, Some(500));
        assert_eq!(point.review_count, None);
    }

    #[test]
    fn test_historical_data_point_value_for() {
        let point = HistoricalDataPoint {
            time: Utc::now(),
            id: "123".to_string(),
            monthly_votes: Some(100),
            total_votes: Some(500),
            server_count: None,
            review_count: Some(10),
        };

        assert_eq!(point.value_for(DataType::MonthlyVotes), Some(100));
        assert_eq!(point.value_for(DataType::TotalVotes), Some(500));
        assert_eq!(point.value_for(DataType::ServerCount), None);
        assert_eq!(point.value_for(DataType::ReviewCount), Some(10));
    }

    #[test]
    fn test_historical_response_deserialization() {
        let json = r#"{
            "data": [
                {
                    "time": "2025-03-31T00:00:00.000Z",
                    "id": "583807014896140293",
                    "monthly_votes": 265
                }
            ]
        }"#;

        let response: HistoricalDataResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].monthly_votes, Some(265));
    }

    #[test]
    fn test_time_frame_default() {
        assert_eq!(TimeFrame::default(), TimeFrame::AllTime);
    }

    #[test]
    fn test_data_type_default() {
        assert_eq!(DataType::default(), DataType::MonthlyVotes);
    }
}

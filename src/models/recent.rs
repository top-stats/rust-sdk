//! Recent data models.

use chrono::{DateTime, Utc};
use serde::Deserialize;

/// A single data point in recent bot statistics.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RecentDataPoint {
    /// Timestamp of this data point.
    pub time: DateTime<Utc>,

    /// Monthly vote count at this time.
    pub monthly_votes: i64,

    /// Total vote count at this time.
    pub total_votes: i64,

    /// Server count at this time.
    pub server_count: Option<i64>,

    /// Review count at this time.
    pub review_count: Option<i64>,

    /// Change in monthly votes since previous data point.
    pub monthly_votes_change: Option<i64>,

    /// Percentage change in monthly votes.
    pub monthly_votes_change_perc: Option<f64>,

    /// Change in server count since previous data point.
    pub server_count_change: Option<i64>,

    /// Change in total votes since previous data point.
    pub total_votes_change: Option<i64>,

    /// Change in review count since previous data point.
    pub review_count_change: Option<i64>,
}

/// Response from the recent data endpoint.
///
/// Contains hourly data for the past 30 hours and daily data for the past month.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RecentDataResponse {
    /// Hourly statistics for the past 30 hours.
    #[serde(rename = "hourlyData")]
    pub hourly_data: Vec<RecentDataPoint>,

    /// Daily statistics for the past month.
    #[serde(rename = "dailyData")]
    pub daily_data: Vec<RecentDataPoint>,
}

impl RecentDataResponse {
    /// Returns the most recent hourly data point, if available.
    #[must_use]
    pub fn latest_hourly(&self) -> Option<&RecentDataPoint> {
        self.hourly_data.first()
    }

    /// Returns the most recent daily data point, if available.
    #[must_use]
    pub fn latest_daily(&self) -> Option<&RecentDataPoint> {
        self.daily_data.first()
    }

    /// Calculates the total change in monthly votes over the hourly period.
    #[must_use]
    pub fn total_hourly_votes_change(&self) -> i64 {
        self.hourly_data
            .iter()
            .filter_map(|p| p.monthly_votes_change)
            .sum()
    }

    /// Calculates the total change in monthly votes over the daily period.
    #[must_use]
    pub fn total_daily_votes_change(&self) -> i64 {
        self.daily_data
            .iter()
            .filter_map(|p| p.monthly_votes_change)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recent_data_point_deserialization() {
        let json = r#"{
            "time": "2024-10-18T18:00:00.000Z",
            "monthly_votes": 1800088,
            "total_votes": 204896149,
            "server_count": 3371839,
            "review_count": 10682,
            "monthly_votes_change": 7310,
            "monthly_votes_change_perc": 0.41,
            "server_count_change": 0,
            "total_votes_change": 5028,
            "review_count_change": 1050
        }"#;

        let point: RecentDataPoint = serde_json::from_str(json).unwrap();
        assert_eq!(point.monthly_votes, 1_800_088);
        assert_eq!(point.total_votes, 204_896_149);
        assert_eq!(point.server_count, Some(3_371_839));
        assert_eq!(point.monthly_votes_change, Some(7310));
        assert!((point.monthly_votes_change_perc.unwrap() - 0.41).abs() < f64::EPSILON);
    }

    #[test]
    fn test_recent_data_response_deserialization() {
        let json = r#"{
            "hourlyData": [
                {
                    "time": "2024-10-18T18:00:00.000Z",
                    "monthly_votes": 1800088,
                    "total_votes": 204896149,
                    "server_count": 3371839,
                    "monthly_votes_change": 7310
                }
            ],
            "dailyData": [
                {
                    "time": "2024-10-18T00:00:00.000Z",
                    "monthly_votes": 1800088,
                    "total_votes": 204896149,
                    "server_count": 3371839,
                    "monthly_votes_change": 110263
                }
            ]
        }"#;

        let response: RecentDataResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.hourly_data.len(), 1);
        assert_eq!(response.daily_data.len(), 1);
        assert_eq!(response.hourly_data[0].monthly_votes_change, Some(7310));
        assert_eq!(response.daily_data[0].monthly_votes_change, Some(110_263));
    }

    #[test]
    fn test_recent_data_response_helpers() {
        let response = RecentDataResponse {
            hourly_data: vec![
                RecentDataPoint {
                    time: Utc::now(),
                    monthly_votes: 100,
                    total_votes: 1000,
                    server_count: None,
                    review_count: None,
                    monthly_votes_change: Some(10),
                    monthly_votes_change_perc: None,
                    server_count_change: None,
                    total_votes_change: None,
                    review_count_change: None,
                },
                RecentDataPoint {
                    time: Utc::now(),
                    monthly_votes: 90,
                    total_votes: 990,
                    server_count: None,
                    review_count: None,
                    monthly_votes_change: Some(5),
                    monthly_votes_change_perc: None,
                    server_count_change: None,
                    total_votes_change: None,
                    review_count_change: None,
                },
            ],
            daily_data: vec![RecentDataPoint {
                time: Utc::now(),
                monthly_votes: 100,
                total_votes: 1000,
                server_count: None,
                review_count: None,
                monthly_votes_change: Some(50),
                monthly_votes_change_perc: None,
                server_count_change: None,
                total_votes_change: None,
                review_count_change: None,
            }],
        };

        assert_eq!(response.latest_hourly().unwrap().monthly_votes, 100);
        assert_eq!(response.latest_daily().unwrap().monthly_votes, 100);
        assert_eq!(response.total_hourly_votes_change(), 15);
        assert_eq!(response.total_daily_votes_change(), 50);
    }
}

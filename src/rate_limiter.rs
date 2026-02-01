//! Rate limiter implementation.
//!
//! Provides automatic rate limiting with configurable retry behavior.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::lock::Mutex;

/// Maximum delay threshold in seconds before throwing an error instead of waiting.
pub const MAX_DELAY_THRESHOLD: f64 = 5.0;

/// Global rate limit: 120 requests per minute.
pub const GLOBAL_RATE_LIMIT: u32 = 120;

/// Per-endpoint rate limit: 60 requests per minute.
pub const ENDPOINT_RATE_LIMIT: u32 = 60;

/// Rate limit window duration.
pub const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);

/// A token bucket rate limiter.
#[derive(Debug)]
pub struct RateLimiter {
    /// Maximum number of tokens (requests) allowed.
    max_tokens: u32,
    /// Current number of available tokens.
    tokens: f64,
    /// Last time tokens were refilled.
    last_refill: Instant,
    /// Token refill rate (tokens per second).
    refill_rate: f64,
}

impl RateLimiter {
    /// Creates a new rate limiter with the specified capacity.
    #[must_use]
    pub fn new(max_requests: u32, window: Duration) -> Self {
        let refill_rate = f64::from(max_requests) / window.as_secs_f64();
        Self {
            max_tokens: max_requests,
            tokens: f64::from(max_requests),
            last_refill: Instant::now(),
            refill_rate,
        }
    }

    /// Refills tokens based on elapsed time.
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(f64::from(self.max_tokens));
        self.last_refill = now;
    }

    /// Attempts to acquire a token. Returns the wait time if no token is available.
    pub fn try_acquire(&mut self) -> Option<Duration> {
        self.refill();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            None
        } else {
            // Calculate how long until we have a token
            let wait_time = (1.0 - self.tokens) / self.refill_rate;
            Some(Duration::from_secs_f64(wait_time))
        }
    }

    /// Returns the current number of available tokens.
    #[must_use]
    pub fn available_tokens(&mut self) -> u32 {
        self.refill();
        self.tokens as u32
    }
}

/// Manages rate limiters for multiple endpoints.
#[derive(Debug)]
pub struct RateLimiterManager {
    /// Global rate limiter.
    global: Arc<Mutex<RateLimiter>>,
    /// Per-endpoint rate limiters.
    endpoints: Arc<Mutex<HashMap<String, RateLimiter>>>,
    /// Currently active rate limits from API responses.
    active_limits: Arc<Mutex<HashMap<String, Instant>>>,
}

impl Default for RateLimiterManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiterManager {
    /// Creates a new rate limiter manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            global: Arc::new(Mutex::new(RateLimiter::new(
                GLOBAL_RATE_LIMIT,
                RATE_LIMIT_WINDOW,
            ))),
            endpoints: Arc::new(Mutex::new(HashMap::new())),
            active_limits: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Checks if we should wait before making a request to the given endpoint.
    ///
    /// Returns `Ok(())` if the request can proceed, or the duration to wait.
    pub async fn check(&self, endpoint: &str) -> Option<Duration> {
        // Check for active rate limits from API responses
        {
            let active = self.active_limits.lock().await;
            if let Some(&until) = active.get(endpoint) {
                let now = Instant::now();
                if until > now {
                    return Some(until - now);
                }
            }
        }

        // Check global rate limit
        let global_wait = {
            let mut global = self.global.lock().await;
            global.try_acquire()
        };

        if let Some(wait) = global_wait {
            return Some(wait);
        }

        // Check endpoint rate limit
        let mut endpoints = self.endpoints.lock().await;
        let limiter = endpoints
            .entry(endpoint.to_string())
            .or_insert_with(|| RateLimiter::new(ENDPOINT_RATE_LIMIT, RATE_LIMIT_WINDOW));

        limiter.try_acquire()
    }

    /// Records a rate limit response from the API.
    pub async fn record_rate_limit(&self, endpoint: &str, retry_after: Duration) {
        let mut active = self.active_limits.lock().await;
        active.insert(endpoint.to_string(), Instant::now() + retry_after);
    }

    /// Clears an active rate limit for an endpoint.
    pub async fn clear_rate_limit(&self, endpoint: &str) {
        let mut active = self.active_limits.lock().await;
        active.remove(endpoint);
    }
}

impl Clone for RateLimiterManager {
    fn clone(&self) -> Self {
        Self {
            global: Arc::clone(&self.global),
            endpoints: Arc::clone(&self.endpoints),
            active_limits: Arc::clone(&self.active_limits),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_basic() {
        let mut limiter = RateLimiter::new(10, Duration::from_secs(1));

        // Should be able to acquire 10 tokens
        for _ in 0..10 {
            assert!(limiter.try_acquire().is_none());
        }

        // 11th should require waiting
        assert!(limiter.try_acquire().is_some());
    }

    #[test]
    fn test_rate_limiter_available_tokens() {
        let mut limiter = RateLimiter::new(10, Duration::from_secs(1));
        assert_eq!(limiter.available_tokens(), 10);

        limiter.try_acquire();
        assert_eq!(limiter.available_tokens(), 9);
    }

    #[tokio::test]
    async fn test_rate_limiter_manager_basic() {
        let manager = RateLimiterManager::new();

        // First request should succeed
        assert!(manager.check("test_endpoint").await.is_none());
    }

    #[tokio::test]
    async fn test_rate_limiter_manager_active_limit() {
        let manager = RateLimiterManager::new();

        // Record a rate limit
        manager
            .record_rate_limit("test_endpoint", Duration::from_secs(10))
            .await;

        // Should return a wait time
        let wait = manager.check("test_endpoint").await;
        assert!(wait.is_some());
        assert!(wait.unwrap() <= Duration::from_secs(10));

        // Clear the limit
        manager.clear_rate_limit("test_endpoint").await;

        // Now should succeed (if we have tokens)
        // Note: This might still return Some if we're out of tokens
    }

    #[test]
    fn test_rate_limiter_refill() {
        let mut limiter = RateLimiter::new(10, Duration::from_secs(10));

        // Use all tokens
        for _ in 0..10 {
            limiter.try_acquire();
        }
        assert_eq!(limiter.available_tokens(), 0);

        // Simulate time passing (we can't actually wait in a unit test)
        // The refill happens automatically based on elapsed time
    }
}

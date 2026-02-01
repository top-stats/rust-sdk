# TopStats Rust SDK

A Rust SDK for the [TopStats.gg API](https://topstats.gg), providing statistics for Discord bots listed on Top.gg.

[![Crates.io](https://img.shields.io/crates/v/topstats.svg)](https://crates.io/crates/topstats)
[![Documentation](https://docs.rs/topstats/badge.svg)](https://docs.rs/topstats)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **Async or blocking** - Choose at compile time via feature flags
- **Multiple HTTP backends**: reqwest (async) or ureq (blocking)
- **Built-in rate limiting** with automatic retry for short delays
- **Type-safe** models with serde serialization
- **Tracing** support for logging (optional)

## Installation

### Async (default)

```toml
[dependencies]
topstats = "0.1"
```

### Blocking

```toml
[dependencies]
topstats = { version = "0.1", default-features = false, features = ["blocking", "ureq-client"] }
```

## Quick Start

### Async Usage (default)

```rust
use topstats::{Client, RankingsQuery, SortBy};

#[tokio::main]
async fn main() -> Result<(), topstats::Error> {
    let client = Client::new("your-api-token")?;
    
    // Get bot information
    let bot = client.get_bot("432610292342587392").await?;
    println!("Bot: {} has {} monthly votes", bot.name, bot.monthly_votes);
    
    // Get rankings
    let rankings = client
        .get_rankings(RankingsQuery::new().sort_by(SortBy::MonthlyVotes).limit(10))
        .await?;
    
    for bot in &rankings.data {
        println!("#{}: {} - {} votes", bot.monthly_votes_rank, bot.name, bot.monthly_votes);
    }
    
    Ok(())
}
```

### Blocking Usage

```rust
use topstats::Client;

fn main() -> Result<(), topstats::Error> {
    let client = Client::new("your-api-token")?;
    
    let bot = client.get_bot("432610292342587392")?;
    println!("Bot: {} has {} monthly votes", bot.name, bot.monthly_votes);
    
    Ok(())
}
```

## API Coverage

### Bot Endpoints

- `get_bot(bot_id)` - Get bot information
- `get_bot_historical(bot_id, time_frame, data_type)` - Get historical data
- `get_bot_recent(bot_id)` - Get recent statistics (hourly/daily)

### Rankings

- `get_rankings(query)` - Get bot rankings with sorting and pagination

### Search

- `search_bots(query, limit, offset)` - Search bots by name
- `search_by_tag(tag, limit, offset)` - Search bots by tag

### Compare

- `compare_bots(bot_ids)` - Compare 2-4 bots
- `compare_bots_historical(bot_ids, time_frame, data_type)` - Compare historical data

### Users

- `get_user_bots(user_id)` - Get bots owned by a user

## Configuration

### Client Builder

```rust
use topstats::Client;

let client = Client::builder()
    .token("your-api-token")
    .base_url("https://api.topstats.gg")  // Optional: custom base URL
    .auto_retry(true)                       // Optional: auto-retry on rate limits
    .max_delay_threshold(5.0)               // Optional: max seconds to wait before error
    .build()?;
```

### Time Frames

```rust
use topstats::TimeFrame;

TimeFrame::AllTime      // All available data
TimeFrame::FiveYears    // Last 5 years
TimeFrame::OneYear      // Last 1 year
TimeFrame::NinetyDays   // Last 90 days
TimeFrame::ThirtyDays   // Last 30 days
TimeFrame::SevenDays    // Last 7 days
TimeFrame::OneDay       // Last 24 hours
TimeFrame::TwelveHours  // Last 12 hours
TimeFrame::SixHours     // Last 6 hours
```

### Data Types

```rust
use topstats::DataType;

DataType::MonthlyVotes  // Monthly vote count
DataType::TotalVotes    // Total votes
DataType::ServerCount   // Server count
DataType::ReviewCount   // Review count
```

### Sort Options

```rust
use topstats::{RankingsQuery, SortBy, SortOrder};

let query = RankingsQuery::new()
    .sort_by(SortBy::MonthlyVotes)
    .sort_order(SortOrder::Ascending)
    .limit(100)
    .offset(0);
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `async` | Yes | Enable async mode (requires `reqwest-client`) |
| `blocking` | No | Enable blocking mode (requires `ureq-client`) |
| `reqwest-client` | Yes | Use reqwest as HTTP backend (async) |
| `ureq-client` | No | Use ureq as HTTP backend (blocking) |
| `rustls-tls` | Yes | Use rustls for TLS |
| `native-tls` | No | Use native TLS implementation |
| `tracing` | No | Enable tracing/logging support |

**Note:** `async` and `blocking` are mutually exclusive. The crate compiles as either async or blocking, not both.

## Error Handling

```rust
use topstats::{Client, Error};

let client = Client::new("your-token")?;

match client.get_bot("invalid-id").await {  // Remove .await for blocking mode
    Ok(bot) => println!("Found: {}", bot.name),
    Err(Error::InvalidBotId(id)) => println!("Invalid bot ID: {}", id),
    Err(Error::NotFound { message }) => println!("Bot not found: {}", message),
    Err(Error::RateLimited { retry_after, .. }) => {
        println!("Rate limited, retry after {}s", retry_after);
    }
    Err(e) => println!("Error: {}", e),
}
```

## Rate Limiting

The SDK includes built-in rate limiting that:
- Automatically waits for short delays (< 10 seconds by default)
- Throws `Error::RateLimited` for longer delays

You can configure this behavior:

```rust
let client = Client::builder()
    .token("your-token")
    .auto_retry(false)           // Disable auto-retry
    .max_delay_threshold(10.0)   // Wait up to 10 seconds
    .build()?;
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Links

- [TopStats.gg](https://topstats.gg)
- [API Documentation](https://docs.topstats.gg)
- [GitHub Repository](https://github.com/top-stats/rust-sdk)

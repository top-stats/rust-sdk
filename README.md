# TopStats Rust SDK

Rust SDK for the [TopStats.gg API](https://topstats.gg) - Discord bot statistics from Top.gg.

## Installation

```toml
[dependencies]
topstats = "0.1.0"
```

For blocking mode:

```toml
[dependencies]
topstats = { version = "0.1.0", default-features = false, features = ["blocking", "ureq-client"] }
```

## Usage

```rust
use topstats::{Client, RankingsQuery, SortBy};

#[tokio::main]
async fn main() -> Result<(), topstats::Error> {
    let client = Client::new("your-api-token")?;

    // Get bot information
    let bot = client.get_bot("432610292342587392").await?;
    println!("{} has {} monthly votes", bot.name, bot.monthly_votes);

    // Get rankings
    let rankings = client
        .get_rankings(RankingsQuery::new().sort_by(SortBy::MonthlyVotes).limit(10))
        .await?;

    for bot in &rankings.data {
        println!("#{}: {}", bot.monthly_votes_rank, bot.name);
    }

    Ok(())
}
```

## Links

- [API Documentation](https://docs.topstats.gg)
- [TopStats.gg](https://topstats.gg)

## License

MIT

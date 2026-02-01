//! Blocking usage example for the TopStats SDK.
//!
//! Run with:
//! ```sh
//! TOPSTATS_TOKEN=your_token cargo run --example blocking_usage --no-default-features --features "blocking,ureq-client"
//! ```

use topstats::{Client, DataType, RankingsQuery, SortBy, TimeFrame};

fn main() -> Result<(), topstats::Error> {
    // Get token from environment
    let token =
        std::env::var("TOPSTATS_TOKEN").expect("TOPSTATS_TOKEN environment variable not set");

    // Create client
    let client = Client::new(token)?;

    // Get bot information
    println!("=== Getting Bot Info ===");
    let bot = client.get_bot("432610292342587392")?;
    println!("Bot: {} (ID: {})", bot.name, bot.id);
    println!("  Monthly votes: {}", bot.monthly_votes);
    println!("  Total votes: {}", bot.total_votes);
    println!("  Server count: {:?}", bot.server_count);
    println!("  Rank: #{}", bot.monthly_votes_rank);

    // Get historical data
    println!("\n=== Historical Data (Last 30 Days) ===");
    let history = client.get_bot_historical(
        "432610292342587392",
        TimeFrame::ThirtyDays,
        DataType::MonthlyVotes,
    )?;
    println!("Got {} data points", history.data.len());
    if let Some(first) = history.data.first() {
        println!(
            "  Latest: {} votes at {}",
            first.monthly_votes.unwrap_or(0),
            first.time
        );
    }

    // Get rankings
    println!("\n=== Top 5 Bots by Monthly Votes ===");
    let rankings =
        client.get_rankings(RankingsQuery::new().sort_by(SortBy::MonthlyVotes).limit(5))?;
    println!("Total bots tracked: {}", rankings.total_bot_count);
    for bot in &rankings.data {
        println!(
            "  #{}: {} - {} monthly votes",
            bot.monthly_votes_rank, bot.name, bot.monthly_votes
        );
    }

    // Compare bots
    println!("\n=== Comparing Two Bots ===");
    let comparison = client.compare_bots(&["432610292342587392", "646937666251915264"])?;
    for bot in &comparison {
        println!(
            "  {}: {} monthly votes (rank #{})",
            bot.name, bot.monthly_votes, bot.monthly_votes_rank
        );
    }

    println!("\nDone!");
    Ok(())
}

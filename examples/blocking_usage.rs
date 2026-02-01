//! Blocking usage example for the TopStats SDK.
//!
//! Run with:
//! ```sh
//! TOPSTATS_TOKEN=your_token cargo run --example blocking_usage --features blocking,ureq-client
//! ```

#[cfg(feature = "blocking")]
fn main() -> Result<(), topstats::Error> {
    use topstats::blocking::Client;
    use topstats::{RankingsQuery, SortBy};

    // Get token from environment
    let token =
        std::env::var("TOPSTATS_TOKEN").expect("TOPSTATS_TOKEN environment variable not set");

    // Create blocking client
    let client = Client::new(token)?;

    // Get bot information
    println!("=== Getting Bot Info ===");
    let bot = client.get_bot("432610292342587392")?;
    println!("Bot: {} (ID: {})", bot.name, bot.id);
    println!("  Monthly votes: {}", bot.monthly_votes);
    println!("  Total votes: {}", bot.total_votes);

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

    println!("\nDone!");
    Ok(())
}

#[cfg(not(feature = "blocking"))]
fn main() {
    println!("This example requires the 'blocking' and 'ureq-client' features.");
    println!("Run with: cargo run --example blocking_usage --features blocking,ureq-client");
}

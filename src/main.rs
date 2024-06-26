use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::{self, Write};
use tokio::time::{sleep, Duration};
use thiserror::Error;

#[derive(Deserialize, Debug)]
struct CoinGeckoPrice {
    usd: f64,
}

#[derive(Error, Debug)]
enum FetchError {
    #[error("Request error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Failed to parse price")]
    ParseError,
}

async fn fetch_prices(ticker: &str) -> Result<CoinGeckoPrice, FetchError> {
    let api_url = format!("https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd", ticker);
    let response = reqwest::get(&api_url).await?.json::<serde_json::Value>().await?;

    let price = response
        .get(ticker)
        .and_then(|c| c.get("usd"))
        .and_then(|usd| usd.as_f64())
        .ok_or(FetchError::ParseError)?;

    Ok(CoinGeckoPrice { usd: price })
}

fn prompt_user(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn prompt_for_f64(prompt: &str) -> f64 {
    loop {
        let input = prompt_user(prompt);
        match input.parse::<f64>() {
            Ok(value) => return value,
            Err(_) => println!("Invalid input. Please enter a valid number."),
        }
    }
}

fn get_valid_ticker() -> String {
    let valid_tickers: HashMap<&str, &str> = [
        ("btc", "bitcoin"),
        ("bitcoin", "bitcoin"),
        ("eth", "ethereum"),
        ("ethereum", "ethereum"),
        ("ada", "cardano"),
        ("cardano", "cardano"),
    ]
        .iter()
        .cloned()
        .collect();

    loop {
        let ticker = prompt_user("Enter the cryptocurrency ticker (e.g., btc, eth, ada): ").to_lowercase();
        if let Some(valid_ticker) = valid_tickers.get(ticker.as_str()) {
            return valid_ticker.to_string();
        } else {
            println!("Invalid ticker. Please enter one of the following: btc, eth, ada.");
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = get_valid_ticker();
    let alert_type = prompt_user("Do you want to set an alert based on (1) $ change or (2) % change? Enter 1 or 2: ");
    let threshold = prompt_for_f64("Enter the threshold value: ");

    let initial_price = fetch_prices(&ticker).await?.usd;

    println!("Monitoring {} price. Initial price: ${:.2}", ticker, initial_price);

    loop {
        sleep(Duration::from_secs(30)).await;

        match fetch_prices(&ticker).await {
            Ok(current_price) => {
                println!("Current {} price: ${:.2}", ticker, current_price.usd);
                let price_change = current_price.usd - initial_price;
                let percent_change = (price_change / initial_price) * 100.0;

                match alert_type.as_str() {
                    "1" => {
                        if price_change.abs() >= threshold {
                            println!(
                                "Alert! {} price changed by ${:.2}. Current price: ${:.2}",
                                ticker, price_change, current_price.usd
                            );
                        }
                    }
                    "2" => {
                        if percent_change.abs() >= threshold {
                            println!(
                                "Alert! {} price changed by {:.2}%. Current price: ${:.2}",
                                ticker, percent_change, current_price.usd
                            );
                        }
                    }
                    _ => println!("Invalid alert type."),
                }
            }
            Err(e) => println!("Error fetching prices: {}", e),
        }
    }
}
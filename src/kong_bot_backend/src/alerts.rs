//! # Cryptocurrency Price Alert System
//! 
//! This canister provides functionality to:
//! - Set price alerts for cryptocurrencies
//! - Check current prices from CoinGecko API
//! - Send notifications when price conditions are met
//! - Store alert data in stable storage



// ===== Constants =====
use ic_cdk::{
    api::management_canister::http_request::{
        http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse,
        TransformArgs, TransformContext,
    },
    query, update, storage, init, api,
};
use serde_json::Value;
use candid::{CandidType, Deserialize};
use std::collections::HashMap;
use crate::trading::Portfolio;






// ===== Data Structures =====

/// Alert configuration for a specific cryptocurrency
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Alert {
    /// User identifier (typically an OpenChat principal or username)
    user: String,
    /// Cryptocurrency identifier (e.g., "bitcoin", "ethereum", "internet-computer")
    coin: String,
    /// Target price in USD that triggers the alert
    target_price: f64,
}

/// Type alias for the alerts storage map
type Alerts = HashMap<String, Alert>;

/// Tracks price history for cryptocurrencies
#[derive(Clone, Debug, CandidType, Deserialize)]
struct PriceHistory {
    /// Last recorded price in USD
    last_price: f64,
}

/// Type alias for the price history storage map
type PriceMap = HashMap<String, PriceHistory>;

// ===== Storage Management =====

/// Initialize the canister with empty storage
#[init]
fn init() {
    let _ = storage::stable_save((
        HashMap::<String, Alert>::new(),
        HashMap::<String, PriceHistory>::new(),
    ))
    .map_err(|e| api::print(format!("❌ Failed to initialize storage: {}", e)));
}

/// Load alerts from stable storage
fn load_alerts() -> Alerts {
    storage::stable_restore::<(Alerts, PriceMap)>()
        .map(|(alerts, _)| alerts)
        .unwrap_or_else(|e| {
            api::print(format!("⚠️ Failed to load alerts: {}", e));
            HashMap::new()
        })
}

/// Load price history from stable storage
fn load_price_history() -> PriceMap {
    storage::stable_restore::<(Alerts, PriceMap)>()
        .map(|(_, prices)| prices)
        .unwrap_or_else(|e| {
            api::print(format!("⚠️ Failed to load price history: {}", e));
            HashMap::new()
        })
}
/// Get the current price history for all tracked cryptocurrencies
/// 
/// # Returns
/// A map of cryptocurrency IDs to their last recorded prices
#[query]
fn get_price_history() -> PriceMap {
    load_price_history()
}

/// Save both alerts and price history to stable storage
fn save_state(alerts: &Alerts, prices: &PriceMap) -> Result<(), String> {
    storage::stable_save((alerts.clone(), prices.clone()))
        .map_err(|e| format!("Failed to save state: {}", e))
}

// ===== Public API Methods =====

/// Set a price alert for a specific cryptocurrency
/// 
/// # Parameters
/// * `user` - User identifier (typically an OpenChat principal or username)
/// * `coin` - Cryptocurrency identifier (e.g., "bitcoin", "ethereum")
/// * `target_price` - Target price in USD that triggers the alert
/// 
/// # Returns
/// A confirmation message or error message
#[update]
fn set_alert(user: String, coin: String, target_price: f64) -> String {
    let mut alerts = load_alerts();
    let key = format!("{}_{}", user, coin);
    
    alerts.insert(
        key.clone(),
        Alert {
            user,
            coin: coin.clone(),
            target_price,
        },
    );

    let prices = load_price_history();
    match save_state(&alerts, &prices) {
        Ok(_) => format!("✅ Alert set for {} when {} reaches ${:.2}", key, coin, target_price),
        Err(e) => format!("❌ Failed to save alert: {}", e),
    }
}

/// Get all registered alerts
/// 
/// # Returns
/// A map of all alerts in the system
#[query]
fn get_alerts() -> Alerts {
    load_alerts()
}

/// Check all alerts against current prices and send notifications if needed
#[update]
async fn check_alerts() {
    api::print(format!("Starting to check alerts..."));
    let alerts = load_alerts();
    api::print(format!("Loaded {} alerts", alerts.len()));
    let mut price_history = load_price_history();
    let mut updated = false;

    for (key, alert) in alerts.iter() {
        api::print(format!("Checking alert for {}: coin={}, target=${}", key, alert.coin, alert.target_price));
        
        // Convert coin name to proper CoinGecko ID
        let coin_id = match alert.coin.to_lowercase().as_str() {
            "btc" => "bitcoin",
            "eth" => "ethereum",
            "icp" => "internet-computer",
            "sol" => "solana",
            // Add more mappings as needed
            _ => &alert.coin, // Use as-is if no mapping exists
        };
        
        api::print(format!("Using CoinGecko ID: {}", coin_id));
        
        match get_crypto_price(coin_id).await {
            Ok(current_price) => {
                api::print(format!("✅ Got price for {}: ${:.4}", coin_id, current_price));
                
                let prev_price = price_history
                    .get(coin_id)
                    .map(|p| p.last_price)
                    .unwrap_or(current_price);
                
                api::print(format!("Previous price: ${:.4}", prev_price));

                let coingecko_url = format!("https://www.coingecko.com/en/coins/{}", coin_id);
                let message = if current_price > prev_price {
                    format!(
                        "🚀 Your coin **{}** has gained value! Current price: **${:.4}** (was ${:.4}). Do you want to trade? [Trade Now]({})",
                        alert.coin, current_price, prev_price, coingecko_url
                    )
                } else if current_price < prev_price {
                    format!(
                        "📉 Your coin **{}** has lost value. Current price: **${:.4}** (was ${:.4}). Consider taking action. [Check Market]({})",
                        alert.coin, current_price, prev_price, coingecko_url
                    )
                } else {
                    format!(
                        "ℹ No change in {} price. Current price: ${:.4}. [View on CoinGecko]({})", 
                        alert.coin, current_price, coingecko_url
                    )
                };

                // Check if target price is reached
                let target_message = if prev_price < alert.target_price && current_price >= alert.target_price {
                    Some(format!(
                        "🎯 Target price alert! **{}** has reached your target of ${:.4}. Current price: ${:.4}. [View on CoinGecko]({})",
                        alert.coin, alert.target_price, current_price, coingecko_url
                    ))
                } else if prev_price > alert.target_price && current_price <= alert.target_price {
                    Some(format!(
                        "🎯 Target price alert! **{}** has dropped to your target of ${:.4}. Current price: ${:.4}. [View on CoinGecko]({})",
                        alert.coin, alert.target_price, current_price, coingecko_url
                    ))
                } else {
                    None
                };

                // Send target price message if applicable
                if let Some(target_msg) = target_message {
                    api::print(format!("Sending target price alert: {}", target_msg));
                    send_openchat_message(&alert.user, &target_msg).await;
                }

                // Send regular price update message
                api::print(format!("Sending price update: {}", message));
                send_openchat_message(&alert.user, &message).await;
                
                // Update price history
                price_history.insert(coin_id.to_string(), PriceHistory { last_price: current_price });
                updated = true;
                api::print(format!("Updated price history for {}", coin_id));
            }
            Err(e) => api::print(format!("❌ Error fetching price for {}: {}", coin_id, e)),
        }
    }

    if updated {
        match save_state(&alerts, &price_history) {
            Ok(_) => api::print(format!("✅ Successfully saved price history")),
            Err(e) => api::print(format!("❌ Failed to save price history: {}", e)),
        }
    } else {
        api::print(format!("ℹ No price updates to save"));
    }
}

/// Get the current price of Internet Computer (ICP) token
/// 
/// # Returns
/// A string with the current ICP price or an error message
#[update]
async fn get_icp_price() -> String {
    let host = "api.coingecko.com";
    let url = format!(
        "https://{}/api/v3/coins/internet-computer?localization=false&tickers=false&market_data=true&community_data=false&developer_data=false&sparkline=false",
        host,
    );

    // Prepare headers for the HTTP request
    let req_headers = vec![
        HttpHeader {
            name: "Accept".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "User-Agent".to_string(),
            value: "IC-Canister".to_string(),
        },
    ];
            
    let req = CanisterHttpRequestArgument {
        url: url.to_string(),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(2_000_000), // Set to 2MB (the maximum allowed)
        transform: Some(TransformContext::from_name("transform".to_string(), vec![])),
        headers: req_headers,
    };

    // Cycles required for the HTTP request
    let cycles: u128 = 1_603_146_400 + 10_000;

    // Make the HTTPS request and wait for response
    match http_request(req, cycles).await {
        Ok((response,)) => {
            let str_body = match String::from_utf8(response.body) {
                Ok(body) => body,
                Err(_) => return "Error: Response body is not valid UTF-8".to_string(),
            };
            
            match serde_json::from_str::<Value>(&str_body) {
                Ok(json) => {
                    if let Some(price) = json["market_data"]["current_price"]["usd"].as_f64() {
                        return format!("ICP Price: ${:.4}", price);
                    } else {
                        "Price data not found in the response".to_string()
                    }
                }
                Err(e) => format!("Invalid JSON response: {}", e),
            }
        }
        Err((r, m)) => format!("HTTP request failed. RejectionCode: {:?}, Error: {}", r, m),
    }
}

/// Transform HTTP response headers for security
#[query]
fn transform(raw: TransformArgs) -> HttpResponse {
    // Add security headers to the response
    let security_headers = vec![
        HttpHeader {
            name: "Content-Security-Policy".to_string(),
            value: "default-src 'self'".to_string(),
        },
        HttpHeader {
            name: "Referrer-Policy".to_string(),
            value: "strict-origin".to_string(),
        },
        HttpHeader {
            name: "Permissions-Policy".to_string(),
            value: "geolocation=(self)".to_string(),
        },
        HttpHeader {
            name: "Strict-Transport-Security".to_string(),
            value: "max-age=63072000".to_string(),
        },
        HttpHeader {
            name: "X-Frame-Options".to_string(),
            value: "DENY".to_string(),
        },
        HttpHeader {
            name: "X-Content-Type-Options".to_string(),
            value: "nosniff".to_string(),
        },
    ];

    let mut res = HttpResponse {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers: security_headers,
        ..Default::default()
    };

    // Only keep the original body if status is 200 OK
    if res.status == candid::Nat::from(200u64) {
        res.body = raw.response.body;
    } else {
        api::print(
            format!("Received an error from CoinGecko: Status: {}, Body: {:?}", 
                    res.status, raw.response.body)
        );
    }
    res
}

// ===== Helper Functions =====

/// Fetch the current price of a cryptocurrency from CoinGecko
async fn get_crypto_price(coin_id: &str) -> Result<f64, String> {
    let host = "api.coingecko.com";
    let url = format!(
        "https://{}/api/v3/coins/{}?localization=false&tickers=false&market_data=true&community_data=false&developer_data=false&sparkline=false",
        host, coin_id
    );

    let req_headers = vec![
        HttpHeader {
            name: "Accept".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "User-Agent".to_string(),
            value: "IC-Canister".to_string(),
        },
    ];
            
    let req = CanisterHttpRequestArgument {
        url,
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(2_000_000), // 2MB max response
        transform: Some(TransformContext::from_name("transform".to_string(), vec![])),
        headers: req_headers,
    };

    let cycles: u128 = 1_603_146_400 + 10_000; // Add buffer to ensure enough cycles

    match http_request(req, cycles).await {
        Ok((response,)) => {
            if response.status != candid::Nat::from(200u64) {
                let error_body = String::from_utf8(response.body.clone())
                    .unwrap_or_else(|_| format!("Non-UTF8 error response: {:?}", response.body));
                return Err(format!("API error (status {}): {}", response.status, error_body));
            }
            
            let str_body = String::from_utf8(response.body)
                .map_err(|e| format!("Invalid UTF-8 in response: {}", e))?;
            
            let json: Value = serde_json::from_str(&str_body)
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;
            
            json["market_data"]["current_price"]["usd"]
                .as_f64()
                .ok_or_else(|| "Price data not found in response".to_string())
        }
        Err((r, m)) => Err(format!("HTTP request failed. RejectionCode: {:?}, Error: {}", r, m)),
    }
}

/// Get the current price of any cryptocurrency supported by CoinGecko
/// 
/// # Parameters
/// * `coin_id` - The CoinGecko ID of the cryptocurrency (e.g., "bitcoin", "ethereum", "internet-computer")
/// 
/// # Returns
/// The current price as a string with USD symbol or an error message
#[update]
async fn get_crypto_price_api(coin_id: String) -> String {
    match get_crypto_price(&coin_id).await {
        Ok(price) => format!("${:.4}", price),
        Err(e) => format!("Error: {}", e)
    }
}

/// Send a message to a user via OpenChat
async fn send_openchat_message(user_id: &str, message: &str) {
    // This is a placeholder for the actual implementation
    // In a real implementation, you would make an HTTP request to OpenChat's API
    api::print(format!("📨 Sending message to {}: {}", user_id, message));
    
    // TODO: Implement actual OpenChat integration
    // Example implementation would look like:
    // let openchat_url = "https://api.openchat.com/messages";
    // let request_body = json!({
    //     "recipient": user_id,
    //     "message": message,
    //     "format": "markdown"
    // }).to_string().into_bytes();
    // 
    // let req = CanisterHttpRequestArgument {
    //     url: openchat_url.to_string(),
    //     method: HttpMethod::POST,
    //     body: Some(request_body),
    //     ...
    // };
    // 
    // match http_request(req, cycles).await {
    //     ...
    // }
}

// Generate Candid interface
ic_cdk::export_candid!();

//! # Cryptocurrency Trading Module
//! 
//! This module provides functionality for simulated cryptocurrency trading:
//! - Get current market prices from CoinGecko
//! - Manage user portfolios and balances
//! - Execute buy and sell orders
//! - Track transaction history

use candid::{CandidType, Deserialize};
use ic_cdk::{
    api::management_canister::http_request::{
        http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, TransformContext,
    },
    api, storage, update, query,
};
use serde_json::Value;
use std::collections::HashMap;

// ===== Data Structures =====

/// Represents a user's portfolio of cryptocurrencies
#[derive(Clone, Debug, CandidType, Deserialize, Default)]
pub struct Portfolio {
    /// User's USD balance
    pub usd_balance: f64,
    /// Map of cryptocurrency ID to amount owned
    pub holdings: HashMap<String, f64>,
    /// Transaction history
    pub transactions: Vec<Transaction>,
}

/// Represents a buy or sell transaction
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Transaction {
    /// Transaction type (buy or sell)
    pub transaction_type: TransactionType,
    /// Cryptocurrency ID
    pub coin_id: String,
    /// Amount of cryptocurrency
    pub amount: f64,
    /// Price per unit in USD
    pub price: f64,
    /// Total value of transaction in USD
    pub total_value: f64,
    /// Timestamp of transaction
    pub timestamp: u64,
}

/// Type of transaction
#[derive(Clone, Debug, CandidType, Deserialize, PartialEq)]
pub enum TransactionType {
    Buy,
    Sell,
}

/// Type alias for user portfolios storage
type Portfolios = HashMap<String, Portfolio>;

// ===== Storage Management =====

/// Load user portfolios from stable storage
fn load_portfolios() -> Portfolios {
    match storage::stable_restore::<(Portfolios,)>() {
        Ok((portfolios,)) => portfolios,
        Err(e) => {
            api::print(format!("⚠️ Failed to load portfolios: {}", e));
            HashMap::new()
        }
    }
}

/// Save user portfolios to stable storage
fn save_portfolios(portfolios: &Portfolios) -> Result<(), String> {
    storage::stable_save((portfolios.clone(),))
        .map_err(|e| format!("Failed to save portfolios: {}", e))
}

// ===== Public API Methods =====

/// Initialize a user's portfolio with a starting USD balance
/// 
/// # Parameters
/// * `user_id` - User identifier
/// * `initial_balance` - Starting USD balance (default: 10000.0)
/// 
/// # Returns
/// A confirmation message
#[update]
pub async fn initialize_portfolio(user_id: String, initial_balance: Option<f64>) -> String {
    let mut portfolios = load_portfolios();
    
    if portfolios.contains_key(&user_id) {
        return format!("Portfolio for user {} already exists", user_id);
    }
    
    let balance = initial_balance.unwrap_or(10000.0);
    
    let portfolio = Portfolio {
        usd_balance: balance,
        holdings: HashMap::new(),
        transactions: Vec::new(),
    };
    
    portfolios.insert(user_id.clone(), portfolio);
    
    match save_portfolios(&portfolios) {
        Ok(_) => format!("Portfolio initialized for {} with ${:.2} USD", user_id, balance),
        Err(e) => format!("Failed to initialize portfolio: {}", e),
    }
}

/// Get a user's portfolio
/// 
/// # Parameters
/// * `user_id` - User identifier
/// 
/// # Returns
/// The user's portfolio or an error message
#[query]
pub fn get_portfolio(user_id: String) -> Result<Portfolio, String> {
    let portfolios = load_portfolios();
    
    portfolios.get(&user_id)
        .cloned()
        .ok_or_else(|| format!("Portfolio not found for user {}", user_id))
}

/// Buy cryptocurrency
/// 
/// # Parameters
/// * `user_id` - User identifier
/// * `coin_id` - CoinGecko cryptocurrency ID
/// * `amount_usd` - Amount in USD to spend
/// 
/// # Returns
/// A confirmation message or error
#[update]
pub async fn buy_cryptocurrency(user_id: String, coin_id: String, amount_usd: f64) -> String {
    if amount_usd <= 0.0 {
        return "Amount must be greater than zero".to_string();
    }
    
    let mut portfolios = load_portfolios();
    
    // Check if user exists
    let portfolio = match portfolios.get_mut(&user_id) {
        Some(p) => p,
        None => return format!("Portfolio not found for user {}", user_id),
    };
    
    // Check if user has enough USD
    if portfolio.usd_balance < amount_usd {
        return format!("Insufficient USD balance. You have ${:.2}, but need ${:.2}", 
                      portfolio.usd_balance, amount_usd);
    }
    
    // Get current price from CoinGecko
    let normalized_id = normalize_coin_id(&coin_id);
    
    match get_crypto_price(&normalized_id).await {
        Ok(price) => {
            // Calculate amount of crypto to buy
            let crypto_amount = amount_usd / price;
            
            // Update portfolio
            portfolio.usd_balance -= amount_usd;
            
            *portfolio.holdings.entry(normalized_id.clone()).or_insert(0.0) += crypto_amount;
            
            // Record transaction
            let transaction = Transaction {
                transaction_type: TransactionType::Buy,
                coin_id: normalized_id.clone(),
                amount: crypto_amount,
                price,
                total_value: amount_usd,
                timestamp: api::time(),
            };
            
            portfolio.transactions.push(transaction);
            
            // Save updated portfolios
            match save_portfolios(&portfolios) {
                Ok(_) => format!("Successfully bought {:.6} {} for ${:.2} USD", 
                                crypto_amount, normalized_id, amount_usd),
                Err(e) => format!("Transaction recorded but failed to save: {}", e),
            }
        },
        Err(e) => format!("Failed to get price for {}: {}", normalized_id, e),
    }
}

/// Sell cryptocurrency
/// 
/// # Parameters
/// * `user_id` - User identifier
/// * `coin_id` - CoinGecko cryptocurrency ID
/// * `crypto_amount` - Amount of cryptocurrency to sell
/// 
/// # Returns
/// A confirmation message or error
#[update]
pub async fn sell_cryptocurrency(user_id: String, coin_id: String, crypto_amount: f64) -> String {
    if crypto_amount <= 0.0 {
        return "Amount must be greater than zero".to_string();
    }
    
    let mut portfolios = load_portfolios();
    
    // Check if user exists
    let portfolio = match portfolios.get_mut(&user_id) {
        Some(p) => p,
        None => return format!("Portfolio not found for user {}", user_id),
    };
    
    let normalized_id = normalize_coin_id(&coin_id);
    
    // Check if user has enough of the cryptocurrency
    let user_crypto_amount = portfolio.holdings.get(&normalized_id).cloned().unwrap_or(0.0);
    
    if user_crypto_amount < crypto_amount {
        return format!("Insufficient {} balance. You have {:.6}, but want to sell {:.6}", 
                      normalized_id, user_crypto_amount, crypto_amount);
    }
    
    // Get current price from CoinGecko
    match get_crypto_price(&normalized_id).await {
        Ok(price) => {
            // Calculate USD value
            let usd_value = crypto_amount * price;
            
            // Update portfolio
            portfolio.usd_balance += usd_value;
            
            if let Some(holding) = portfolio.holdings.get_mut(&normalized_id) {
                *holding -= crypto_amount;
                
                // Remove the entry if balance is zero or very close to zero
                if *holding < 0.000001 {
                    portfolio.holdings.remove(&normalized_id);
                }
            }
            
            // Record transaction
            let transaction = Transaction {
                transaction_type: TransactionType::Sell,
                coin_id: normalized_id.clone(),
                amount: crypto_amount,
                price,
                total_value: usd_value,
                timestamp: api::time(),
            };
            
            portfolio.transactions.push(transaction);
            
            // Save updated portfolios
            match save_portfolios(&portfolios) {
                Ok(_) => format!("Successfully sold {:.6} {} for ${:.2} USD", 
                                crypto_amount, normalized_id, usd_value),
                Err(e) => format!("Transaction recorded but failed to save: {}", e),
            }
        },
        Err(e) => format!("Failed to get price for {}: {}", normalized_id, e),
    }
}

/// Get current portfolio value in USD
/// 
/// # Parameters
/// * `user_id` - User identifier
/// 
/// # Returns
/// Total portfolio value or error
#[update]
pub async fn get_portfolio_value(user_id: String) -> Result<f64, String> {
    let portfolios = load_portfolios();
    
    let portfolio = match portfolios.get(&user_id) {
        Some(p) => p,
        None => return Err(format!("Portfolio not found for user {}", user_id)),
    };
    
    let mut total_value = portfolio.usd_balance;
    
    // Calculate value of each cryptocurrency holding
    for (coin_id, amount) in &portfolio.holdings {
        match get_crypto_price(coin_id).await {
            Ok(price) => {
                total_value += amount * price;
            },
            Err(e) => {
                api::print(format!("Failed to get price for {}: {}", coin_id, e));
                // Continue with other coins even if one fails
            }
        }
    }
    
    Ok(total_value)
}

/// Get list of supported cryptocurrencies
#[query]
pub fn get_supported_cryptocurrencies() -> Vec<String> {
    vec![
        "bitcoin".to_string(),
        "ethereum".to_string(),
        "internet-computer".to_string(),
        "solana".to_string(),
        "cardano".to_string(),
        "polkadot".to_string(),
        "binancecoin".to_string(),
        "ripple".to_string(),
        "dogecoin".to_string(),
        "shiba-inu".to_string(),
    ]
}

// ===== Helper Functions =====

/// Normalize coin ID to match CoinGecko format
fn normalize_coin_id(coin_id: &str) -> String {
    match coin_id.to_lowercase().as_str() {
        "btc" => "bitcoin",
        "eth" => "ethereum",
        "icp" => "internet-computer",
        "sol" => "solana",
        "ada" => "cardano",
        "dot" => "polkadot",
        "bnb" => "binancecoin",
        "xrp" => "ripple",
        "doge" => "dogecoin",
        "shib" => "shiba-inu",
        other => other,
    }.to_string()
}

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

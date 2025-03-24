// use ic_cdk::{
//     api::management_canister::http_request::{
//         http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
//          TransformContext,
//     },
//      update,
// };
// use candid::{CandidType, Deserialize};
// use serde_json::Value;


// #[derive(CandidType, Deserialize)]
// struct CoinPrice {
//     coin: String,
//     price: f64,
// }

// /// Fetch the current price of a cryptocurrency from CoinGecko
// async fn get_crypto_price(coin_id: &str) -> Result<f64, String> {
//     let host = "api.coingecko.com";
//     let url = format!(
//         "https://{}/api/v3/coins/{}?localization=false&tickers=false&market_data=true&community_data=false&developer_data=false&sparkline=false",
//         host, coin_id
//     );

//     let req_headers = vec![
//         HttpHeader {
//             name: "Accept".to_string(),
//             value: "application/json".to_string(),
//         },
//         HttpHeader {
//             name: "User-Agent".to_string(),
//             value: "IC-Canister".to_string(),
//         },
//     ];
            
//     let req = CanisterHttpRequestArgument {
//         url,
//         method: HttpMethod::GET,
//         body: None,
//         max_response_bytes: Some(2_000_000), // 2MB max response
//         transform: Some(TransformContext::from_name("transform".to_string(), vec![])),
//         headers: req_headers,
//     };

//     let cycles: u128 = 1_603_146_400 + 10_000; // Add buffer to ensure enough cycles

//     match http_request(req, cycles).await {
//         Ok((response,)) => {
//             if response.status != candid::Nat::from(200u64) {
//                 let error_body = String::from_utf8(response.body.clone())
//                     .unwrap_or_else(|_| format!("Non-UTF8 error response: {:?}", response.body));
//                 return Err(format!("API error (status {}): {}", response.status, error_body));
//             }
            
//             let str_body = String::from_utf8(response.body)
//                 .map_err(|e| format!("Invalid UTF-8 in response: {}", e))?;
            
//             let json: Value = serde_json::from_str(&str_body)
//                 .map_err(|e| format!("Failed to parse JSON: {}", e))?;
            
//             json["market_data"]["current_price"]["usd"]
//                 .as_f64()
//                 .ok_or_else(|| "Price data not found in response".to_string())
//         }
//         Err((r, m)) => Err(format!("HTTP request failed. RejectionCode: {:?}, Error: {}", r, m)),
//     }
// }

// /// Get the current price of any cryptocurrency supported by CoinGecko
// /// 
// /// # Parameters
// /// * `coin_id` - The CoinGecko ID of the cryptocurrency (e.g., "bitcoin", "ethereum", "internet-computer")
// /// 
// /// # Returns
// /// The current price as a string with USD symbol or an error message
// #[update]
// async fn get_crypto_price_api(coin_id: String) -> String {
//     match get_crypto_price(&coin_id).await {
//         Ok(price) => format!("${:.4}", price),
//         Err(e) => format!("Error: {}", e)
//     }
// }
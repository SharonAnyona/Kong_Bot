# Kong Bot

## Overview
Kong Bot is a trading bot built on the **Internet Computer** that runs in **OpenChat**. It fetches cryptocurrency prices from **CoinGecko**, allows users to set **price alerts**, and provides **buy/sell** functionality upon user confirmation.

## Features
- **Buy & Sell**: Users can confirm transactions before execution.
- **Manage Wallet**: Store balances & track transactions.
- **Refer Friends**: Invite others and earn rewards.
- **Price Alerts**: Get notified when a coin reaches a set price.
- **Authentication**: Open to anyone with an **Internet Identity (II)**.

## Tech Stack
- **Backend**: Rust (ic-cdk, reqwest, serde)
- **Frontend**: React (TypeScript, Dfinity agent)
- **Database**: Stable storage in Internet Computer canister
- **API**: CoinGecko for price data

## Installation & Setup
### **1. Clone the Repository**
```sh
 git clone https://github.com/your-repo/kong-bot.git
 cd kong-bot
```

### **2. Install Dependencies**
#### **Backend (Rust Canister)**
```sh
cd kong_bot_backend
cargo build
```
#### **Frontend (React UI)**
```sh
cd kong_bot_frontend
npm install
```

### **3. Start Local Development Environment**
```sh
dfx start --background
```

### **4. Deploy Canister to Internet Computer**
```sh
dfx deploy
```

## API Endpoints
### **1. Fetch Coin Price**
```sh
GET /api/get_coin_price?coin=bitcoin
```
_Response:_
```json
{
  "coin": "bitcoin",
  "price": 65000.0
}
```

### **2. Set Price Alert**
```sh
POST /api/set_price_alert
{
  "user": "@username",
  "coin": "bitcoin",
  "target_price": 70000.0
}
```

### **3. Generate Referral Code**
```sh
GET /api/generate_referral_code
```
_Response:_
```json
{
  "referral_code": "8F9D2A3C"
}
```

## Deployment
1. **Verify Internet Identity Setup**
2. **Deploy to IC Mainnet**
```sh
dfx deploy --network ic
```
3. **Monitor logs**
```sh
dfx canister log kong_bot
```

## Future Enhancements
- **Multi-Chain Trading**
- **AI-based Trade Suggestions**

## License
MIT



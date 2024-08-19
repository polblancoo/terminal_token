// src/api_calls/tradingview.rs
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TradingviewPrice {
    // Estructura de datos según la respuesta de TradingView
    // Ejemplo:
   pub price: f64,
   pub timestamp: String,
   pub interval: String,
}

pub async fn fetch_prices_from_tradingview(token: &str) -> Result<Vec<TradingviewPrice>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!("https://api.tradingview.com/api/v1/price?symbol={}", token);
    let response = client.get(&url).send().await?.json::<TradingviewPrice>().await?;
    
    // Aquí deberías convertir la respuesta en un Vec<TradingviewPrice>
    Ok(vec![response])
}
//pub mod coingecko;
//pub mod tradingview;
use crate::llamadasApi::coingecko::fetch_prices_from_coingecko;
use crate::llamadasApi::tradingview::fetch_prices_from_tradingview;
use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct PriceData {
   pub price: f64,
    pub timestamp: String,
    pub interval: String,
}

pub async fn fetch_prices(token: &str) -> Result<Vec<PriceData>, Box<dyn std::error::Error>> {
     // Leer la configuración
    // let config = crate::myconfig::leer_config_obtencion_datos()?;
    let config="coingecko";
     let mut all_prices = Vec::new();
 
     // Consultar CoinGecko si está habilitado en la configuración
     if config=="coingecko" {
         let coingecko_prices = fetch_prices_from_coingecko(token).await?;
         for price_data in coingecko_prices {
             all_prices.push(PriceData {
                 price: price_data.price,
                 timestamp: price_data.timestamp,
                 interval: price_data.interval,
             });
         }
     }
 
     // Consultar TradingView si está habilitado en la configuración
     if config=="tradingview" {
         let tradingview_prices = fetch_prices_from_tradingview(token).await?;
         for price_data in tradingview_prices {
             all_prices.push(PriceData {
                 price: price_data.price,
                 timestamp: price_data.timestamp,
                 interval: price_data.interval,
             });
         }
     }
 
     Ok(all_prices) 
}

use reqwest::Client;
use std::{error::Error, io}; 
use reqwest::Request;
use serde::{Deserialize, Serialize};
use serde_json::Value;


#[derive(Debug, Deserialize)]
pub struct CoinGeckoResponse {
    // Define los campos de la respuesta de la API de CoinGecko que necesitas
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub current_price: f64,
}

#[derive(Debug, Deserialize)]
pub struct CoingeckoPrice {
    // Estructura de datos según la respuesta de CoinGecko
    // Ejemplo:
   pub price: f64,
    pub timestamp: String,
    pub interval: String,
}

#[derive(Deserialize,Serialize, Debug)]
#[derive(Clone)]
pub struct Coin_price{
    usd: f64,
    usd_market_cap: f64,
    usd_24h_vol: f64,
    usd_24h_change: f64,
    last_updated_at: String
}

#[derive(Deserialize,Serialize, Debug)]
#[derive(Clone)]
pub struct Coin{
  pub id: String,
  pub symbol: String,
  pub name: String,
  pub image: String,
  pub current_price: f64,
  pub market_cap: u64,
  pub market_cap_rank: u32,
  pub fully_diluted_valuation: Option<u64>,
  pub total_volume: u64,
  pub high_24h: f64,
  pub low_24h: f64,
  pub price_change_24h: f64,
  pub price_change_percentage_24h: f64,
  pub market_cap_change_24h: f64,
  pub market_cap_change_percentage_24h: f64,
  pub circulating_supply: f64,
  pub total_supply: f64,
  pub max_supply: Option<f64>,
  pub ath: f64,
  pub ath_change_percentage: f64,
  pub ath_date: String,
  pub atl: f64,
  pub atl_change_percentage: f64,
  pub atl_date: String,
  pub roi: Option<serde_json::Value>,
  pub last_updated: String,  
}

#[derive(Debug, Deserialize)]
pub struct HistoricalPrice {
    pub prices: Vec<(f64, f64)>, // (timestamp, price)
}


fn concat_url(consulta :&str ,coin: Vec<&str>, api_key01 : &str)->String{
  //Constructing de list of coin
      
  let inicio = "vs_currency=usd&ids=";
  let coin_str = concat_vector_to_string(coin , inicio);
  //println!("{}", coin_str);
  let url = match consulta {
    "all" => format!("https://api.coingecko.com/api/v3/coins/markets?{coin_str}&x_cg_demo_api_key={api_key01}"),
    "price" =>format!("https://api.coingecko.com/api/v3/simple/price?{coin_str}&x_cg_demo_api_key={api_key01}"),
    _ => format!("Caso no reconocido: {}", consulta)
  };
  url
}

fn concat_vector_to_string(vec: Vec<&str>, inicio : &str)-> String{
  let mut resultado = inicio.to_string();
 // let inicio = "=vs_currency=usd&ids=";
  let separador ="%2C";

  resultado.push_str(&vec[0]);
  
        for elemento in &vec[1..] {
            resultado.push_str(separador);
            resultado.push_str(elemento);
        }
       resultado 
}


pub async fn get_coin_data(coin_id: &str) -> Result<CoinGeckoResponse, reqwest::Error> {
    let url = format!("https://api.coingecko.com/api/v3/coins/{}", coin_id);
    let client = Client::new();
    let response = client.get(&url).send().await?.json::<CoinGeckoResponse>().await?;
    Ok(response)
}
 pub async fn fetch_prices_from_coingecko(token: &str) -> Result<Vec<CoingeckoPrice>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!("https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd", token);
    let response = client.get(&url).send().await?.json::<CoingeckoPrice>().await?;
    
    // Aquí deberías convertir la respuesta en un Vec<CoingeckoPrice>
    Ok(vec![response])
} 

pub async fn get_coins_list_full( coin: Vec<&str>, api_key01 : &str)->Result< Vec<Coin> , reqwest::Error>{

    
    //println!("{:#?}",url.bright_blue());

    let url = concat_url("all",coin,api_key01);
    
    // Sending a blocking GET request to the API endpoint
    let response = reqwest::get(&url).await?;
      
    // Parsing the JSON response into WeatherResponse struct
    let response_json : Vec<Coin>= response.json().await?;
    
   //println!("{:?}", &response_json);

    Ok(response_json) // Returning the deserialized response
  }


  pub async fn consulta_api_de_precios(token: &str, start_time: u64, end_time: u64, api_key: &str) -> Result<HistoricalPrice, Box<dyn std::error::Error + Send + Sync>> {
    
    let client = Client::new();
    //let url = format!("https://api.coingecko.com/api/v3/coins/{}/market_chart?vs_currency=usd&from={}&to={}&api_key={}", token, start_time, end_time, api_key);
    let url = format!("https://api.coingecko.com/api/v3/coins/{token}/market_chart?vs_currency=usd&days=30");
    
    let response = client
        .get(url)
        .send()
        .await?;
    let json_response = response.json::<serde_json::Value>().await?;
    let historical_price: HistoricalPrice = serde_json::from_value(json_response)?;
    Ok(historical_price)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio; // Ensure you have tokio in your dependencies for async tests

    #[tokio::test]
    async fn test_consulta_api_de_precios() {
        let token = "bitcoin"; // Replace with a valid token
        let start_time = 1609459200; // Example start time (2021-01-01)
        let end_time = 1609545600; // Example end time (2021-01-02)
          let APIKEY="YOU_KEY";

        let result = consulta_api_de_precios(token, start_time, end_time, &APIKEY).await;

        match result {
            Ok(data) => {
                // Assert that the data is as expected
                assert!(!data.prices.is_empty(), "Prices should not be empty");
            },
            Err(e) => panic!("Error fetching prices: {:?}", e),
        }
    }
}

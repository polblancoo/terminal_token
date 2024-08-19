use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, ErrorKind, Write};

use inquire::error;

use crate::ConsultaCrypto;


#[derive(Debug)]
pub struct TokenData {
    pub token: String,
    pub data: Vec<(f64, f64)>,
}


// Guarda las criptomonedas de ConsultaCrypto en un archivo
pub fn save_crypto_to_file(consulta: &ConsultaCrypto, filename: &str) -> io::Result<()> {
    let mut file = File::create(filename)?;
    let data = consulta.crypto_list.join(",");
    file.write_all(data.as_bytes())?;
    Ok(())
}

// Lee las criptomonedas de un archivo y las carga en una instancia de ConsultaCrypto
pub fn load_crypto_from_file(filename: &str) -> io::Result<ConsultaCrypto> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut crypto_list = Vec::new();

    for line in reader.lines() {
        let line = line?;
        crypto_list = line.split(',').map(|s| s.trim().to_string()).collect();
    }

    Ok(ConsultaCrypto { crypto_list })
}

// Lee un listado de tokens habilitados desde un archivo
pub fn load_token_list(filename: &str) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut tokens = Vec::new();

    for line in reader.lines() {
        let line = line?;
        tokens = line.split(',').map(|s| s.trim().to_string()).collect();
    }

    Ok(tokens)
}

pub fn process_and_save_data_token(token: &str, data: Vec<(f64, f64)>)  {
    let normalized_data: Vec<(f64, f64)> = data.into_iter()
        .map(|(timestamp, price)| (timestamp, (price * 1000.0).round() / 1000.0))
        .collect();

    let filename = format!("{}_data.txt", token);
    let mut file = File::create(&filename);
    let mut file = File::create(&filename).expect("Unable to create file");
    for (timestamp, price) in normalized_data.iter() {
        writeln!(file, "{:.3}\t{:.3}", timestamp, price).expect("Unable to write to file");
    }
}
pub fn load_data(token: &str) -> Result<TokenData, Box<dyn Error>> {
    // let filename = format!("{}_data.txt", token.to_lowercase());
   //  let filename =  format!("/home/pablo/Rust-Curso/Ejemplos_Mios/Coingeko_API/{}", filename);
    let filename = "/home/pablo/Rust-Curso/Ejemplos_Mios/Coingeko_API/bitcoin_data.txt";
     let file = File::open(&filename)?;
     let reader = BufReader::new(file);
 
     let mut data = Vec::new();
 
     for line in reader.lines() {
         let line = line?;
         let mut parts = line.split_whitespace();
         let timestamp: f64 = parts.next().ok_or("Missing timestamp")?.parse()?;
         let price: f64 = parts.next().ok_or("Missing price")?.parse()?;
         data.push((timestamp, price));
     }
 
     Ok(TokenData {
         token: token.to_string(),
         data,
     })
 }
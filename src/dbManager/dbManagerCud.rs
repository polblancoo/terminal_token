use std::error::Error;

use chrono::{DateTime,TimeZone, Utc}; // Para obtener la hora actual en UTC
use rusqlite::{params, Connection, OptionalExtension, Result};

use crate::ConsultaCrypto;
use crate::llamadasApi::{self, coingecko};


// Define una estructura para almacenar los detalles del token
#[derive(Clone, Debug)] 
pub struct TokenData {
    pub name: String,
    pub symbol: String,
    pub current_price: f64,
    pub market_cap: f64,
    pub total_suply: f64,
    pub max_suply: f64,
pub circulating_suply: f64
}


pub fn get_token_prices(db_conn: &Connection, token_name: &str) -> Vec<(f64, f64)> {
    let mut stmt = db_conn.prepare(
        "SELECT timestamp, price FROM prices
         INNER JOIN tokens ON prices.token_id = tokens.id
         WHERE tokens.name = ?1
         ORDER BY timestamp ASC",
    ).expect("Failed to prepare statement");

    let price_iter = stmt
        .query_map([token_name], |row| {
            let timestamp: String = row.get(0)?;
            let price: f64 = row.get(1)?;

            // Convertir timestamp a un formato adecuado (por ejemplo, segundos desde la época UNIX)
            let timestamp_float = convert_timestamp_to_f64(&timestamp);

            Ok((timestamp_float, price))
        })
        .expect("Failed to map query");

    let mut prices = Vec::new();
    for price in price_iter {
        prices.push(price.expect("Failed to retrieve price"));
    }

    prices
}

// Función auxiliar para convertir un timestamp en string a f64
fn convert_timestamp_to_f64(timestamp: &str) -> f64 {
    let parsed_time = chrono::NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S")
        .expect("Failed to parse timestamp");
    let duration_since_epoch = parsed_time.and_utc().timestamp() as f64;
    duration_since_epoch
}

pub fn get_token_data(db_conn: &Connection, token_name: &str) -> Result<Option<TokenData>, Box<dyn Error>> {
    let mut stmt = db_conn.prepare(
    "SELECT name, symbol, current_price, market_cap, total_suply, max_suply, circulating_suply FROM tokens WHERE name = ?1",
    )?;

    let mut token_iter = stmt.query_map(params![token_name.trim()], |row| {
        Ok(TokenData {
            name: row.get(0)?,
            symbol: row.get(1)?,
            current_price: row.get(2)?,
            market_cap: row.get(3)?,
            total_suply: row.get(4)?,
            max_suply: row.get (5)?,
            circulating_suply: row.get (6)?,
        })
    })?;
    if let Some(token) = token_iter.next() {
        return Ok(Some(token?));
    }
   
    Ok(None) // Si no se encuentra el token
}
// Insertar un nuevo token
pub fn insert_token(conn: &Connection, name: &str, symbol: &str, current_price : f64 ,market_price: f64 , total_suply : f64, max_suply : f64, circulation_suply : f64) -> Result<usize> {
    conn.execute(
        "INSERT INTO tokens (name, symbol,current_price,market_cap,total_suply,max_suply,circulating_suply) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7  )",
        params![name, symbol, current_price, market_price, total_suply, max_suply, circulation_suply],
    )
}

pub fn get_tokens_from_db(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT name FROM tokens")?;
    let tokens = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        Ok(name)
    })?
    .collect::<Result<Vec<String>, _>>()?;

    Ok(tokens)
}

// Insertar un nuevo precio
pub fn insert_price(conn: &Connection, token_name: String, price: f64, timestamp: &str, interval: &str ) -> Result<usize> {
   //let price : f32= price as f32;
    conn.execute(
        "INSERT INTO prices (token_id, price, timestamp, interval) VALUES (?1, ?2, ?3, ?4)",
        params![token_name, price, timestamp, interval],
    )
}

// Obtener todos los tokens
pub fn get_tokens(conn: &Connection) -> Result<Vec<(String)>> {
    let mut stmt = conn.prepare("SELECT name FROM tokens")?;
    let tokens = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        Ok(name)
    })?.collect::<Result<Vec<String>, _>>()?;
    
    Ok(tokens)
}

// Obtener precios por token_id y opción de intervalo
pub fn get_prices(conn: &Connection, token_id: i32, interval: Option<&str>) -> Result<Vec<(i32, f64, String, String)>> {
    let mut query = "SELECT id, price, timestamp, interval FROM prices WHERE token_id = ?1".to_string();
    
    let params = if let Some(interval_filter) = interval {
        query.push_str(" AND interval = ?2");
        params![token_id, interval_filter.to_string()]
    } else {
        params![token_id]
    };

    let mut stmt = conn.prepare(&query)?;
    let prices = stmt.query_map(params, |row| {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
        ))
    })?.collect::<Result<_, _>>()?;
    
    Ok(prices)
}
/// Verifica si un token está en la base de datos.
pub fn is_token_in_db(conn: &Connection, token_name: &str) -> Result<bool> {
    let token_name = token_name.trim();
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM tokens WHERE name = ?1")?;
    let count: i64 = stmt.query_row(&[token_name], |row| row.get(0))?;
    Ok(count > 0)
}

/// Recorre el listado de tokens y actualiza la base de datos.
pub fn update_tokens_in_db_solo_nombre(conn: &Connection, consulta: &ConsultaCrypto) -> Result<()> {
    for name in &consulta.crypto_list {
        if !is_token_in_db(conn, name)? {
            // Llamada a la API o alguna otra lógica para obtener información del token
            // Por ahora, solo insertamos el token como ejemplo
           // insert_token(conn, token, "SYM",0.0,0.0,0.0,0.0,0.0)?; // "SYM" es un símbolo de ejemplo; deberías reemplazarlo con el valor real
            /* conn.execute(
                "INSERT INTO tokens (name, symbol) VALUES (?1, ?2)",
                params![token, "symbol"],
            )?; */
            conn.execute(
                "INSERT INTO tokens (name, symbol,current_price,market_cap,total_suply,max_suply,circulating_suply) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7  )",
                params![name.trim(), "symbol", 0.0, 0.0, 0.0, 0.0, 0.0],
            )?;
       }
    }
    Ok(())
}
//
pub fn update_tokens_masparametros_in_db(conn: &Connection, name: &str, symbol: &str, current_price: f64, market_price: f64, total_suply: f64, max_suply: f64, circulation_suply: f64) -> Result<(), rusqlite::Error> {
    if is_token_in_db(conn, name)? {
        // Llamada a la API o alguna otra lógica para obtener información del token
        // Por ahora, solo insertamos el token como ejemplo
        conn.execute(
            "UPDATE tokens SET symbol = ?2, current_price = ?3, market_cap = ?4, total_suply = ?5, max_suply = ?6, circulating_suply = ?7 WHERE name = ?1",
            params![name, symbol, current_price, market_price, total_suply, max_suply, circulation_suply],
        )?;
    
    }
    Ok(())
}
  
// Eliminar un token por su name
pub fn delete_token(conn: &Connection, token_name: String) -> Result<usize> {
    let token_name = token_name.trim();
    // Primero, eliminar los precios asociados al token
    conn.execute(
        "DELETE FROM prices WHERE name = ?1",
        params![token_name],
    )?;
    
    // Luego, eliminar el token
    conn.execute(
        "DELETE FROM tokens WHERE name = ?1",
        params![token_name],
    )
}

// Eliminar un precio por su ID
pub fn delete_price(conn: &Connection, price_id: i32) -> Result<usize> {
    conn.execute(
        "DELETE FROM prices WHERE id = ?1",
        params![price_id],
    )
}

pub async fn update_database_token_prices(conn: &Connection, token_name: &str, token_id: i32, symbol: &str, current_price: f64) -> Result<()> {
   let  token_name = token_name.trim();
    // Llamada asíncrona a fetch_prices
    let prices = llamadasApi::llamadasApi::fetch_prices(token_name).await;
    let prices = prices.unwrap();
    for price_data in prices.iter() {
        let interval = price_data.interval.as_str();
        let price = price_data.price;
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // Verificar si ya existe un precio para este token
        let existing_price: Option<f64> = conn.query_row(
            "SELECT price FROM prices WHERE token_id = ?1 AND interval = ?2 AND timestamp = ?3",
            params![token_id, interval, timestamp],
            |row| row.get(0),
        )?; // .optional() devuelve un Result<Option<T>, E>

        if let Some(_) = existing_price {
            // Actualizar el precio existente
            conn.execute(
                "UPDATE prices SET price = ?1 WHERE token_id = ?2 AND interval = ?3 AND timestamp = ?4",
                params![price, token_id, interval, timestamp],
            )?;
        } else {
            // Insertar un nuevo precio
            conn.execute(
                "INSERT INTO prices (token_id, price, timestamp, interval) VALUES (?1, ?2, ?3, ?4)",
                params![token_id, price, timestamp, interval],
            )?;
        }
    }

    Ok(())
}

pub fn get_last_update_timestamp_prices_token(conn: &Connection, token_name: String) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT timestamp FROM prices WHERE name = ?1 ORDER BY timestamp DESC LIMIT 1")?;
    
    let result = stmt.query_row(params![token_name], |row| row.get(0)).optional()?;
    
    Ok(result)
}

//obtiene datos prices para graficar 
pub fn get_prices_time_to_vec(
    conn: &Connection,
    token_id: &str,
    start_time: &DateTime<Utc>,
    end_time: &DateTime<Utc>,
) -> Result<Vec<(DateTime<Utc>, f64)>> {
    // Prepara la consulta SQL
    let mut stmt = conn.prepare(
        "SELECT timestamp, price FROM prices 
         WHERE token_id = ?1 AND timestamp >= ?2 AND timestamp <= ?3"
    )?;

    // Convierte las fechas a timestamps para la consulta
    let start_time_ts = start_time.timestamp();
    let end_time_ts = end_time.timestamp();

    // Ejecuta la consulta y mapea los resultados a un vector
    let prices_iter = stmt.query_map(
        params![token_id, start_time_ts, end_time_ts],
        |row| {
            let timestamp: i64 = row.get(0)?;
            let price: f64 = row.get(1)?;
            Ok((timestamp, price))
        },
    )?;

    // Colecta los resultados en un vector, filtrando los datos inválidos
    let prices: Vec<(DateTime<Utc>, f64)> = prices_iter
        .filter_map(|result| {
            result.ok().and_then(|(timestamp, price)| {
                Utc.timestamp_opt(timestamp, 0)
                    .single()
                    .map(|datetime| (datetime, price))
            })
        })
        .collect();

    Ok(prices)
}
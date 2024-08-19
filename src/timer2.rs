use core::time;
use std::fmt::Debug;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc};
use colorize::AnsiColor;
use tokio::sync::Mutex as TokioMutex;
use std::time::Duration;
use tokio::time::sleep;


use chrono::{ Utc};

use rusqlite::Connection;
use crate::coingecko::consulta_api_de_precios;
use crate::dbManagerCud::{self, insert_token};
use crate::{coingecko, App, ConsultaCrypto};
use crate::dbManager::dbManagerCud::get_tokens_from_db;
use crate::dbManager::dbManagerCud::{update_tokens_in_db_solo_nombre,update_tokens_masparametros_in_db};

use crate::myconfig;


pub async fn start_timer(conn: Arc<TokioMutex<Connection>>, consulta: ConsultaCrypto,tx: mpsc::Sender<String>) {
    let update_interval = Duration::from_secs(60 * 10); // Actualiza cada 10 minutos
    //let msg = "Start actualizando prices tokens (cada 5'). ".yellow();
    //tx.send( msg).unwrap();

    loop {
        // Clonar el Arc para usar en la tarea asincrónica
        let conn = Arc::clone(&conn);
        let consulta = consulta.clone();
        let tx = tx.clone(); // Clonamos el transmisor aquí

        tokio::spawn(async move {
            let conn = conn.lock().await; // Bloquear el mutex para obtener la conexión

            // Actualiza los tokens en la base de datos
            if let Err(e) = update_tokens_in_db_solo_nombre(&conn, &consulta) {
                eprintln!("Error actualizando tokens: {:?}", e);
            }
        });

        // Espera antes de la siguiente actualización
        sleep(update_interval).await;
    }
}
pub async fn start_timer_Info_tokens(conn: Arc<TokioMutex<Connection>>, tx: mpsc::Sender<String>) {
    let update_interval = Duration::from_secs(60 * 5); // Actualiza cada 5 minutos
    //let msg = "Start actualizando prices tokens (cada 5'). ".yellow();
    //tx.send( msg).unwrap();
 
    loop {
        // Obtener la lista de tokens desde la base de datos
        let tokens = {
            let conn_guard = conn.lock().await;
            let tx = tx.clone(); // Clonamos el transmisor aquí

            match get_tokens_from_db(&*conn_guard) {
                Ok(tokens) => tokens,
                Err(e) => {
                    tx.send(format!("Error al obtener tokens de la base de datos : {}", e)).unwrap();
                    //eprintln!("Error al obtener tokens de la base de datos: {}", e);
                    sleep(update_interval).await;
                    continue;
                }
            }
        };

        // Obtener la APIKEY desde la configuración
        let (api_key, _) = myconfig::leer_config();

        // Actualizar cada token en la base de datos
        for token in tokens {
            
            match coingecko::get_coins_list_full(vec![&token], &api_key).await {
                Ok(token_data) => {
                   //if token_data[0].symbol.len() == 0 { break; } // Verificar si token_data está vacío
                   if token_data.is_empty() {
                      //envio mensaje x canal a la interface
                      tx.send(format!("Simbolo vacio de datos para el token {}: {}", token.to_uppercase(), "empty")).unwrap();
                       continue; // Saltar a la siguiente iteración del bucle
                   }
                                      
  
                    // Llamar a la función para actualizar la base de datos
                    let conn_guard = conn.lock().await;
                    if let Err(e) = update_tokens_masparametros_in_db(
                            &*conn_guard,
                            &token,                                     // token_name
                            &token_data[0].symbol,                           // symbol
                            token_data[0].current_price,                     // current_price
                            token_data[0].market_cap as f64,                 // market_cap
                            token_data[0].total_supply,                      // total_supply
                            token_data[0].max_supply.unwrap_or(0.0),         // max_supply
                            token_data[0].circulating_supply, 

                    ) {
                        tx.send(format!("Error al actualizar la base de datos para el token {}: {}", token.to_uppercase(), e)).unwrap();
                        //eprintln!("Error al actualizar la base de datos para el token {}: {}", token, e);
                    }
                }
                Err(e) => {
                    tx.send(format!("Error al obtener datos de CoinGecko para el token {}: {}", token.to_uppercase(), e)).unwrap();
                    //eprintln!("Error al obtener datos de CoinGecko para el token {}: {}", token, e);
                }
            }
        }

        // Espera antes de la siguiente actualización
        sleep(update_interval).await;
    }
}


pub async fn start_timer_Prices_tokens(conn: Arc<TokioMutex<Connection>>, tx: mpsc::Sender<String>, app: Arc<TokioMutex <App>> ) {
   
    //obtengo el ultimo update del token elegido
    //COMO ACEDO AL STRUCT APP{} DE INTERFACE.RS 
    
    //1)// seteo : token= name--->PARAMETRO TOMADO DE APP.selected_item: String
    // timestamp: fechahsinicio----->BASE DE DATOS PRICES WHER NAME= --- INTERVALO=--- 
    // timestamp :: fecha actual-final ---> TOMADO DE now() utc timestamp


    //2) // Obtener la APIKEY desde la configuración
    let (api_key, _) = myconfig::leer_config();
    let interval_actualizarBD: u64 = myconfig::leer_config_Intervalo_actualizacionPrices().parse().unwrap();
    let update_interval = Duration::from_secs( interval_actualizarBD); // Actualiza cada interval_actualizarBD
    
  
    //3)// API CONSULTA NAME INTERVALO TIMESTAMP= INICIO 
     
    //1)//GUARDAR DATOS EN B.D QUE NO HAY        
       // Actualizar cada token en la base de datos
       loop {
        {
            let app_lock = app.lock().await; // Bloquea el mutex
            let  token = app_lock.selected_item.clone();
            //println!("Dentro  del bucle Token:   ------ {} en intervalos de :{}",
            //         token, interval_actualizarBD);
            
            // Bloquea la conexión a la base de datos para obtener el último timestamp
            let conn_guard = conn.lock().await;
            //let token_id = token.parse::<i32>().unwrap();

            // Consulta el último timestamp desde la base de datos
             // Luego, en la función que llama a get_last_update_timestamp_prices_token:
            // Obtén el último timestamp registrado para el token actual en la base de datos
            let start_time_unix = match dbManagerCud::get_last_update_timestamp_prices_token(&conn_guard, token.clone()) {
                Ok(Some(ts)) => ts.parse::<f64>().unwrap(),
                Ok(None) => 0.0,  // Si no hay registros, empieza desde 0
                Err(e) => {
                    //println!("Error al obtener el último timestamp: {}", e);
                    0.0
                },
            };

            let now_unix = Utc::now().timestamp() as u64;

            // Consulta el precio del token para el intervalo de tiempo desde `start_time_unix` hasta `now_unix`
            if let Ok(price_data) = consulta_api_de_precios(&token, start_time_unix as u64, now_unix, &api_key).await {
                for (timestamp, price) in price_data.prices {
                    // Si el timestamp es mayor que el último registrado, inserta los datos
                    if timestamp > start_time_unix {
                        let fecha_y_hora = &chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp as i64, 0);
                        let formato_fecha_y_hora = fecha_y_hora.unwrap().to_string();

                        let result = dbManagerCud::insert_price(
                            &conn_guard,
                            token.clone(),
                            price,
                            &formato_fecha_y_hora,
                            &interval_actualizarBD.to_string(), 
                        );

                        if let Err(e) = result {
                            println!("Error al insertar precio: {}", e);
                        }
                    } else {
                        println!("Datos duplicados para el timestamp: {}", timestamp);
                    }
                }
            } else {
                println!("Error al obtener datos de CoinGecko para el token: {}", token);
            }
        }

        // Espera el intervalo de actualización antes de la próxima consulta
        tokio::time::sleep(update_interval).await;
    }
}
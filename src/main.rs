use colorize::AnsiColor;
//use inquire::{validator::Validation, CustomUserError};
//use chrono::NaiveDate;
use chrono::{Utc, Duration};
use ui::graficas::grafica_token_by_price_time;

//use crate::dbManager::dbManagerCrud::get_tokens

mod timer2;
use crate::timer2::{start_timer_Info_tokens, start_timer};
use rusqlite::Connection;
use std::error::Error;

use std::ops::Add;
//use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
//use tokio::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
//use tokio::main;
//use tokio;
//use tokio::sync::Mutex as TokioMutex;


mod llamadasApi;
use llamadasApi::*;
mod promptInit;
use promptInit::*;
mod coinStdinOut;
use coinStdinOut::*;
mod dbManager;
use dbManager::*;
mod myconfig;
use myconfig::*;
mod ui;

//ratatui

use crossterm::terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;

use crossterm::event::{self, EnableMouseCapture, Event, KeyCode};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use ui::interface::*;
//***** */
use crate::coinStdinOut::coinStdinOut::load_crypto_from_file;
//***** */

//aplica que no muestre el no uso de var en todo el mod .
#[allow(unused_variables)]


#[derive(Debug, Clone)] // Deriva Clone
pub struct ConsultaCrypto{
   // name : String,
    crypto_list: Vec<String>,
  //  fecha_creacion: NaiveDate
}

#[tokio::main]
async fn main()-> Result<(), Box<dyn std::error::Error>> {
    if my_prompt_boolean()  {
      
       let Consulta = ConsultaCrypto{
        crypto_list: my_prompt_multiselect(),
      };   // Crear las tablas si no existen
    /*-levanto el listado de criptos de ListToken.txt.
      -Guardo en el archivo consulta.txt las tokens a seguir */
    crate::coinStdinOut::coinStdinOut::save_crypto_to_file(&Consulta, "consulta.txt")?;

    }    
//*****levanto del archivo consulta.txt las cripto a seguir***** */    
let consulta =crate::coinStdinOut::coinStdinOut::load_crypto_from_file("consulta.txt")?;
 // Establecer conexión con la base de datos
 let conn = Connection::open("data.db").map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
 // Crear las tablas si no existen
 dbManager::dbManagerCreation::create_tables(&conn)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
 // Obtener los tokens existentes en la base de datos
 let existing_tokens = dbManager::dbManagerCud::get_tokens(&conn);
 // Crear un conjunto de tokens a seguir desde el archivo para comparación rápida
 let consulta2 = consulta.clone();
 let tokens_to_keep: std::collections::HashSet<String> = consulta2.crypto_list.into_iter().collect();
 // Iterar sobre los tokens existentes y
 // eliminar los que NO están en el archivo
    for name in existing_tokens.unwrap() {
    if !tokens_to_keep.contains(&name) {
        // Eliminar el token de la base de datos
        dbManager::dbManagerCud::delete_token(&conn, name);
    }
} 
  
 // Actualizar tokens en la base de datos
 let _ = dbManager::dbManagerCud::update_tokens_in_db_solo_nombre(&conn, &consulta);

 //Creo canal de comunicacion para Mensajes 
 let (tx, rx) = mpsc::channel();
  // Clonar el transmisor para usarlo en diferentes tareas
  let tx1 = tx.clone();//timer encabezado tokens
  let tx2 = tx.clone();//timer Info tokens
  let tx3 = tx.clone();//timer prices


 // Establecer conexión con la base de datos nuevamente 
 let conn1 = Arc::new(TokioMutex::new(Connection::open("data.db").map_err(|e| io::Error::new(io::ErrorKind::Other, e)).expect("Failed to open database")));
// Iniciar el temporizador para actualizar la base de datos cada 5 minutos
    // tokio::spawn(start_timer(conn1.clone(), consulta.clone()),tx).await;
    let consulta1 = consulta.clone();
    tokio::spawn(async move {
        start_timer(conn1.clone(),consulta1.clone(), tx1.clone()).await;
    });
// Llamar a la función `start_timer_Info_tokens` en otro hilo separado
    let conn_clone = Arc::new(TokioMutex::new(Connection::open("data.db").expect("Failed to open database")));
    
    tokio::spawn(async move {
        start_timer_Info_tokens(conn_clone, tx2).await;
    });

//**************Ratatoui***************** */
// Configurar terminal
enable_raw_mode()?;
let mut stdout = io::stdout();
execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

let backend = CrosstermBackend::new(stdout);
let mut terminal = Terminal::new(backend)?;
//cargo el listado de criptos , viene del consulta.txt 
let app = Arc::new(TokioMutex::new(App::new(consulta.crypto_list, rx)));

// Al llamar a render_ui, bloquea el mutex
{
    let mut  app_lock = app.lock().await; // Bloquea el mutex
    app_lock.render_ui(&mut terminal, &conn)?; // Llama a render_ui
}

// Llamar a la función `start_timer_prices_tokens` en otro hilo separado
let conn_clone1 = Arc::new(TokioMutex::new(Connection::open("data.db").expect("Failed to open database")));
 // Clona `app` para pasarla a la tarea asíncrona
let app_clone_for_timer = Arc::clone(&app);
tokio::spawn(async move {
    timer2::start_timer_Prices_tokens(conn_clone1, tx3, app_clone_for_timer).await;
});
loop {
    if event::poll(std::time::Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            let mut app_lock = app.lock().await; // Bloquea el mutex una vez por evento

            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => break, // Salir con Esc o 'q'
                
                KeyCode::Up => {
                    app_lock.move_up(); // Moverse hacia arriba en la lista
                }

                KeyCode::Down => {
                    app_lock.move_down(); // Moverse hacia abajo en la lista
                }

                KeyCode::Right => {
                    // Obtén el nombre del token seleccionado desde la lista
                    let selected_token = app_lock.selected_item.clone();
                    //actualiza asyncronicamente los datos en d.b
                    match dbManagerCud::get_token_data(&conn, &selected_token) {
                        Ok(Some(token_data)) => {
                            app_lock.token_data = Some(token_data.clone());
                            app_lock.add_message(format!("Datos cargados para: {}", selected_token.green()));
                        }
                        Ok(None) => {
                            app_lock.token_data = None;
                            app_lock.add_message(format!("No hay datos para: {}", selected_token.red()));
                        }
                        Err(e) => {
                            app_lock.token_data = None;
                            app_lock.add_message(format!("Error al cargar datos: {}", e.to_string().red()));
                        }
                    }
                    
                }

                _ => {}
            }
             // Redibujar la interfaz de usuario después de procesar el evento
             app_lock.render_ui(&mut terminal, &conn)?;
        }
    }
}

execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
disable_raw_mode()?;
terminal.show_cursor()?;
//*********************end ratatui******************************** */
    Ok(())
}
use chrono::Utc;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::palette::material::YELLOW;
use ratatui::widgets::{Axis, Block, Borders, Chart, Clear, Dataset, List, ListItem, ListState, Paragraph};
use ratatui::{symbols, Frame};
use ratatui::{Terminal, style::{Style, Color}};

use rusqlite::Connection;

use std::io;

use colorize::AnsiColor;
//use colored::*; 

use crate::dbManager;
use crate::dbManagerCud::TokenData;
use crate::grafica_token_by_price_time;


use std::sync::mpsc::Receiver;

use super::graficas;



#[derive(Debug)]
pub struct App {
   
    pub selected_item: String,
    pub intervalo_consulta_item:u64 ,
    pub list_items: Vec<String>,
    pub token_data: Option<TokenData>,
    pub selected_index: usize,
    pub list_state: ListState, 
    messages: String, // Almacenar los mensajes 
    rx: Receiver<String>, // Canal p recibir mensajes y mostrar en la app
}

impl App {
    // Función para agregar un mensaje
    pub fn add_message(&mut self, msg: String) {
        self.messages= msg.yellow();
        
    }
    pub fn new(crypto_list : Vec<String>,rx: Receiver<String>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0)); // Inicialmente selecciona el primer ítem

        App {
            token_data: None, //TokenData { name: (" ".to_string()), symbol: (" ".to_string()), current_price: (0.0), market_cap: (0.0), total_suply: (0.0), max_suply: (0.0), circulating_suply: (0.0) },
            selected_item: crypto_list[0].clone(),
            intervalo_consulta_item: 0,
            list_items: crypto_list,
            selected_index: 0,
            list_state,  // Inicializar el estado de la lista
            messages: " ".to_string(),
            rx: rx, // Inicializar el canal de recepción
        }
    }

    // Agregar una función para obtener el estado de la lista mutable
    pub fn list_state(&mut self) -> &mut ListState {
        &mut self.list_state
    }

    pub fn move_up(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if selected > 0 {
                self.list_state.select(Some(selected - 1));
                self.selected_item = self.list_items[selected - 1].clone();
            }
        }
    }

    pub fn move_down(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if selected < self.list_items.len() - 1 {
                self.list_state.select(Some(selected + 1));
                self.selected_item = self.list_items[selected + 1].clone();
            }
        }
    }

    pub fn render_ui(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, db_conn: &Connection) -> Result<(), io::Error> {
        
      /*   // Leer mensajes desde el canal
         while let Ok(message) = self.rx.try_recv() {
            self.messages = message.yellow();
            
        }
  */
        terminal.draw(|f| {
            // Dividir la pantalla en tres partes principales: izquierda, derecha y el pie de página.
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(80), // Espacio para la parte principal (izquierda + derecha)
                        Constraint::Length(3),      // Espacio para los mensajes en la parte inferior
                        Constraint::Length(3),      // Espacio para los atajos de teclado en la parte inferior
                    ]
                    .as_ref(),
                )
                .split(f.size());

            // Dividir la parte principal en izquierda y derecha.
            let left_right_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(15), // Parte izquierda
                        Constraint::Percentage(85), // Parte derecha (dividida en dos)
                    ]
                    .as_ref(),
                )
                .split(main_chunks[0]);

            // Dividir la parte derecha en dos secciones.
            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(40), // Parte superior derecha
                        Constraint::Percentage(60), // Parte inferior derecha
                    ]
                    .as_ref(),
                )
                .split(left_right_chunks[1]);
             //-----------Mensaje-----------------
             //let msg= "Cargando Datos".b_yellow();
             //app.add_message(format!("{}", msg));
            // f.render_widget(msg, main_chunks[1]);
             //----------------------------    
  //-------------------------// Renderizar la parte izquierda con los elementos de `self.list_items`.
            let items: Vec<_> = self.list_items.iter().map(|i| ListItem::new(i.as_str())).collect();
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Lista Tokens"))
                .highlight_style(Style::default().fg(Color::Yellow)) // Resaltar el ítem seleccionado
                .highlight_symbol("🦀->> ");
  
            f.render_stateful_widget(list, left_right_chunks[0], &mut self.list_state);

  //-------------------------// Renderizar la parte superior derecha.
            let details_paragraph = if let Some(token_data) = &self.token_data {
                let details = vec![
                    format!("Name: {}", token_data.name.to_string().yellow()),
                    format!("Symbol: {}", token_data.symbol.to_string().yellow()),
                    format!("Price: ${:.3}", token_data.current_price),
                    format!("Market Cap: ${:.2}", token_data.market_cap),
                    format!("Total Supply: {:.0}", token_data.total_suply),
                    format!("Max Supply: {:.0}", token_data.max_suply),
                    format!("Circulating Supply: {:.0}", token_data.circulating_suply),
                ];
                Paragraph::new(details.join("\n"))
                    .block(Block::default().borders(Borders::ALL).title("Token Details"))
            } else {
                Paragraph::new("No token data available")
                    .block(Block::default().borders(Borders::ALL).title("Token Details"))
            };
            //Limpia la parte derecha superior (puedes ajustar las divisiones)
            f.render_widget(Clear, right_chunks[0]); // Limpia la parte derecha antes de dibujar
            f.set_cursor(0, 0);
            f.render_widget(details_paragraph, right_chunks[0]);
            
               
          
 //-------------------------// Renderizar la parte inferior derecha. Graficos
            let right_bottom_section = Paragraph::new("Parte Inferior Derecha")
                .block(Block::default().borders(Borders::ALL).title("Derecha Inferior"));
            //lamada a graficar -----
            let start_time = Utc::now() - chrono::Duration::hours(1);
            let end_time = Utc::now();

            // Llamada a la función de graficado
            graficas::grafica_token_by_price_time(
                f,
                right_chunks[1],
                &self.selected_item,
                db_conn,
                start_time,
                end_time,
            );

            //---------------------
           // f.render_widget(Clear, main_chunks[1]); 
           // f.render_widget(right_bottom_section, right_chunks[1]);

 //-------------------------//Renderizar la sección de mensajes en la parte inferior.
           let messages_text = self.messages.clone(); //.join(" -- ");
            let messages_section = Paragraph::new(messages_text)
                .block(Block::default().borders(Borders::ALL).title("Mensajes"));
           
            //Limpia 
            f.render_widget(Clear, main_chunks[1]); 
            f.set_cursor(0, 0);
            f.render_widget(messages_section, main_chunks[1]);

 //-------------------------// Renderizar la sección de atajos de teclado en la parte inferior.
            let shortcuts_section = Paragraph::new("Atajos: [Q] Salir  [H] Ayuda  [⬆️⬇️] Mover [➡️] Cargar ")
                .block(Block::default().borders(Borders::ALL).title("Atajos"));
            f.render_widget(shortcuts_section, main_chunks[2]);
 //-------------------------// 

        })?;
        Ok(())
    }



    pub fn update_right_section(&mut self, selected_item: String, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, db_conn: &Connection) -> Result<(), io::Error> {
        self.selected_item = selected_item;
        self.render_ui(terminal, db_conn)
    }
}
pub fn render_token_details(f: &mut Frame, area: Rect, selected_token: &str, db_conn: &Connection) {
    match dbManager::dbManagerCud::get_token_data(db_conn, selected_token) {
        Ok(Some(token_data)) => {
            let block = Block::default().title("Token Details").borders(Borders::ALL);
            let paragraph = Paragraph::new(format!(
                "Name: {}\nSymbol: {}\nPrice: {}\nMarket Cap: {}",
                token_data.name,
                token_data.symbol,
                token_data.current_price,
                token_data.market_cap
            ))
            .block(block)
            .alignment(Alignment::Left);

            f.render_widget(paragraph, area);
        },
        Ok(None) => {
            // Manejo de caso cuando no se encuentran datos para el token
            let block = Block::default().title("Token Details").borders(Borders::ALL);
            let paragraph = Paragraph::new("No data available for this token")
                .block(block)
                .alignment(Alignment::Left);

            f.render_widget(paragraph, area);
        },
        Err(e) => {
            // Manejo de errores
            let block = Block::default().title("Token Details").borders(Borders::ALL);
            let paragraph = Paragraph::new(format!("Error fetching data: {}", e))
                .block(block)
                .alignment(Alignment::Left);

            f.render_widget(paragraph, area);
        },
    }
}

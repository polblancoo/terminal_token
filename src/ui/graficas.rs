// En grafica.rs

use ratatui::{
    prelude::*,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
};
use chrono::{DateTime, Duration, TimeZone, Utc};
use rusqlite::Connection;

use crate::dbManagerCud::{self, get_prices_time_to_vec};

pub fn grafica_token_by_price_time( f: &mut Frame, area: Rect,selected_token: &str,
                                    db_conn: &Connection,start_time: DateTime<Utc>,end_time: DateTime<Utc>,) {
    
    // Obtener los datos de precio del token
    let prices = match get_prices_time_to_vec(db_conn, 
                            selected_token, &start_time, &end_time, 60.0) {
        Ok(prices) => prices,
        Err(_) => return, // Si hay un error, simplemente retornamos sin dibujar nada
    };

    let mut prices = prices.clone();
    const MAX_SIZE: usize = 50;
    if  prices.len() > MAX_SIZE {
        prices.truncate(MAX_SIZE);
    }

 // Asegúrate de que prices no esté vacío
 if prices.is_empty() {
    let error_message = "No hay precios para graficar.";
    let error_paragraph = Paragraph::new(error_message)
        .block(Block::default().title("Error").borders(Borders::ALL));
    f.render_widget(error_paragraph, area);
    //f.render_widget(chart, area);
    return;
}
// Convertir los datos a un formato que ratatui pueda usar
let data: Vec<(f64, f64)> = prices
    .iter()
    .map(|(time, price)| (time.timestamp() as f64, *price))
    .collect();


let (min_x, max_x) = prices
.iter()
.map(|(time, _)| time.timestamp() as f64)
.fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), x| (min.min(x), max.max(x)));

let (min_y, max_y) = prices
.iter()
.map(|(_, price)| *price)
.fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), y| (min.min(y), max.max(y)));

// Ajustar los límites para dar un poco de margen
let x_range = (min_x - 0.1 * (max_x - min_x), max_x + 0.1 * (max_x - min_x));
let y_range = (min_y - 0.1 * (max_y - min_y), max_y + 0.1 * (max_y - min_y));

let dataset = Dataset::default()
.name(selected_token).fg(Color::Yellow)
.marker(symbols::Marker::Braille)
.graph_type(GraphType::Line)
.data(&data);

let chart = Chart::new(vec![dataset])
.block(Block::default().title("Token Prices").borders(Borders::ALL))
.x_axis(
    Axis::default()
        .title("Tiempo")
        .style(Style::default().fg(Color::Green))
        .bounds(x_range.into())
        .labels(vec![
            Span::styled(
                Utc.timestamp_opt(x_range.0 as i64, 0)
                    .single()
                    .unwrap()
                    .format("%Y-%m-%d")
                    .to_string(),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                Utc.timestamp_opt(((x_range.0 + x_range.1) / 2.0 as f64 )as i64, 0)
                    .single()
                    .unwrap()
                    .format("%Y-%m-%d")
                    .to_string(),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                Utc.timestamp_opt(x_range.1 as i64, 0)
                    .single()
                    .unwrap()
                    .format("%y-%m-%d")
                    .to_string(),
                Style::default().fg(Color::White),
            ),
        ]),
)
.y_axis(
    Axis::default()
        .title("Precio")
        .style(Style::default().fg(Color::Green))
        .bounds(y_range.into())
        .labels(vec![
            format!("{:.2}", y_range.0).into(),
            format!("{:.2}", (y_range.0 + y_range.1) / 2.0).into(),
            format!("{:.2}", y_range.1).into(),
        ]),
);

f.render_widget(chart, area);
}





async fn generar_datos_grafico(conn: &Connection, token: &str, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<Vec<(f64, f64)>, Box<dyn std::error::Error>> {
    // Obtiene los precios del token desde la base de datos
    let precios = dbManagerCud::get_prices_time_to_vec(conn, token, &start_time, &end_time, 60.0)?;

    // Convierte los precios a un formato adecuado para el gráfico
    let mut data: Vec<(f64, f64)> = precios.iter()
        .map(|&(timestamp, price)| (timestamp.timestamp_millis() as f64, price))
        .collect();

    Ok(data)
}

/*
pub fn render_token_graph(f: &mut Frame, area: Rect, selected_token: &str, db_conn: &Connection) {
    let price_data = dbManager::dbManagerCud::get_token_prices(db_conn, selected_token);

    let datasets = vec![Dataset::default()
        .name(selected_token)
        .marker(symbols::Marker::Dot)
        .style(Style::default().fg(Color::Cyan))
        .data(&price_data)];

    let chart = Chart::new(datasets)
        .block(Block::default().title("Price Graph").borders(Borders::ALL))
        .x_axis(Axis::default()
            .title("Time")
            .style(Style::default().fg(Color::Gray))
            .bounds([0.0, 100.0]))
        .y_axis(Axis::default()
            .title("Price")
            .style(Style::default().fg(Color::Gray))
            .bounds([0.0, 100.0]));

    f.render_widget(chart, area);
}
*/
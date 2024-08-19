// En grafica.rs

use ratatui::{
    prelude::*,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
};
use chrono::{DateTime, Utc, Duration};
use rusqlite::Connection;

use crate::dbManagerCud::{self, get_prices_time_to_vec};

pub fn grafica_token_by_price_time(
    f: &mut Frame,
    area: Rect,
    selected_token: &str,
    db_conn: &Connection,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) {
    // Obtener los datos de precio del token
    let prices = match get_prices_time_to_vec(db_conn, selected_token, &start_time, &end_time) {
        Ok(prices) => prices,
        Err(_) => return, // Si hay un error, simplemente retornamos sin dibujar nada
    };

    // Convertir los datos a un formato que ratatui pueda usar
    let data: Vec<(f64, f64)> = prices
        .iter()
        .map(|(time, price)| {
            (time.timestamp() as f64, *price)
        })
        .collect();

    // Crear el dataset
    let dataset = Dataset::default()
        .name(selected_token)
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .data(&data);

    // Configurar los ejes
    let x_labels = vec![
        Span::raw(start_time.format("%Y-%m-%d").to_string()),
        Span::raw(end_time.format("%Y-%m-%d").to_string()),
    ];
    let y_labels = vec![
        Span::raw(format!("{:.2}", data.iter().map(|&(_, y)| y).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0))),
        Span::raw(format!("{:.2}", data.iter().map(|&(_, y)| y).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0))),
    ];

    // Crear el gráfico
    let chart = Chart::new(vec![dataset])
        .block(Block::default().title(format!("Precio de {} en el tiempo", selected_token)).borders(Borders::ALL))
        .x_axis(Axis::default().title("Fecha").bounds([start_time.timestamp() as f64, end_time.timestamp() as f64]).labels(x_labels))
        .y_axis(Axis::default().title("Precio").bounds([data.iter().map(|&(_, y)| y).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0), 
                                                        data.iter().map(|&(_, y)| y).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(0.0)]).labels(y_labels));

    // Renderizar el gráfico
    f.render_widget(chart, area);
}


async fn generar_datos_grafico(conn: &Connection, token: &str, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Result<Vec<(f64, f64)>, Box<dyn std::error::Error>> {
    // Obtiene los precios del token desde la base de datos
    let precios = dbManagerCud::get_prices_time_to_vec(conn, token, &start_time, &end_time)?;

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
// src/db_manager/db_manager_creation.rs

use rusqlite::{Connection, Result};

pub fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            symbol TEXT NOT NULL,
            current_price REAL NOT NULL,
            market_cap REAL NOT NULL,
            total_suply REAL NOT NULL,
            max_suply REAL NOT NULL,
            circulating_suply REAL NOT NULL

        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS prices (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            token_id TEXT NOT NULL,
            price REAL NOT NULL,
            timestamp REAL NOT NULL,
            interval REAL NOT NULL,
            FOREIGN KEY(token_id) REFERENCES tokens(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            key TEXT NOT NULL,
            value TEXT NOT NULL
        )",
        [],
    )?;

    Ok(())
}

//! Application configuration loaded from environment variables.

use std::env;

/// Application configuration.
///
/// Loaded from environment variables at startup.
#[derive(Clone)]
pub struct Config {
    /// PostgreSQL connection string
    pub database_url: String,
    /// Server bind address (default: 0.0.0.0:8080)
    pub server_addr: String,
    /// Binance API key (optional, for live trading)
    pub binance_api_key: Option<String>,
    /// Binance secret key (optional, for live trading)
    pub binance_secret_key: Option<String>,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// # Required Environment Variables
    ///
    /// - `DATABASE_URL` - PostgreSQL connection string
    ///
    /// # Optional Environment Variables
    ///
    /// - `SERVER_ADDR` - Server bind address (default: 0.0.0.0:8080)
    /// - `BINANCE_API_KEY` - Binance API key for live trading
    /// - `BINANCE_SECRET_KEY` - Binance secret key for live trading
    ///
    /// # Panics
    ///
    /// Panics if `DATABASE_URL` is not set.
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            server_addr: env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            binance_api_key: env::var("BINANCE_API_KEY").ok(),
            binance_secret_key: env::var("BINANCE_SECRET_KEY").ok(),
        }
    }
}

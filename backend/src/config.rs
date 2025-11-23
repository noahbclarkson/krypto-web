use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub server_addr: String,
    pub binance_api_key: Option<String>,
    pub binance_secret_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            server_addr: env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            binance_api_key: env::var("BINANCE_API_KEY").ok(),
            binance_secret_key: env::var("BINANCE_SECRET_KEY").ok(),
        }
    }
}

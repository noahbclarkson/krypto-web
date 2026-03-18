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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_from_env_with_defaults() {
        env::set_var("DATABASE_URL", "postgres://test:test@localhost/test");
        env::remove_var("SERVER_ADDR");
        env::remove_var("BINANCE_API_KEY");
        env::remove_var("BINANCE_SECRET_KEY");

        let config = Config::from_env();

        assert_eq!(config.database_url, "postgres://test:test@localhost/test");
        assert_eq!(config.server_addr, "0.0.0.0:8080");
        assert!(config.binance_api_key.is_none());
        assert!(config.binance_secret_key.is_none());

        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn test_config_from_env_with_custom_values() {
        env::set_var("DATABASE_URL", "postgres://custom:5432/db");
        env::set_var("SERVER_ADDR", "127.0.0.1:3000");
        env::set_var("BINANCE_API_KEY", "my_api_key");
        env::set_var("BINANCE_SECRET_KEY", "my_secret");

        let config = Config::from_env();

        assert_eq!(config.database_url, "postgres://custom:5432/db");
        assert_eq!(config.server_addr, "127.0.0.1:3000");
        assert_eq!(config.binance_api_key, Some("my_api_key".to_string()));
        assert_eq!(config.binance_secret_key, Some("my_secret".to_string()));

        env::remove_var("DATABASE_URL");
        env::remove_var("SERVER_ADDR");
        env::remove_var("BINANCE_API_KEY");
        env::remove_var("BINANCE_SECRET_KEY");
    }

    #[test]
    fn test_config_clone() {
        env::set_var("DATABASE_URL", "postgres://test@localhost/db");

        let config1 = Config::from_env();
        let config2 = config1.clone();

        assert_eq!(config1.database_url, config2.database_url);
        assert_eq!(config1.server_addr, config2.server_addr);

        env::remove_var("DATABASE_URL");
    }
}

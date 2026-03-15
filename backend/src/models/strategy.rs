use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Strategy {
    pub id: Uuid,
    pub name: String,
    pub strategy_type: String,
    pub symbol: String,
    pub interval: String,
    pub parameters: serde_json::Value,
    pub performance_metrics: Option<serde_json::Value>,
    pub backtest_curve: Option<serde_json::Value>,
    pub kelly_fraction: Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStrategyRequest {
    pub name: String,
    pub strategy_type: String,
    pub symbol: String,
    pub interval: String,
    pub parameters: serde_json::Value,
    pub performance_metrics: Option<serde_json::Value>,
    pub backtest_curve: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateStrategiesRequest {
    pub symbols: Vec<String>,
    pub intervals: Vec<String>,
    pub top_n: Option<usize>,
    pub limit: Option<u16>,
    pub iterations: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub symbol: String,
    pub interval: String,
    pub initial_capital: f64,
    pub current_equity: f64,
    pub entry_equity: Option<f64>,
    pub current_position: f64,
    pub entry_price: Option<f64>,
    pub highest_high: Option<f64>,
    pub lowest_low: Option<f64>,
    pub status: String,
    pub execution_mode: String,
    pub allocated_weight: f64,
    pub created_at: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub strategy_id: Uuid,
    pub initial_capital: f64,
    pub execution_mode: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Trade {
    pub id: Uuid,
    pub session_id: Uuid,
    pub symbol: String,
    pub side: String,
    pub price: f64,
    pub quantity: f64,
    pub pnl: Option<f64>,
    pub reason: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_generate_strategies_request_full() {
        let json = r#"{
            "symbols": ["BTCUSDT", "ETHUSDT"],
            "intervals": ["1h", "4h"],
            "top_n": 10,
            "limit": 1000,
            "iterations": 50
        }"#;

        let req: GenerateStrategiesRequest =
            serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(req.symbols, vec!["BTCUSDT", "ETHUSDT"]);
        assert_eq!(req.intervals, vec!["1h", "4h"]);
        assert_eq!(req.top_n, Some(10));
        assert_eq!(req.limit, Some(1000));
        assert_eq!(req.iterations, Some(50));
    }

    #[test]
    fn test_generate_strategies_request_minimal() {
        let json = r#"{
            "symbols": ["BTCUSDT"],
            "intervals": ["1h"]
        }"#;

        let req: GenerateStrategiesRequest =
            serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(req.symbols, vec!["BTCUSDT"]);
        assert_eq!(req.intervals, vec!["1h"]);
        assert_eq!(req.top_n, None);
    }

    #[test]
    fn test_create_session_request_full() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let json = format!(
            "{{
                \"strategy_id\": \"{}\",
                \"initial_capital\": 10000.0,
                \"execution_mode\": \"paper\"
            }}",
            uuid_str
        );

        let req: CreateSessionRequest =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(req.strategy_id, Uuid::parse_str(uuid_str).unwrap());
        assert!((req.initial_capital - 10000.0).abs() < f64::EPSILON);
        assert_eq!(req.execution_mode, Some("paper".to_string()));
    }

    #[test]
    fn test_create_session_request_minimal() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let json = format!(
            "{{
                \"strategy_id\": \"{}\",
                \"initial_capital\": 5000.0
            }}",
            uuid_str
        );

        let req: CreateSessionRequest =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(req.strategy_id, Uuid::parse_str(uuid_str).unwrap());
        assert!((req.initial_capital - 5000.0).abs() < f64::EPSILON);
        assert_eq!(req.execution_mode, None);
    }

    #[test]
    fn test_create_strategy_request() {
        let json = r#"{
            "name": "Bollinger Reversion",
            "strategy_type": "bollinger_reversion",
            "symbol": "BTCUSDT",
            "interval": "1h",
            "parameters": {"period": 20, "std_dev": 2.0},
            "performance_metrics": {"sharpe": 1.5, "win_rate": 0.55},
            "backtest_curve": [[1.0, 10000.0], [2.0, 10200.0]]
        }"#;

        let req: CreateStrategyRequest =
            serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(req.name, "Bollinger Reversion");
        assert_eq!(req.strategy_type, "bollinger_reversion");
        assert_eq!(req.symbol, "BTCUSDT");
        assert_eq!(req.interval, "1h");
        assert!(req.performance_metrics.is_some());
        assert!(req.backtest_curve.is_some());
    }
}

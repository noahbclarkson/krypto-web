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

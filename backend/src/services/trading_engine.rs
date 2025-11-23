use std::sync::Arc;
use std::time::Duration;

use krypto::algo::strategies::{
    AdaptiveMaCrossover, AtrBreakout, BollingerReversion, DynamicTrend, MacdTrend, ObvTrend,
    PriceMomentum, RsiMeanReversion, VolatilitySqueeze,
};
use krypto::algo::SignalGenerator;
use krypto::features::indicators::FeatureEngine;
use polars::prelude::*;
use serde_json::Value;
use sqlx::{FromRow, PgPool};
use tokio::time;
use tracing::{error, info, warn};

use chrono::Utc;

use crate::error::AppError;
use crate::models::strategy::Session;
use crate::services::market_data::MarketDataService;

#[derive(FromRow)]
struct StrategyRow {
    strategy_type: String,
    parameters: Value,
}

pub async fn start_engine(pool: PgPool, market_service: Arc<MarketDataService>) {
    let interval_duration = Duration::from_secs(30);
    let mut interval = time::interval(interval_duration);

    info!("Trading Engine Started");

    loop {
        interval.tick().await;
        if let Err(e) = process_sessions(&pool, &market_service).await {
            error!("Error in trading engine loop: {:?}", e);
        }
    }
}

async fn process_sessions(pool: &PgPool, market: &MarketDataService) -> Result<(), AppError> {
    let sessions = sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE status = 'active'")
        .fetch_all(pool)
        .await?;

    for session in sessions {
        let strategy_record = sqlx::query_as::<_, StrategyRow>(
            "SELECT strategy_type, parameters FROM strategies WHERE id = $1",
        )
        .bind(session.strategy_id)
        .fetch_one(pool)
        .await?;
        let strategy_type = strategy_record.strategy_type;

        let raw_df = market
            .fetch_candles(&session.symbol, &session.interval, 500)
            .await?;

        let time_col = raw_df.column("time").map_err(|e| AppError::Data(e.to_string()))?;
        let last_time_idx = time_col.len().saturating_sub(1);

        let last_timestamp_val = time_col.datetime().map_err(|e| AppError::Data(e.to_string()))?
            .get(last_time_idx)
            .ok_or(AppError::Data("No time data".into()))?;

        let _last_candle_time = chrono::DateTime::from_timestamp(
            last_timestamp_val / 1000,
            (last_timestamp_val % 1000 * 1_000_000) as u32
        ).unwrap_or_default();

        // Allow new sync sessions to process the latest candle immediately.
        let is_fresh_start = session.current_position == 0.0
            && session.execution_mode == "sync"
            && session.entry_price.is_none();

        let time_since_update = Utc::now() - session.last_update;
        if !is_fresh_start && time_since_update < chrono::Duration::seconds(5) {
            continue;
        }

        let df = FeatureEngine::add_technicals(&raw_df, None)
            .map_err(|e| AppError::Data(e.to_string()))?;

        let (signal_series, explanation_series) = match strategy_type.as_str() {
            "DynamicTrend" => {
                let strat: DynamicTrend =
                    serde_json::from_value(strategy_record.parameters.clone())
                        .map_err(|e| AppError::Strategy(format!("Config error: {e}")))?;
                let signals = strat.predict(&df).map_err(|e| AppError::Strategy(e.to_string()))?;
                let explanations = strat.explain(&df).map_err(|e| AppError::Strategy(e.to_string()))?;
                (signals, explanations)
            }
            "RsiMeanReversion" => {
                let strat: RsiMeanReversion =
                    serde_json::from_value(strategy_record.parameters.clone())
                        .map_err(|e| AppError::Strategy(format!("Config error: {e}")))?;
                let signals = strat.predict(&df).map_err(|e| AppError::Strategy(e.to_string()))?;
                let explanations = strat.explain(&df).map_err(|e| AppError::Strategy(e.to_string()))?;
                (signals, explanations)
            }
            "BollingerReversion" => {
                let strat: BollingerReversion =
                    serde_json::from_value(strategy_record.parameters.clone())
                        .map_err(|e| AppError::Strategy(format!("Config error: {e}")))?;
                let signals = strat.predict(&df).map_err(|e| AppError::Strategy(e.to_string()))?;
                let explanations = strat.explain(&df).map_err(|e| AppError::Strategy(e.to_string()))?;
                (signals, explanations)
            }
            "AtrBreakout" => {
                let strat: AtrBreakout = serde_json::from_value(strategy_record.parameters.clone())
                    .map_err(|e| AppError::Strategy(format!("Config error: {e}")))?;
                let signals = strat.predict(&df).map_err(|e| AppError::Strategy(e.to_string()))?;
                let explanations = strat.explain(&df).map_err(|e| AppError::Strategy(e.to_string()))?;
                (signals, explanations)
            }
            "VolatilitySqueeze" => {
                let strat: VolatilitySqueeze =
                    serde_json::from_value(strategy_record.parameters.clone())
                        .map_err(|e| AppError::Strategy(format!("Config error: {e}")))?;
                let signals = strat.predict(&df).map_err(|e| AppError::Strategy(e.to_string()))?;
                let explanations = strat.explain(&df).map_err(|e| AppError::Strategy(e.to_string()))?;
                (signals, explanations)
            }
            "MacdTrend" => {
                let strat: MacdTrend =
                    serde_json::from_value(strategy_record.parameters.clone())
                        .map_err(|e| AppError::Strategy(format!("Config error: {e}")))?;
                let signals = strat.predict(&df).map_err(|e| AppError::Strategy(e.to_string()))?;
                let explanations = strat.explain(&df).map_err(|e| AppError::Strategy(e.to_string()))?;
                (signals, explanations)
            }
            "ObvTrend" => {
                let strat: ObvTrend =
                    serde_json::from_value(strategy_record.parameters.clone())
                        .map_err(|e| AppError::Strategy(format!("Config error: {e}")))?;
                let signals = strat.predict(&df).map_err(|e| AppError::Strategy(e.to_string()))?;
                let explanations = strat.explain(&df).map_err(|e| AppError::Strategy(e.to_string()))?;
                (signals, explanations)
            }
            "PriceMomentum" => {
                let strat: PriceMomentum =
                    serde_json::from_value(strategy_record.parameters.clone())
                        .map_err(|e| AppError::Strategy(format!("Config error: {e}")))?;
                let signals = strat.predict(&df).map_err(|e| AppError::Strategy(e.to_string()))?;
                let explanations = strat.explain(&df).map_err(|e| AppError::Strategy(e.to_string()))?;
                (signals, explanations)
            }
            "AdaptiveMaCrossover" => {
                let strat: AdaptiveMaCrossover =
                    serde_json::from_value(strategy_record.parameters.clone())
                        .map_err(|e| AppError::Strategy(format!("Config error: {e}")))?;
                let signals = strat.predict(&df).map_err(|e| AppError::Strategy(e.to_string()))?;
                let explanations = strat.explain(&df).map_err(|e| AppError::Strategy(e.to_string()))?;
                (signals, explanations)
            }
            _ => {
                warn!("Unknown strategy type: {}", strategy_type);
                continue;
            }
        };

        let signals = signal_series
            .f64()
            .map_err(|e| AppError::Data(e.to_string()))?;
        if signals.is_empty() {
            continue;
        }
        let latest_idx = signals.len() - 1;
        let latest_signal = signals.get(latest_idx).unwrap_or(0.0);
        let latest_reason = explanation_series
            .str()
            .map_err(|e| AppError::Data(e.to_string()))?
            .get(latest_idx)
            .map(|s| s.to_string())
            .unwrap_or_else(|| "No explanation".to_string());

        let closes = df
            .column("close")
            .map_err(|e| AppError::Data(e.to_string()))?
            .f64()
            .map_err(|e| AppError::Data(e.to_string()))?;
        if closes.is_empty() {
            continue;
        }
        let current_price = closes.get(closes.len() - 1).unwrap_or(0.0);

        let target_signal = if session.execution_mode == "edge" {
            let prev_signal = if latest_idx > 0 {
                signals.get(latest_idx - 1).unwrap_or(0.0)
            } else {
                0.0
            };

            if session.current_position == 0.0 && (latest_signal - prev_signal).abs() < 0.01 {
                0.0
            } else {
                latest_signal
            }
        } else {
            latest_signal
        };

        execute_paper_trade(pool, &session, target_signal, current_price, latest_reason).await?;
    }

    Ok(())
}

async fn execute_paper_trade(
    pool: &PgPool,
    session: &Session,
    signal: f64,
    price: f64,
    reason: String,
) -> Result<(), AppError> {
    let now = Utc::now();

    if (signal - session.current_position).abs() < 0.1 {
        let mtm_equity = if session.current_position.abs() > 0.0 {
            let entry = session.entry_price.unwrap_or(price);
            let raw_pnl_pct = if session.current_position > 0.0 {
                (price - entry) / entry
            } else {
                (entry - price) / entry
            };
            session.current_equity * (1.0 + raw_pnl_pct)
        } else {
            session.current_equity
        };

        let mut tx = pool.begin().await?;
        sqlx::query(
            "UPDATE sessions SET current_equity = $1, last_update = $2 WHERE id = $3",
        )
        .bind(mtm_equity)
        .bind(now)
        .bind(session.id)
        .execute(&mut *tx)
        .await?;

        sqlx::query("INSERT INTO equity_snapshots (session_id, equity, timestamp) VALUES ($1, $2, $3)")
            .bind(session.id)
            .bind(mtm_equity)
            .bind(now)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        return Ok(());
    }

    info!(
        "Signal change for session {}: {} -> {} @ ${}",
        session.id, session.current_position, signal, price
    );

    let mut tx = pool.begin().await?;
    let mut new_equity = session.current_equity;

    if session.current_position.abs() > 0.0 {
        let entry = session.entry_price.unwrap_or(price);
        let raw_pnl_pct = if session.current_position > 0.0 {
            (price - entry) / entry
        } else {
            (entry - price) / entry
        };

        let pnl_amount = session.current_equity * raw_pnl_pct;
        new_equity += pnl_amount;

        let side = if session.current_position > 0.0 {
            "SELL"
        } else {
            "BUY"
        };
        sqlx::query(
            "INSERT INTO trades (session_id, symbol, side, price, quantity, pnl, reason) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(session.id)
        .bind(&session.symbol)
        .bind(side)
        .bind(price)
        .bind(0.0_f64)
        .bind(pnl_amount)
        .bind(&reason)
        .execute(&mut *tx)
        .await?;
    }

    if signal.abs() > 0.0 {
        let side = if signal > 0.0 { "BUY" } else { "SELL" };
        sqlx::query(
            "INSERT INTO trades (session_id, symbol, side, price, quantity, pnl, reason) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(session.id)
        .bind(&session.symbol)
        .bind(side)
        .bind(price)
        .bind(0.0_f64)
        .bind(0.0_f64)
        .bind(&reason)
        .execute(&mut *tx)
        .await?;
    }

    let new_entry_price = if signal.abs() > 0.0 {
        Some(price)
    } else {
        None
    };

    sqlx::query(
        "UPDATE sessions SET current_equity = $1, current_position = $2, entry_price = $3, last_update = $4 WHERE id = $5",
    )
    .bind(new_equity)
    .bind(signal)
    .bind(new_entry_price)
    .bind(now)
    .bind(session.id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("INSERT INTO equity_snapshots (session_id, equity, timestamp) VALUES ($1, $2, $3)")
        .bind(session.id)
        .bind(new_equity)
        .bind(now)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

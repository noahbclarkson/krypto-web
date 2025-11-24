use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use binance::ws_model::{CombinedStreamEvent, Kline, WebsocketEvent, WebsocketEventUntag};
use chrono::{DateTime, Utc};
use krypto::algo::strategies::{
    AdaptiveMaCrossover, AtrBreakout, BollingerReversion, DynamicTrend, MacdTrend, ObvTrend,
    PriceMomentum, RsiMeanReversion, VolatilitySqueeze,
};
use krypto::algo::SignalGenerator;
use krypto::features::indicators::FeatureEngine;
use polars::prelude::*;
use serde_json::Value;
use sqlx::{FromRow, PgPool};
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::strategy::Session;
use crate::services::market_data::MarketDataService;
use crate::services::market_stream::MarketStream;

#[derive(FromRow)]
struct StrategyRow {
    strategy_type: String,
    parameters: Value,
}

// Limit snapshot inserts so we don't flood the DB when ticks are noisy.
const SNAPSHOT_COOLDOWN_MS: i64 = 1_000;

pub async fn start_engine(pool: PgPool, market_service: Arc<MarketDataService>) {
    info!("Trading Engine Starting (WebSocket mode)...");

    loop {
        if let Err(e) = run_engine_cycle(&pool, &market_service).await {
            error!("Trading engine error: {:?}", e);
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    }
}

async fn run_engine_cycle(
    pool: &PgPool,
    market_service: &Arc<MarketDataService>,
) -> Result<(), AppError> {
    let mut symbols = fetch_active_symbols(pool).await?;

    if symbols.is_empty() {
        info!("No active sessions detected. Waiting for new sessions...");
        tokio::time::sleep(Duration::from_secs(5)).await;
        return Ok(());
    }

    let (tx, mut rx) = mpsc::unbounded_channel();
    let stream = MarketStream::new();
    stream.start_stream(symbols.clone(), tx).await;
    let mut snapshot_tracker: HashMap<Uuid, DateTime<Utc>> = HashMap::new();
    let mut refresh = tokio::time::interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            maybe_event = rx.recv() => {
                let Some(event) = maybe_event else {
                    warn!("Websocket channel closed, restarting stream after short backoff...");
                    break;
                };
                if let Some((symbol, kline)) = extract_kline(event) {
                    if let Err(e) = process_symbol_update(
                        pool,
                        market_service,
                        &symbol,
                        &kline,
                        &mut snapshot_tracker,
                    )
                    .await
                    {
                        error!("Error processing update for {}: {:?}", symbol, e);
                    }
                }
            }
            _ = refresh.tick() => {
                let current_symbols = fetch_active_symbols(pool).await?;
                if current_symbols != symbols {
                    info!("Active session set changed, refreshing websocket subscriptions...");
                    break;
                }
            }
        }
    }

    stream.stop();
    tokio::time::sleep(Duration::from_secs(2)).await;
    Ok(())
}

async fn fetch_active_symbols(pool: &PgPool) -> Result<Vec<String>, AppError> {
    let sessions = sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE status = 'active'")
        .fetch_all(pool)
        .await?;

    let mut symbols: Vec<String> = sessions.into_iter().map(|s| s.symbol).collect();
    symbols.sort();
    symbols.dedup();
    Ok(symbols)
}

fn extract_kline(event: CombinedStreamEvent<WebsocketEventUntag>) -> Option<(String, Kline)> {
    if let WebsocketEventUntag::WebsocketEvent(WebsocketEvent::Kline(kline_event)) = event.data {
        let symbol = kline_event.kline.symbol.to_uppercase();
        return Some((symbol, kline_event.kline));
    }
    None
}

async fn process_symbol_update(
    pool: &PgPool,
    market: &MarketDataService,
    symbol: &str,
    kline: &Kline,
    snapshot_tracker: &mut HashMap<Uuid, DateTime<Utc>>,
) -> Result<(), AppError> {
    let price = kline.close;
    let is_final_bar = kline.is_final_bar;

    let sessions = sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE status = 'active' AND symbol = $1",
    )
    .bind(symbol)
    .fetch_all(pool)
    .await?;

    for session in sessions {
        update_equity_mtm(pool, &session, price, snapshot_tracker, is_final_bar).await?;

        if is_final_bar {
            run_strategy_logic(pool, market, &session, price, snapshot_tracker).await?;
        }
    }

    Ok(())
}

async fn update_equity_mtm(
    pool: &PgPool,
    session: &Session,
    current_price: f64,
    snapshot_tracker: &mut HashMap<Uuid, DateTime<Utc>>,
    force_snapshot: bool,
) -> Result<(), AppError> {
    if session.current_position.abs() < f64::EPSILON || session.entry_price.is_none() {
        return Ok(());
    }

    let entry_price = session.entry_price.unwrap_or(current_price);
    let basis_equity = session.entry_equity.unwrap_or(session.current_equity);
    let direction = if session.current_position > 0.0 {
        1.0
    } else {
        -1.0
    };
    let raw_pnl_pct = direction * (current_price - entry_price) / entry_price;
    let mtm_equity = basis_equity * (1.0 + raw_pnl_pct);

    let now = Utc::now();
    let time_since_update = now
        .signed_duration_since(session.last_update)
        .num_milliseconds();

    let equity_move = (mtm_equity - session.current_equity).abs();
    let should_update = force_snapshot || equity_move > 1e-6 && time_since_update >= 500;
    if !should_update {
        return Ok(());
    }

    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE sessions SET current_equity = $1, last_update = $2 WHERE id = $3")
        .bind(mtm_equity)
        .bind(now)
        .bind(session.id)
        .execute(&mut *tx)
        .await?;

    let allow_snapshot = force_snapshot
        || snapshot_tracker
            .get(&session.id)
            .map(|ts| now.signed_duration_since(*ts).num_milliseconds() >= SNAPSHOT_COOLDOWN_MS)
            .unwrap_or(true);

    if allow_snapshot {
        sqlx::query(
            "INSERT INTO equity_snapshots (session_id, equity, timestamp) VALUES ($1, $2, $3)",
        )
        .bind(session.id)
        .bind(mtm_equity)
        .bind(now)
        .execute(&mut *tx)
        .await?;
        snapshot_tracker.insert(session.id, now);
    }

    tx.commit().await?;
    Ok(())
}

async fn run_strategy_logic(
    pool: &PgPool,
    market: &MarketDataService,
    session: &Session,
    current_price: f64,
    snapshot_tracker: &mut HashMap<Uuid, DateTime<Utc>>,
) -> Result<(), AppError> {
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

    let df =
        FeatureEngine::add_technicals(&raw_df, None).map_err(|e| AppError::Data(e.to_string()))?;

    let (signal_series, explanation_series) = match strategy_type.as_str() {
        "DynamicTrend" => {
            let strat: DynamicTrend = serde_json::from_value(strategy_record.parameters.clone())
                .map_err(|e| AppError::Strategy(format!("Config error: {e}")))?;
            let signals = strat
                .predict(&df)
                .map_err(|e| AppError::Strategy(e.to_string()))?;
            let explanations = strat
                .explain(&df)
                .map_err(|e| AppError::Strategy(e.to_string()))?;
            (signals, explanations)
        }
        "RsiMeanReversion" => {
            let strat: RsiMeanReversion =
                serde_json::from_value(strategy_record.parameters.clone())
                    .map_err(|e| AppError::Strategy(format!("Config error: {e}")))?;
            let signals = strat
                .predict(&df)
                .map_err(|e| AppError::Strategy(e.to_string()))?;
            let explanations = strat
                .explain(&df)
                .map_err(|e| AppError::Strategy(e.to_string()))?;
            (signals, explanations)
        }
        "BollingerReversion" => {
            let strat: BollingerReversion =
                serde_json::from_value(strategy_record.parameters.clone())
                    .map_err(|e| AppError::Strategy(format!("Config error: {e}")))?;
            let signals = strat
                .predict(&df)
                .map_err(|e| AppError::Strategy(e.to_string()))?;
            let explanations = strat
                .explain(&df)
                .map_err(|e| AppError::Strategy(e.to_string()))?;
            (signals, explanations)
        }
        "AtrBreakout" => {
            let strat: AtrBreakout = serde_json::from_value(strategy_record.parameters.clone())
                .map_err(|e| AppError::Strategy(format!("Config error: {e}")))?;
            let signals = strat
                .predict(&df)
                .map_err(|e| AppError::Strategy(e.to_string()))?;
            let explanations = strat
                .explain(&df)
                .map_err(|e| AppError::Strategy(e.to_string()))?;
            (signals, explanations)
        }
        "VolatilitySqueeze" => {
            let strat: VolatilitySqueeze =
                serde_json::from_value(strategy_record.parameters.clone())
                    .map_err(|e| AppError::Strategy(format!("Config error: {e}")))?;
            let signals = strat
                .predict(&df)
                .map_err(|e| AppError::Strategy(e.to_string()))?;
            let explanations = strat
                .explain(&df)
                .map_err(|e| AppError::Strategy(e.to_string()))?;
            (signals, explanations)
        }
        "MacdTrend" => {
            let strat: MacdTrend = serde_json::from_value(strategy_record.parameters.clone())
                .map_err(|e| AppError::Strategy(format!("Config error: {e}")))?;
            let signals = strat
                .predict(&df)
                .map_err(|e| AppError::Strategy(e.to_string()))?;
            let explanations = strat
                .explain(&df)
                .map_err(|e| AppError::Strategy(e.to_string()))?;
            (signals, explanations)
        }
        "ObvTrend" => {
            let strat: ObvTrend = serde_json::from_value(strategy_record.parameters.clone())
                .map_err(|e| AppError::Strategy(format!("Config error: {e}")))?;
            let signals = strat
                .predict(&df)
                .map_err(|e| AppError::Strategy(e.to_string()))?;
            let explanations = strat
                .explain(&df)
                .map_err(|e| AppError::Strategy(e.to_string()))?;
            (signals, explanations)
        }
        "PriceMomentum" => {
            let strat: PriceMomentum =
                serde_json::from_value(strategy_record.parameters.clone())
                    .map_err(|e| AppError::Strategy(format!("Config error: {e}")))?;
            let signals = strat
                .predict(&df)
                .map_err(|e| AppError::Strategy(e.to_string()))?;
            let explanations = strat
                .explain(&df)
                .map_err(|e| AppError::Strategy(e.to_string()))?;
            (signals, explanations)
        }
        "AdaptiveMaCrossover" => {
            let strat: AdaptiveMaCrossover =
                serde_json::from_value(strategy_record.parameters.clone())
                    .map_err(|e| AppError::Strategy(format!("Config error: {e}")))?;
            let signals = strat
                .predict(&df)
                .map_err(|e| AppError::Strategy(e.to_string()))?;
            let explanations = strat
                .explain(&df)
                .map_err(|e| AppError::Strategy(e.to_string()))?;
            (signals, explanations)
        }
        _ => {
            warn!("Unknown strategy type: {}", strategy_type);
            return Ok(());
        }
    };

    let signals = signal_series
        .f64()
        .map_err(|e| AppError::Data(e.to_string()))?;
    if signals.is_empty() {
        return Ok(());
    }
    let latest_idx = signals.len() - 1;
    let latest_signal = signals.get(latest_idx).unwrap_or(0.0);
    let latest_reason = explanation_series
        .str()
        .map_err(|e| AppError::Data(e.to_string()))?
        .get(latest_idx)
        .map(|s| s.to_string())
        .unwrap_or_else(|| "No explanation".to_string());

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

    execute_paper_trade(
        pool,
        session,
        target_signal,
        current_price,
        latest_reason,
        snapshot_tracker,
    )
    .await?;

    Ok(())
}

async fn execute_paper_trade(
    pool: &PgPool,
    session: &Session,
    signal: f64,
    price: f64,
    reason: String,
    snapshot_tracker: &mut HashMap<Uuid, DateTime<Utc>>,
) -> Result<(), AppError> {
    let now = Utc::now();
    if (signal - session.current_position).abs() < 0.1 {
        update_equity_mtm(pool, session, price, snapshot_tracker, false).await?;
        return Ok(());
    }

    info!(
        "Signal change for session {}: {} -> {} @ ${}",
        session.id, session.current_position, signal, price
    );

    let mut tx = pool.begin().await?;
    let mut new_equity = session.current_equity;

    if session.current_position.abs() > 0.0 {
        let entry_price = session.entry_price.unwrap_or(price);
        let basis_equity = session.entry_equity.unwrap_or(session.current_equity);
        let direction = if session.current_position > 0.0 {
            1.0
        } else {
            -1.0
        };
        let raw_pnl_pct = direction * (price - entry_price) / entry_price;
        let settled_equity = basis_equity * (1.0 + raw_pnl_pct);
        let pnl_amount = settled_equity - basis_equity;
        new_equity = settled_equity;

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
    let new_entry_equity = if signal.abs() > 0.0 {
        Some(new_equity)
    } else {
        None
    };

    sqlx::query(
        "UPDATE sessions SET current_equity = $1, current_position = $2, entry_price = $3, entry_equity = $4, last_update = $5 WHERE id = $6",
    )
    .bind(new_equity)
    .bind(signal)
    .bind(new_entry_price)
    .bind(new_entry_equity)
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
    snapshot_tracker.insert(session.id, now);

    Ok(())
}

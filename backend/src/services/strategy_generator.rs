use std::sync::Arc;

use anyhow::Result;
use krypto::algo::optimization::{OptimizableStrategy, Optimizer};
use krypto::algo::strategies::{
    AdaptiveMaCrossover, AtrBreakout, BollingerReversion, DynamicTrend, MacdTrend, ObvTrend,
    PriceMomentum, RsiMeanReversion, VolatilitySqueeze,
};
use krypto::backtest::engine::BacktestResult;
use krypto::features::indicators::FeatureEngine;
use polars::prelude::*;
use serde::Serialize;
use sqlx::PgPool;
use tracing::{error, info};

use crate::services::market_data::MarketDataService;

pub struct StrategyGenerator {
    pool: PgPool,
    market: Arc<MarketDataService>,
}

struct Candidate {
    symbol: String,
    interval: String,
    strategy_name: String,
    strategy_type: String,
    config_json: serde_json::Value,
    metrics: BacktestResult,
}

impl StrategyGenerator {
    pub fn new(pool: PgPool, market: Arc<MarketDataService>) -> Self {
        Self { pool, market }
    }

    pub async fn generate_and_save(
        &self,
        symbols: Vec<String>,
        intervals: Vec<String>,
        top_n: usize,
        limit: u16,
        iterations: usize,
    ) -> Result<usize> {
        info!(
            "Starting strategy generation: {} symbols, {} intervals, depth {}, iter {}",
            symbols.len(),
            intervals.len(),
            limit,
            iterations
        );

        let mut candidates = Vec::new();
        let optimizer = Optimizer::new(iterations, 0.7);

        for symbol in &symbols {
            for interval in &intervals {
                let raw_df = match self.market.fetch_candles(symbol, interval, limit).await {
                    Ok(df) => df,
                    Err(e) => {
                        error!("Failed to fetch data for {} {}: {}", symbol, interval, e);
                        continue;
                    }
                };

                let df = match FeatureEngine::add_technicals(&raw_df, None) {
                    Ok(df) => df,
                    Err(e) => {
                        error!("Feature calc failed for {} {}: {}", symbol, interval, e);
                        continue;
                    }
                };

                self.evaluate_type::<DynamicTrend>(
                    &optimizer,
                    &df,
                    symbol,
                    interval,
                    "DynamicTrend",
                    &mut candidates,
                )?;
                self.evaluate_type::<RsiMeanReversion>(
                    &optimizer,
                    &df,
                    symbol,
                    interval,
                    "RsiMeanReversion",
                    &mut candidates,
                )?;
                self.evaluate_type::<BollingerReversion>(
                    &optimizer,
                    &df,
                    symbol,
                    interval,
                    "BollingerReversion",
                    &mut candidates,
                )?;
                self.evaluate_type::<AtrBreakout>(
                    &optimizer,
                    &df,
                    symbol,
                    interval,
                    "AtrBreakout",
                    &mut candidates,
                )?;
                self.evaluate_type::<VolatilitySqueeze>(
                    &optimizer,
                    &df,
                    symbol,
                    interval,
                    "VolatilitySqueeze",
                    &mut candidates,
                )?;
                self.evaluate_type::<MacdTrend>(
                    &optimizer,
                    &df,
                    symbol,
                    interval,
                    "MacdTrend",
                    &mut candidates,
                )?;
                self.evaluate_type::<ObvTrend>(
                    &optimizer,
                    &df,
                    symbol,
                    interval,
                    "ObvTrend",
                    &mut candidates,
                )?;
                self.evaluate_type::<PriceMomentum>(
                    &optimizer,
                    &df,
                    symbol,
                    interval,
                    "PriceMomentum",
                    &mut candidates,
                )?;
                self.evaluate_type::<AdaptiveMaCrossover>(
                    &optimizer,
                    &df,
                    symbol,
                    interval,
                    "AdaptiveMaCrossover",
                    &mut candidates,
                )?;
            }
        }

        candidates.sort_by(|a, b| {
            b.metrics
                .sharpe_ratio
                .partial_cmp(&a.metrics.sharpe_ratio)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut saved_count = 0usize;
        for cand in candidates.into_iter().take(top_n) {
            let kelly_fraction = cand.metrics.kelly_fraction;

            let metrics_json = serde_json::json!({
                "sharpe": cand.metrics.sharpe_ratio,
                "total_return_pct": cand.metrics.total_return_pct,
                "max_drawdown_pct": cand.metrics.max_drawdown_pct,
                "win_rate": cand.metrics.win_rate,
                "profit_factor": cand.metrics.profit_factor,
                "trades": cand.metrics.total_trades
            });

            let curve = &cand.metrics.equity_curve;
            let step = (curve.len() / 50).max(1);
            let mut downsampled: Vec<f64> = curve.iter().step_by(step).copied().collect();
            if let Some(last) = curve.last().copied() {
                if downsampled.last().copied() != Some(last) {
                    downsampled.push(last);
                }
            }
            let curve_json = serde_json::to_value(&downsampled)?;

            let name = format!("{} {} {}", cand.symbol, cand.interval, cand.strategy_name);

            sqlx::query(
                r#"
                INSERT INTO strategies
                (name, strategy_type, symbol, interval, parameters, performance_metrics, backtest_curve, kelly_fraction)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(name)
            .bind(&cand.strategy_type)
            .bind(&cand.symbol)
            .bind(&cand.interval)
            .bind(cand.config_json)
            .bind(metrics_json)
            .bind(curve_json)
            .bind(kelly_fraction)
            .execute(&self.pool)
            .await?;

            saved_count += 1;
        }

        info!("Saved {} optimized strategies", saved_count);
        Ok(saved_count)
    }

    fn evaluate_type<S>(
        &self,
        optimizer: &Optimizer,
        df: &DataFrame,
        symbol: &str,
        interval: &str,
        type_name: &str,
        candidates: &mut Vec<Candidate>,
    ) -> Result<()>
    where
        S: OptimizableStrategy + Clone + Default + Serialize,
    {
        let mut strat = S::default();
        let (_, best_result) = optimizer.optimize(&mut strat, df);

        if let Some(res) = best_result {
            if res.total_trades > 10 && res.total_return_pct > 0.0 {
                let config_json = serde_json::to_value(&strat)?;

                let strategy_name = strat.name().to_string();
                candidates.push(Candidate {
                    symbol: symbol.to_string(),
                    interval: interval.to_string(),
                    strategy_name,
                    strategy_type: type_name.to_string(),
                    config_json,
                    metrics: res,
                });
            }
        }
        Ok(())
    }
}

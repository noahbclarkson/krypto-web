//! Market data fetching from Binance REST API.

use binance::{api::Binance, market::Market, rest_model::KlineSummaries};
use chrono::{DateTime, Utc};
use polars::prelude::*;

use crate::error::AppError;

/// Thin wrapper around the Binance market REST client.
pub struct MarketDataService {
    market: Market,
}

/// A single OHLCV candlestick bar.
#[derive(Clone, Debug)]
pub struct CandleBar {
    /// Open time as Unix timestamp in milliseconds.
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

impl MarketDataService {
    /// Create a new service.  API keys are optional for public market data.
    pub fn new(api_key: Option<String>, secret_key: Option<String>) -> Self {
        let market: Market = Binance::new(api_key, secret_key);
        Self { market }
    }

    /// Fetch the most recent `limit` OHLCV candles as a Polars [`DataFrame`].
    ///
    /// Columns: `time`, `open`, `high`, `low`, `close`, `volume`.
    pub async fn fetch_candles(
        &self,
        symbol: &str,
        interval: &str,
        limit: u16,
    ) -> Result<DataFrame, AppError> {
        let klines = self
            .market
            .get_klines(symbol, interval, Some(limit), None, None)
            .await
            .map_err(|e| AppError::Binance(e.to_string()))?;

        let KlineSummaries::AllKlineSummaries(data) = klines;

        let mut times = Vec::with_capacity(data.len());
        let mut opens = Vec::with_capacity(data.len());
        let mut highs = Vec::with_capacity(data.len());
        let mut lows = Vec::with_capacity(data.len());
        let mut closes = Vec::with_capacity(data.len());
        let mut volumes = Vec::with_capacity(data.len());

        for k in data {
            let dt = DateTime::<Utc>::from_timestamp_millis(k.open_time)
                .map(|d| d.naive_utc())
                .unwrap_or_else(|| Utc::now().naive_utc());
            times.push(dt);
            opens.push(k.open);
            highs.push(k.high);
            lows.push(k.low);
            closes.push(k.close);
            volumes.push(k.volume);
        }

        let df = df!(
            "time" => times,
            "open" => opens,
            "high" => highs,
            "low" => lows,
            "close" => closes,
            "volume" => volumes
        )
        .map_err(|e| AppError::Data(e.to_string()))?;

        Ok(df)
    }

    /// Fetch the most recent `limit` candles as a `Vec<CandleBar>`.
    ///
    /// Lighter than [`fetch_candles`](Self::fetch_candles) when a full
    /// DataFrame is not required (e.g. live charting endpoint).
    pub async fn fetch_candles_vec(
        &self,
        symbol: &str,
        interval: &str,
        limit: u16,
    ) -> Result<Vec<CandleBar>, AppError> {
        let klines = self
            .market
            .get_klines(symbol, interval, Some(limit), None, None)
            .await
            .map_err(|e| AppError::Binance(e.to_string()))?;

        let KlineSummaries::AllKlineSummaries(data) = klines;
        let mut out = Vec::with_capacity(data.len());

        for k in data {
            out.push(CandleBar {
                time: k.open_time,
                open: k.open,
                high: k.high,
                low: k.low,
                close: k.close,
            });
        }

        Ok(out)
    }
}

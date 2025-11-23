use binance::{api::Binance, market::Market, rest_model::KlineSummaries};
use chrono::{DateTime, Utc};
use polars::prelude::*;

use crate::error::AppError;

pub struct MarketDataService {
    market: Market,
}

impl MarketDataService {
    pub fn new(api_key: Option<String>, secret_key: Option<String>) -> Self {
        let market: Market = Binance::new(api_key, secret_key);
        Self { market }
    }

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
}

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Duration as ChronoDuration, Timelike, Utc};
use sqlx::{FromRow, PgPool, QueryBuilder};
use tracing::{error, info};
use uuid::Uuid;

#[derive(FromRow)]
struct SnapshotRow {
    session_id: Uuid,
    equity: f64,
    timestamp: DateTime<Utc>,
}

pub struct PortfolioManager {
    pool: PgPool,
}

impl PortfolioManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn start_background_task(self: Arc<Self>) {
        info!("Portfolio Manager started. Syncing cache every 60s.");

        if let Err(e) = self.update_cache().await {
            error!("Initial portfolio cache update failed: {}", e);
        }

        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Err(e) = self.update_cache().await {
                error!("Portfolio cache update failed: {}", e);
            }
        }
    }

    async fn update_cache(&self) -> Result<(), sqlx::Error> {
        let snapshots = sqlx::query_as::<_, SnapshotRow>(
            "SELECT session_id, equity, timestamp FROM equity_snapshots ORDER BY timestamp ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        if snapshots.is_empty() {
            return Ok(());
        }

        let start_time = snapshots[0].timestamp
            .with_second(0).unwrap()
            .with_nanosecond(0).unwrap();
        let end_time = Utc::now();

        let mut current_equities: HashMap<Uuid, f64> = HashMap::new();
        let mut cache_points: Vec<(DateTime<Utc>, f64)> = Vec::with_capacity(10000);
        let mut snapshot_idx = 0;
        let mut curr = start_time;

        while curr <= end_time {
            while snapshot_idx < snapshots.len() && snapshots[snapshot_idx].timestamp <= curr {
                let snap = &snapshots[snapshot_idx];
                current_equities.insert(snap.session_id, snap.equity);
                snapshot_idx += 1;
            }

            let total: f64 = current_equities.values().sum();

            if total > 0.0 {
                cache_points.push((curr, total));
            }

            curr += ChronoDuration::minutes(1);
        }

        if cache_points.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;

        sqlx::query("TRUNCATE TABLE portfolio_cache").execute(&mut *tx).await?;

        for chunk in cache_points.chunks(5000) {
            let mut query_builder = QueryBuilder::new(
                "INSERT INTO portfolio_cache (timestamp, total_equity) "
            );

            query_builder.push_values(chunk, |mut b, (ts, eq)| {
                b.push_bind(ts)
                 .push_bind(eq);
            });

            query_builder.build().execute(&mut *tx).await?;
        }

        tx.commit().await?;

        info!("Updated portfolio cache with {} data points", cache_points.len());
        Ok(())
    }
}

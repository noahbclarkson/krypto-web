use std::sync::Arc;

use actix_web::{delete, get, post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::strategy::{
    CreateSessionRequest, CreateStrategyRequest, GenerateStrategiesRequest, Session, Strategy,
    Trade,
};
use crate::services::market_data::MarketDataService;
use crate::services::strategy_generator::StrategyGenerator;

#[post("/strategies/generate")]
async fn generate_strategies(
    generator: web::Data<Arc<StrategyGenerator>>,
    body: web::Json<GenerateStrategiesRequest>,
) -> Result<impl Responder, AppError> {
    let req = body.into_inner();
    let top_n = req.top_n.unwrap_or(10);
    let limit = req.limit.unwrap_or(1000);
    let iterations = req.iterations.unwrap_or(50);

    let count = generator
        .generate_and_save(req.symbols, req.intervals, top_n, limit, iterations)
        .await
        .map_err(|e| AppError::Strategy(e.to_string()))?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Generation complete",
        "strategies_created": count
    })))
}

#[post("/strategies")]
async fn create_strategy(
    pool: web::Data<PgPool>,
    body: web::Json<CreateStrategyRequest>,
) -> Result<impl Responder, AppError> {
    let CreateStrategyRequest {
        name,
        strategy_type,
        symbol,
        interval,
        parameters,
        performance_metrics,
        backtest_curve,
    } = body.into_inner();

    let rec = sqlx::query_as::<_, Strategy>("INSERT INTO strategies (name, strategy_type, symbol, interval, parameters, performance_metrics, backtest_curve) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *")
        .bind(name)
        .bind(strategy_type)
        .bind(symbol)
        .bind(interval)
        .bind(parameters)
        .bind(performance_metrics)
        .bind(backtest_curve)
        .fetch_one(pool.get_ref())
        .await?;

    Ok(HttpResponse::Ok().json(rec))
}

#[get("/strategies")]
async fn list_strategies(pool: web::Data<PgPool>) -> Result<impl Responder, AppError> {
    let recs = sqlx::query_as::<_, Strategy>("SELECT * FROM strategies ORDER BY created_at DESC")
        .fetch_all(pool.get_ref())
        .await?;
    Ok(HttpResponse::Ok().json(recs))
}

#[delete("/strategies/{id}")]
async fn delete_strategy(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> Result<impl Responder, AppError> {
    let strategy_id = path.into_inner();
    let mut tx = pool.begin().await?;

    sqlx::query(
        "DELETE FROM trades WHERE session_id IN (SELECT id FROM sessions WHERE strategy_id = $1)",
    )
    .bind(strategy_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM equity_snapshots WHERE session_id IN (SELECT id FROM sessions WHERE strategy_id = $1)")
        .bind(strategy_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM sessions WHERE strategy_id = $1")
        .bind(strategy_id)
        .execute(&mut *tx)
        .await?;

    let res = sqlx::query("DELETE FROM strategies WHERE id = $1")
        .bind(strategy_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound("Strategy not found".into()));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Deleted" })))
}

#[delete("/strategies")]
async fn delete_all_strategies(pool: web::Data<PgPool>) -> Result<impl Responder, AppError> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM trades").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM equity_snapshots")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM sessions")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM strategies")
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "All strategies deleted" })))
}

#[post("/sessions")]
async fn start_session(
    pool: web::Data<PgPool>,
    body: web::Json<CreateSessionRequest>,
) -> Result<impl Responder, AppError> {
    let req = body.into_inner();
    let strategy = sqlx::query_as::<_, Strategy>("SELECT * FROM strategies WHERE id = $1")
        .bind(req.strategy_id)
        .fetch_one(pool.get_ref())
        .await?;

    let initial_capital = req.initial_capital;
    let execution_mode = req.execution_mode.unwrap_or_else(|| "sync".to_string());

    let rec = sqlx::query_as::<_, Session>(
        "INSERT INTO sessions (strategy_id, symbol, interval, initial_capital, current_equity, execution_mode) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
    )
    .bind(strategy.id)
    .bind(strategy.symbol)
    .bind(strategy.interval)
    .bind(initial_capital)
    .bind(initial_capital)
    .bind(execution_mode)
    .fetch_one(pool.get_ref())
    .await?;

    sqlx::query(
        "INSERT INTO equity_snapshots (session_id, equity, timestamp) VALUES ($1, $2, NOW())",
    )
    .bind(rec.id)
    .bind(rec.initial_capital)
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(rec))
}

#[derive(serde::Deserialize)]
struct BulkSessionRequest {
    strategy_ids: Vec<Uuid>,
}

#[post("/sessions/bulk")]
async fn bulk_start_session(
    pool: web::Data<PgPool>,
    body: web::Json<BulkSessionRequest>,
) -> Result<impl Responder, AppError> {
    let ids = body.into_inner().strategy_ids;
    let mut created_count = 0;

    for strategy_id in ids {
        if let Ok(strategy) =
            sqlx::query_as::<_, Strategy>("SELECT * FROM strategies WHERE id = $1")
                .bind(strategy_id)
                .fetch_one(pool.get_ref())
                .await
        {
            let _ = sqlx::query(
                "INSERT INTO sessions (strategy_id, symbol, interval) VALUES ($1, $2, $3)",
            )
            .bind(strategy.id)
            .bind(strategy.symbol)
            .bind(strategy.interval)
            .execute(pool.get_ref())
            .await;

            created_count += 1;
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Bulk sessions started",
        "count": created_count
    })))
}

#[get("/sessions")]
async fn list_sessions(pool: web::Data<PgPool>) -> Result<impl Responder, AppError> {
    let recs = sqlx::query_as::<_, Session>("SELECT * FROM sessions ORDER BY created_at DESC")
        .fetch_all(pool.get_ref())
        .await?;
    Ok(HttpResponse::Ok().json(recs))
}

#[post("/sessions/reset")]
async fn reset_sessions(pool: web::Data<PgPool>) -> Result<impl Responder, AppError> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM trades WHERE session_id IN (SELECT id FROM sessions)")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM equity_snapshots WHERE session_id IN (SELECT id FROM sessions)")
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM sessions")
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Sessions reset" })))
}

#[get("/sessions/{id}/trades")]
async fn get_trades(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> Result<impl Responder, AppError> {
    let id = path.into_inner();
    let recs = sqlx::query_as::<_, Trade>(
        "SELECT * FROM trades WHERE session_id = $1 ORDER BY timestamp DESC",
    )
    .bind(id)
    .fetch_all(pool.get_ref())
    .await?;
    Ok(HttpResponse::Ok().json(recs))
}

#[derive(serde::Serialize, sqlx::FromRow)]
struct Snapshot {
    equity: f64,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[get("/sessions/{id}/equity")]
async fn get_equity_curve(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> Result<impl Responder, AppError> {
    let id = path.into_inner();
    let recs = sqlx::query_as::<_, Snapshot>(
        "SELECT equity, timestamp FROM equity_snapshots WHERE session_id = $1 ORDER BY timestamp ASC",
    )
    .bind(id)
    .fetch_all(pool.get_ref())
    .await?;
    Ok(HttpResponse::Ok().json(recs))
}

#[derive(serde::Serialize, sqlx::FromRow)]
struct PortfolioPoint {
    timestamp: DateTime<Utc>,
    total_equity: f64,
}

#[derive(serde::Deserialize)]
struct PortfolioQuery {
    range_days: Option<i64>,
    interval: Option<String>,
}

#[derive(serde::Serialize)]
struct CandleBar {
    time: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

#[get("/portfolio/history")]
async fn get_portfolio_history(
    pool: web::Data<PgPool>,
    query: web::Query<PortfolioQuery>,
) -> Result<impl Responder, AppError> {
    let range_days = query.range_days.unwrap_or(7).max(1);
    let (bucket_expr, bucket_start_expr, bucket_end_expr, step_expr) = match query
        .interval
        .as_deref()
        .unwrap_or("15m")
    {
        "3m" => (
            "date_trunc('minute', es.timestamp) - ((extract(minute from es.timestamp)::int % 3) * interval '1 minute')",
            "date_trunc('minute', start_ts) - ((extract(minute from start_ts)::int % 3) * interval '1 minute')",
            "date_trunc('minute', end_ts) - ((extract(minute from end_ts)::int % 3) * interval '1 minute')",
            "3 minutes",
        ),
        "15m" => (
            "date_trunc('minute', es.timestamp) - ((extract(minute from es.timestamp)::int % 15) * interval '1 minute')",
            "date_trunc('minute', start_ts) - ((extract(minute from start_ts)::int % 15) * interval '1 minute')",
            "date_trunc('minute', end_ts) - ((extract(minute from end_ts)::int % 15) * interval '1 minute')",
            "15 minutes",
        ),
        "1h" => (
            "date_trunc('hour', es.timestamp)",
            "date_trunc('hour', start_ts)",
            "date_trunc('hour', end_ts)",
            "1 hour",
        ),
        "1d" => (
            "date_trunc('day', es.timestamp)",
            "date_trunc('day', start_ts)",
            "date_trunc('day', end_ts)",
            "1 day",
        ),
        _ => (
            "date_trunc('minute', es.timestamp) - ((extract(minute from es.timestamp)::int % 15) * interval '1 minute')",
            "date_trunc('minute', start_ts) - ((extract(minute from start_ts)::int % 15) * interval '1 minute')",
            "date_trunc('minute', end_ts) - ((extract(minute from end_ts)::int % 15) * interval '1 minute')",
            "15 minutes",
        ),
    };

    let sql = format!(
        r#"
        WITH params AS (
            SELECT
                NOW() - ($1 * interval '1 day') AS start_ts,
                NOW() AS end_ts
        ),
        bounds AS (
            SELECT
                {start_bucket} AS start_bucket,
                {end_bucket} AS end_bucket
            FROM params
        ),
        buckets AS (
            SELECT generate_series(start_bucket, end_bucket, interval '{step}') AS bucket
            FROM bounds
        ),
        bucketed AS (
            SELECT
                es.session_id,
                {bucket} AS bucket,
                es.equity,
                row_number() OVER (
                    PARTITION BY es.session_id, {bucket}
                    ORDER BY es.timestamp DESC
                ) AS rn
            FROM equity_snapshots es
            WHERE es.timestamp >= (SELECT start_ts FROM params)
        )
        SELECT
            bucket AS timestamp,
            ROUND(SUM(equity)::numeric, 2)::float8 AS total_equity
        FROM (
            SELECT
                sb.session_id,
                sb.bucket,
                (
                    SELECT d.equity
                    FROM bucketed d
                    WHERE d.session_id = sb.session_id
                      AND d.rn = 1
                      AND d.bucket <= sb.bucket
                    ORDER BY d.bucket DESC
                    LIMIT 1
                ) AS equity
            FROM (
                SELECT DISTINCT es.session_id, b.bucket
                FROM equity_snapshots es
                CROSS JOIN buckets b
                WHERE es.timestamp >= (SELECT start_ts FROM params)
            ) sb
        ) filled
        WHERE equity IS NOT NULL
        GROUP BY bucket
        ORDER BY bucket ASC
        "#,
        bucket = bucket_expr,
        start_bucket = bucket_start_expr,
        end_bucket = bucket_end_expr,
        step = step_expr
    );

    let recs = sqlx::query_as::<_, PortfolioPoint>(&sql)
        .bind(range_days)
        .fetch_all(pool.get_ref())
        .await?;

    Ok(HttpResponse::Ok().json(recs))
}

#[get("/sessions/{id}/candles")]
async fn get_session_candles(
    pool: web::Data<PgPool>,
    market: web::Data<Arc<MarketDataService>>,
    path: web::Path<Uuid>,
) -> Result<impl Responder, AppError> {
    let id = path.into_inner();
    let session = sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = $1")
        .bind(id)
        .fetch_one(pool.get_ref())
        .await?;

    let raw = market
        .fetch_candles_vec(&session.symbol, &session.interval, 300)
        .await?;

    let candles: Vec<CandleBar> = raw
        .into_iter()
        .map(|c| CandleBar {
            time: DateTime::<Utc>::from_timestamp_millis(c.time)
                .unwrap_or_else(|| Utc::now())
                .to_rfc3339(),
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
        })
        .collect();

    Ok(HttpResponse::Ok().json(candles))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(generate_strategies)
        .service(create_strategy)
        .service(list_strategies)
        .service(delete_strategy)
        .service(delete_all_strategies)
        .service(start_session)
        .service(bulk_start_session)
        .service(list_sessions)
        .service(reset_sessions)
        .service(get_trades)
        .service(get_equity_curve)
        .service(get_session_candles)
        .service(get_portfolio_history);
}

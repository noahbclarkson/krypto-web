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

#[get("/portfolio/history")]
async fn get_portfolio_history(
    pool: web::Data<PgPool>,
    query: web::Query<PortfolioQuery>,
) -> Result<impl Responder, AppError> {
    let range_days = query.range_days.unwrap_or(7).max(1);
    let bucket_expr = match query
        .interval
        .as_deref()
        .unwrap_or("1h")
    {
        "3m" => "date_trunc('minute', timestamp) - ((extract(minute from timestamp)::int % 3) * interval '1 minute')",
        "15m" => "date_trunc('minute', timestamp) - ((extract(minute from timestamp)::int % 15) * interval '1 minute')",
        "1d" => "date_trunc('day', timestamp)",
        _ => "date_trunc('hour', timestamp)",
    };

    let sql = format!(
        r#"
        WITH bucketed AS (
            SELECT
                es.session_id,
                {bucket} AS bucket,
                es.equity,
                row_number() OVER (
                    PARTITION BY es.session_id, {bucket}
                    ORDER BY es.timestamp DESC
                ) AS rn
            FROM equity_snapshots es
            JOIN sessions s ON es.session_id = s.id AND s.status = 'active'
            WHERE es.timestamp >= NOW() - ($1 * interval '1 day')
        )
        SELECT
            bucket AS timestamp,
            SUM(equity) AS total_equity
        FROM bucketed
        WHERE rn = 1
        GROUP BY bucket
        ORDER BY bucket ASC
        "#,
        bucket = bucket_expr
    );

    let recs = sqlx::query_as::<_, PortfolioPoint>(&sql)
        .bind(range_days)
        .fetch_all(pool.get_ref())
        .await?;

    Ok(HttpResponse::Ok().json(recs))
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
        .service(get_portfolio_history);
}

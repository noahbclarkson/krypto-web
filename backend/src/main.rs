mod config;
mod db;
mod error;
mod handlers;
mod models;
mod services;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use config::Config;
use services::market_data::MarketDataService;
use services::portfolio_manager::PortfolioManager;
use services::strategy_generator::StrategyGenerator;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    std::env::set_var("RUST_LOG", "debug,actix_web=debug,actix_server=info");

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = Config::from_env();
    let pool = db::create_pool(&config.database_url)
        .await
        .expect("Failed to connect to DB");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let market_service = Arc::new(MarketDataService::new(
        config.binance_api_key.clone(),
        config.binance_secret_key.clone(),
    ));
    let generator_service = Arc::new(StrategyGenerator::new(pool.clone(), market_service.clone()));
    let portfolio_manager = Arc::new(PortfolioManager::new(pool.clone()));

    let engine_pool = pool.clone();
    let engine_market = market_service.clone();
    tokio::spawn(async move {
        services::trading_engine::start_engine(engine_pool, engine_market).await;
    });

    let pm_clone = portfolio_manager.clone();
    tokio::spawn(async move {
        pm_clone.start_background_task().await;
    });

    info!("Server starting at {}", config.server_addr);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(market_service.clone()))
            .app_data(web::Data::new(generator_service.clone()))
            .configure(handlers::trade_handler::config)
    })
    .bind(&config.server_addr)?
    .run()
    .await
}

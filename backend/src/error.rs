//! Error types for the Krypto Web API.
//!
//! All handler errors funnel through [`AppError`], which implements
//! [`actix_web::ResponseError`] to produce consistent JSON error bodies.

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

/// Top-level API error type.
#[derive(Error, Debug)]
pub enum AppError {
    /// Wraps sqlx database errors.
    #[error("Database Error: {0}")]
    Database(#[from] sqlx::Error),
    /// Binance API or WebSocket errors.
    #[error("Binance API Error: {0}")]
    Binance(String),
    /// Strategy generation / optimisation errors.
    #[error("Strategy Error: {0}")]
    Strategy(String),
    /// Data processing / feature engineering errors.
    #[error("Data Processing Error: {0}")]
    Data(String),
    /// Resource not found (returns 404).
    #[allow(dead_code)]
    #[error("Not Found: {0}")]
    NotFound(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorResponse {
            error: self.to_string(),
        })
    }
}

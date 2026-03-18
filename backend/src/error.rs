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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;

    #[test]
    fn test_error_display() {
        let err = AppError::Binance("connection timeout".to_string());
        assert_eq!(err.to_string(), "Binance API Error: connection timeout");

        let err = AppError::Strategy("invalid parameters".to_string());
        assert_eq!(err.to_string(), "Strategy Error: invalid parameters");

        let err = AppError::Data("missing column".to_string());
        assert_eq!(err.to_string(), "Data Processing Error: missing column");

        let err = AppError::NotFound("session 123".to_string());
        assert_eq!(err.to_string(), "Not Found: session 123");
    }

    #[test]
    fn test_status_codes() {
        assert_eq!(AppError::NotFound("test".to_string()).status_code(), StatusCode::NOT_FOUND);
        assert_eq!(AppError::Binance("test".to_string()).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(AppError::Strategy("test".to_string()).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(AppError::Data("test".to_string()).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_response() {
        let err = AppError::NotFound("session 123".to_string());
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_from_sqlx_error() {
        let sqlx_err = sqlx::Error::RowNotFound;
        let app_err: AppError = sqlx_err.into();
        assert!(matches!(app_err, AppError::Database(_)));
    }
}

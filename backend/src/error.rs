use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database Error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Binance API Error: {0}")]
    Binance(String),
    #[error("Strategy Error: {0}")]
    Strategy(String),
    #[error("Data Processing Error: {0}")]
    Data(String),
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

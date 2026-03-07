//! Database connection utilities.

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// Create a PostgreSQL connection pool with default settings.
///
/// # Arguments
///
/// * `database_url` - PostgreSQL connection string
///
/// # Returns
///
/// A `PgPool` with max 5 connections.
///
/// # Example
///
/// ```rust,no_run
/// let pool = create_pool("postgres://user:pass@localhost/db").await?;
/// ```
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}

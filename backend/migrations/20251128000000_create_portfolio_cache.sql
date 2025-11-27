CREATE TABLE portfolio_cache (
    timestamp TIMESTAMPTZ NOT NULL PRIMARY KEY,
    total_equity DOUBLE PRECISION NOT NULL
);

CREATE INDEX idx_portfolio_cache_ts_equity ON portfolio_cache(timestamp, total_equity);

# Krypto Web — Backend

Rust/Actix-web backend for the Krypto trading platform. Manages strategy generation, paper trading sessions, live market data, and portfolio tracking.

## Stack

- **Runtime**: Rust / Actix-web
- **Database**: PostgreSQL via sqlx
- **Market Data**: Binance REST + WebSocket
- **Optimisation**: Genetic algorithm over strategy parameter space

## Architecture

```
┌────────────────────────────────────────────────────────┐
│                    HTTP API (:8080)                     │
│  /strategies   /sessions   /portfolio   /sessions/:id  │
└────────────────────────┬───────────────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         ▼               ▼               ▼
 StrategyGenerator  TradingEngine  PortfolioManager
  (genetic algo)   (bar-close      (equity snapshots
                    execution)      + cache)
         │               │               │
         └───────────────┴───────────────┘
                         │
                    PostgreSQL
```

## Services

| Service | Responsibility |
|---------|---------------|
| `StrategyGenerator` | Genetic algorithm optimisation over indicator parameters; saves top-N strategies to DB |
| `TradingEngine` | Subscribes to Binance WebSocket bar close events; evaluates strategies and places paper trades |
| `PortfolioManager` | Aggregates session equity into `portfolio_cache` for charting |
| `MarketDataService` | Fetches OHLCV candles from Binance REST API |

## Prerequisites

- Rust 1.80+
- PostgreSQL 13+
- Binance API key (optional — only needed for live order placement; paper trading works without)

## Setup

### 1. Database

```bash
createdb krypto
```

Migrations run automatically on startup via `sqlx::migrate!`.

### 2. Environment Variables

Create a `.env` file in `backend/`:

```env
# Required
DATABASE_URL=postgres://user:pass@localhost/krypto

# Optional
SERVER_ADDR=0.0.0.0:8080

# Only needed for live trading
BINANCE_API_KEY=your_key
BINANCE_SECRET_KEY=your_secret
```

### 3. Run

```bash
cargo run
```

Server starts on `0.0.0.0:8080`.

## API Reference

### Strategies

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/strategies/generate` | Run optimiser; generates and saves top-N strategies |
| `GET` | `/strategies` | List all saved strategies |
| `DELETE` | `/strategies` | Delete all strategies (and cascade sessions/trades) |
| `DELETE` | `/strategies/:id` | Delete a single strategy |

**POST /strategies/generate body:**
```json
{
  "symbols": ["BTCUSDT", "ETHUSDT"],
  "intervals": ["1h", "4h"],
  "top_n": 10,
  "limit": 1000,
  "iterations": 50
}
```

### Sessions

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/sessions` | Start a paper trading session for a strategy |
| `POST` | `/sessions/bulk` | Start sessions for multiple strategies at once |
| `GET` | `/sessions` | List all sessions |
| `POST` | `/sessions/reset` | Stop all sessions and clear history |
| `GET` | `/sessions/:id/trades` | Trade history for a session |
| `GET` | `/sessions/:id/equity` | Equity curve snapshots |
| `GET` | `/sessions/:id/candles` | Live OHLCV candles for the session's symbol |

**POST /sessions body:**
```json
{
  "strategy_id": "uuid",
  "initial_capital": 10000.0
}
```

### Portfolio

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/portfolio/history` | Aggregate equity curve across all sessions |

**Query params:** `range_days`, `interval` (e.g. `15m`), `style` (`line` or `candle`).

## Database Schema

Core tables (created automatically by migrations):

```
strategies       — saved optimised strategies with backtest metrics
sessions         — paper trading sessions (active / stopped)
trades           — individual trade events per session
equity_snapshots — point-in-time equity for each session
portfolio_cache  — aggregate portfolio equity over time
```

## Building for Production

```bash
cargo build --release
./target/release/backend
```

## Notes

- **Paper trading only** — no real orders are placed regardless of API key presence
- The trading engine fires on bar close; latency is one candle interval
- `portfolio_cache` is updated every 30 s by `PortfolioManager`

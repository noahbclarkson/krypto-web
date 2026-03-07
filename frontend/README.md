# Krypto Web Frontend

Web UI for Krypto V6 - a Rust-powered algorithmic trading platform.

## Tech Stack

- **Framework:** Next.js 16 (App Router)
- **UI:** React 19, shadcn/ui, Tailwind CSS
- **State:** TanStack Query
- **Charts:** Recharts, lightweight-charts

## Architecture

This is the frontend component that connects to a Rust/Actix-web backend.

```
┌─────────────────┐     ┌──────────────────┐
│  Next.js UI     │────▶│  Rust Backend    │
│  (port 3001)    │     │  (port 8080)     │
└─────────────────┘     └──────────────────┘
                               │
                        ┌──────▼──────┐
                        │ PostgreSQL  │
                        └─────────────┘
```

## Prerequisites

1. Node.js 18+
2. PostgreSQL database
3. Running backend (see `../backend/`)

## Getting Started

### 1. Install Dependencies

```bash
cd frontend
npm install
```

### 2. Configure Environment

Create a `.env.local` file:

```env
# Backend API URL (defaults to http://localhost:8080)
NEXT_PUBLIC_API_URL=http://localhost:8080
```

### 3. Run Development Server

```bash
npm run dev
```

Open [http://localhost:3001](http://localhost:3001) to see the application.

## Features

### Optimizer Config
- Configure trading symbols (tickers)
- Select timeframes (15m, 30m, 1h, 4h, 12h, 1d)
- Set history depth (candles to analyze)
- Configure optimizer iterations
- Run genetic algorithm optimization

### Strategy Pool
- View optimized strategies
- Kelly fraction allocation
- Backtest performance charts
- Sharpe ratio, win rate, max drawdown metrics

### Live Trading
- Deploy portfolio with selected strategies
- Real-time equity tracking
- Position monitoring
- Trade history

### Risk Metrics
- Value at Risk (95%)
- Max Drawdown
- Daily Volatility
- Capital Allocation visualization

## Building for Production

```bash
npm run build
npm start
```

## API Endpoints (Backend)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/strategies/generate` | Generate strategies |
| GET | `/strategies` | List all strategies |
| DELETE | `/strategies` | Delete all strategies |
| DELETE | `/strategies/{id}` | Delete single strategy |
| POST | `/sessions` | Start trading session |
| POST | `/sessions/bulk` | Start multiple sessions |
| GET | `/sessions` | List active sessions |
| POST | `/sessions/reset` | Reset all sessions |
| GET | `/sessions/{id}/trades` | Get session trades |
| GET | `/sessions/{id}/equity` | Get equity curve |
| GET | `/sessions/{id}/candles` | Get session candles |
| GET | `/portfolio/history` | Get portfolio history |

## Learn More

- [Next.js Documentation](https://nextjs.org/docs)
- [shadcn/ui](https://ui.shadcn.com)
- [lightweight-charts](https://tradingview.github.io/lightweight-charts/)

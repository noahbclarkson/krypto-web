export interface Strategy {
  id: string;
  name: string;
  strategy_type: string;
  symbol: string;
  interval: string;
  // We treat parameters as a generic object since they vary per strategy
  parameters: Record<string, any>;
  performance_metrics: {
    sharpe: number;
    win_rate: number;
    total_return_pct: number;
    max_drawdown_pct: number;
    trades: number;
    profit_factor: number;
  };
  // Array of equity values for the sparkline
  backtest_curve: number[];
  kelly_fraction: number;
  created_at: string;
}

export interface Session {
  id: string;
  strategy_id: string;
  symbol: string;
  interval: string;
  initial_capital: number;
  current_equity: number;
  current_position: number;
  entry_price: number | null;
  status: "active" | "stopped";
  created_at: string;
  last_update: string;
}

export interface Trade {
  id: string;
  session_id: string;
  symbol: string;
  side: "BUY" | "SELL";
  price: number;
  quantity: number;
  pnl: number | null;
  reason: string | null;
  timestamp: string;
}

export interface EquitySnapshot {
  equity: number;
  timestamp: string;
}

export interface PortfolioPoint {
  timestamp: string;
  total_equity: number;
}

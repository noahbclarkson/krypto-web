"use client";

import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ShieldAlert, TrendingDown, Wallet, Activity } from "lucide-react";

function calculateMaxDrawdown(equityCurve: { total_equity: number }[]): number {
  if (!equityCurve || equityCurve.length === 0) return 0;

  let maxEquity = equityCurve[0].total_equity;
  let maxDrawdown = 0;

  for (const point of equityCurve) {
    if (point.total_equity > maxEquity) {
      maxEquity = point.total_equity;
    }
    const drawdown = ((maxEquity - point.total_equity) / maxEquity) * 100;
    if (drawdown > maxDrawdown) {
      maxDrawdown = drawdown;
    }
  }

  return maxDrawdown;
}

function calculateVolatility(equityCurve: { total_equity: number }[]): number {
  if (!equityCurve || equityCurve.length < 2) return 0;

  const returns: number[] = [];
  for (let i = 1; i < equityCurve.length; i++) {
    const ret = (equityCurve[i].total_equity - equityCurve[i - 1].total_equity) / equityCurve[i - 1].total_equity;
    returns.push(ret);
  }

  const meanReturn = returns.reduce((sum, r) => sum + r, 0) / returns.length;
  const variance = returns.reduce((sum, r) => sum + Math.pow(r - meanReturn, 2), 0) / returns.length;
  const volatility = Math.sqrt(variance) * 100;

  return volatility;
}

function calculateVaR(equityCurve: { total_equity: number }[], confidence: number = 0.95): number {
  if (!equityCurve || equityCurve.length < 2) return 0;

  const returns: number[] = [];
  for (let i = 1; i < equityCurve.length; i++) {
    returns.push(equityCurve[i].total_equity - equityCurve[i - 1].total_equity);
  }

  returns.sort((a, b) => a - b);
  const index = Math.floor((1 - confidence) * returns.length);
  return returns[index] || 0;
}

export function RiskMetricsCard() {
  const { data: portfolioHistory } = useQuery({
    queryKey: ["portfolio_history", 7, "15m"],
    queryFn: () => api.getPortfolioHistory({ rangeDays: 7, interval: "15m" }),
    refetchInterval: 5000
  });

  const { data: sessions } = useQuery({
    queryKey: ["sessions"],
    queryFn: api.getSessions,
    refetchInterval: 5000
  });

  if (!portfolioHistory || portfolioHistory.length === 0) {
    return (
      <Card className="bg-slate-900 border-slate-800 h-full">
        <CardHeader>
          <CardTitle className="text-sm uppercase text-slate-400 font-bold">Risk Profile</CardTitle>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="text-center text-slate-500 py-8">
            No active trading data
          </div>
        </CardContent>
      </Card>
    );
  }

  const maxDD = calculateMaxDrawdown(portfolioHistory);
  const volatility = calculateVolatility(portfolioHistory);
  const var95 = calculateVaR(portfolioHistory, 0.95);

  const activeSessions = sessions?.filter(s => s.status === 'active') || [];
  const totalInitial = activeSessions.reduce((sum, s) => sum + s.initial_capital, 0);
  const totalExposure = activeSessions.reduce((sum, s) => {
    return sum + (s.current_position !== 0 ? s.current_equity : 0);
  }, 0);
  const cashPct = totalInitial > 0 ? ((totalInitial - totalExposure) / totalInitial) * 100 : 0;
  const exposurePct = totalInitial > 0 ? (totalExposure / totalInitial) * 100 : 0;

  return (
    <Card className="bg-slate-900 border-slate-800 h-full">
      <CardHeader>
        <CardTitle className="text-sm uppercase text-slate-400 font-bold">Risk Profile</CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="flex justify-between items-center">
          <div className="flex items-center gap-2 text-slate-300">
            <ShieldAlert className="w-4 h-4 text-orange-500" />
            <span className="text-sm">Value at Risk (95%)</span>
          </div>
          <span className={`font-mono text-sm ${var95 < 0 ? 'text-red-400' : 'text-green-400'}`}>
            ${var95.toFixed(2)}
          </span>
        </div>

        <div className="flex justify-between items-center">
          <div className="flex items-center gap-2 text-slate-300">
            <TrendingDown className="w-4 h-4 text-red-500" />
            <span className="text-sm">Max Drawdown</span>
          </div>
          <span className="font-mono text-red-400 text-sm">
            -{maxDD.toFixed(2)}%
          </span>
        </div>

        <div className="flex justify-between items-center">
          <div className="flex items-center gap-2 text-slate-300">
            <Activity className="w-4 h-4 text-purple-500" />
            <span className="text-sm">Daily Volatility</span>
          </div>
          <span className="font-mono text-white text-sm">
            {volatility.toFixed(2)}%
          </span>
        </div>

        <div className="border-t border-slate-800 pt-4">
          <div className="flex items-center gap-2 text-slate-300 mb-2">
            <Wallet className="w-4 h-4 text-indigo-500" />
            <span className="text-sm">Capital Allocation</span>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between text-xs">
              <span className="text-slate-400">Cash</span>
              <span className="text-green-400 font-mono">{cashPct.toFixed(1)}%</span>
            </div>
            <div className="flex justify-between text-xs">
              <span className="text-slate-400">Exposure</span>
              <span className="text-indigo-400 font-mono">{exposurePct.toFixed(1)}%</span>
            </div>
            <div className="w-full h-2 bg-slate-950 rounded-full overflow-hidden flex">
              <div
                className="bg-green-500 h-full"
                style={{ width: `${cashPct}%` }}
              />
              <div
                className="bg-indigo-500 h-full"
                style={{ width: `${exposurePct}%` }}
              />
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

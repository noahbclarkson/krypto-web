"use client";

import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { Generator } from "@/components/Generator";
import { StrategyList } from "@/components/StrategyList";
import { ActiveSessions } from "@/components/ActiveSessions";
import { PortfolioChart } from "@/components/PortfolioChart";
import { RiskMetricsCard } from "@/components/RiskMetricsCard";
import { Card, CardContent } from "@/components/ui/card";
import { TrendingUp, DollarSign, Activity } from "lucide-react";

export default function Home() {
  const { data: sessions } = useQuery({
    queryKey: ["sessions"],
    queryFn: api.getSessions,
    refetchInterval: 5000
  });

  const { data: portfolioHistory } = useQuery({
    queryKey: ["portfolio_history", 7, "15m", "line"],
    queryFn: () => api.getPortfolioHistory({ rangeDays: 7, interval: "15m", style: "line" }),
    refetchInterval: 5000
  });

  const isTrading = sessions && sessions.length > 0;

  const isLineData = (data: any): data is Array<{ timestamp: string; total_equity: number }> => {
    return data && data.length > 0 && 'total_equity' in data[0];
  };

  const latestEquity = portfolioHistory && isLineData(portfolioHistory)
    ? portfolioHistory[portfolioHistory.length - 1].total_equity
    : 0;
  const startEquity = portfolioHistory && isLineData(portfolioHistory)
    ? portfolioHistory[0].total_equity
    : 0;
  const pnl24h = latestEquity - startEquity;
  const pnlPct24h = startEquity > 0 ? (pnl24h / startEquity) * 100 : 0;

  return (
    <div className="min-h-screen bg-slate-950 p-6">
      <header className="flex justify-between items-center mb-8">
        <div>
          <h1 className="text-2xl font-bold text-white tracking-tight">
            KRYPTO <span className="text-indigo-500">TERMINAL</span>
          </h1>
          <div className="flex items-center gap-2 text-xs text-slate-400 mt-1">
            <span className="w-2 h-2 rounded-full bg-green-500 animate-pulse"></span>
            System Operational
          </div>
        </div>
        {isTrading && (
          <div className="flex gap-4">
            <Card className="bg-slate-900 border-slate-800">
              <CardContent className="p-4">
                <div className="flex items-center gap-2 text-slate-400 text-xs mb-1">
                  <DollarSign className="w-3 h-3" />
                  <span>Total Equity</span>
                </div>
                <div className="text-2xl font-bold text-white font-mono">
                  ${latestEquity.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                </div>
              </CardContent>
            </Card>
            <Card className="bg-slate-900 border-slate-800">
              <CardContent className="p-4">
                <div className="flex items-center gap-2 text-slate-400 text-xs mb-1">
                  <TrendingUp className="w-3 h-3" />
                  <span>24h PnL</span>
                </div>
                <div className={`text-2xl font-bold font-mono ${pnl24h >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                  {pnl24h >= 0 ? '+' : ''}${pnl24h.toFixed(2)} ({pnl24h >= 0 ? '+' : ''}{pnlPct24h.toFixed(2)}%)
                </div>
              </CardContent>
            </Card>
            <Card className="bg-slate-900 border-slate-800">
              <CardContent className="p-4">
                <div className="flex items-center gap-2 text-slate-400 text-xs mb-1">
                  <Activity className="w-3 h-3" />
                  <span>Active Sessions</span>
                </div>
                <div className="text-2xl font-bold text-white font-mono">
                  {sessions?.filter(s => s.status === 'active').length || 0}
                </div>
              </CardContent>
            </Card>
          </div>
        )}
      </header>

      <div className="grid grid-cols-12 gap-6">
        {!isTrading && (
          <div className="col-span-12 lg:col-span-3 space-y-6">
            <Generator />
          </div>
        )}

        <div className={isTrading ? "col-span-12" : "col-span-12 lg:col-span-9"}>
          {isTrading && (
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-6">
              <div className="lg:col-span-2">
                <PortfolioChart />
              </div>
              <div className="lg:col-span-1">
                <RiskMetricsCard />
              </div>
            </div>
          )}

          {isTrading && (
            <div className="mb-6">
              <ActiveSessions />
            </div>
          )}

          <h2 className="text-2xl font-bold text-white mb-4">Strategy Pool</h2>
          <StrategyList />
        </div>
      </div>
    </div>
  );
}

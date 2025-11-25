"use client";
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { Area, AreaChart, ResponsiveContainer, XAxis, YAxis, Tooltip } from "recharts";
import { format } from "date-fns";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { TrendingUp } from "lucide-react";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";

export function PortfolioChart() {
  const [rangeDays, setRangeDays] = useState(7);
  const [interval, setInterval] = useState("15m");

  const { data } = useQuery({
    queryKey: ["portfolio_history", rangeDays, interval],
    queryFn: () => api.getPortfolioHistory({ rangeDays, interval }),
    refetchInterval: 5000
  });

  if (!data || data.length === 0) {
    return (
      <Card className="bg-slate-900 border-slate-800 mb-8 p-8 text-center text-slate-500">
        Waiting for trading data...
      </Card>
    );
  }

  const latest = data[data.length - 1].total_equity;
  const start = data[0].total_equity;
  const pnl = latest - start;

  return (
    <Card className="bg-slate-900 border-slate-800 h-full">
      <CardHeader className="pb-3">
        <div className="flex justify-between items-center">
          <CardTitle className="flex items-center gap-2 text-slate-100">
            <TrendingUp className="h-5 w-5 text-indigo-500" />
            Portfolio Equity Curve
          </CardTitle>
          <div className="flex gap-2">
            <Select value={String(rangeDays)} onValueChange={(v) => setRangeDays(Number(v))}>
              <SelectTrigger className="h-8 bg-slate-950 border-slate-800 text-xs w-24">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="1">24h</SelectItem>
                <SelectItem value="7">7d</SelectItem>
                <SelectItem value="30">30d</SelectItem>
                <SelectItem value="90">90d</SelectItem>
              </SelectContent>
            </Select>
            <Select value={interval} onValueChange={(v) => setInterval(v)}>
              <SelectTrigger className="h-8 bg-slate-950 border-slate-800 text-xs w-20">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="3m">3m</SelectItem>
                <SelectItem value="15m">15m</SelectItem>
                <SelectItem value="1h">1h</SelectItem>
                <SelectItem value="1d">1d</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
      </CardHeader>
      <CardContent className="h-[340px]">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={data}>
            <defs>
              <linearGradient id="colorEquity" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="#6366f1" stopOpacity={0.3}/>
                <stop offset="95%" stopColor="#6366f1" stopOpacity={0}/>
              </linearGradient>
            </defs>
            <XAxis
                dataKey="timestamp"
                tickFormatter={(str) => format(new Date(str), "MMM dd")}
                stroke="#475569"
                fontSize={12}
            />
            <YAxis
                domain={['auto', 'auto']}
                stroke="#475569"
                fontSize={12}
                tickFormatter={(val) => `$${val.toLocaleString('en-US', { minimumFractionDigits: 0, maximumFractionDigits: 0 })}`}
            />
            <Tooltip
                contentStyle={{ backgroundColor: '#1e293b', borderColor: '#334155' }}
                labelFormatter={(v) => format(new Date(v), "MMM dd HH:mm")}
                formatter={(value: number) => [`$${value.toFixed(2)}`, "Total Equity"]}
            />
            <Area
                type="monotone"
                dataKey="total_equity"
                stroke="#6366f1"
                fillOpacity={1}
                fill="url(#colorEquity)"
            />
          </AreaChart>
        </ResponsiveContainer>
      </CardContent>
    </Card>
  );
}

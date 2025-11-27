"use client";
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { Area, AreaChart, ResponsiveContainer, XAxis, YAxis, Tooltip } from "recharts";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { TrendingUp, BarChart2, LineChart } from "lucide-react";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { CandleChart } from "./CandleChart";
import { PortfolioPoint, Candle } from "@/lib/types";

export function PortfolioChart() {
  const [rangeDays, setRangeDays] = useState(7);
  const [interval, setInterval] = useState("15m");
  const [chartType, setChartType] = useState<"line" | "candle">("line");

  const { data } = useQuery({
    queryKey: ["portfolio_history", rangeDays, interval, chartType],
    queryFn: () => api.getPortfolioHistory({ rangeDays, interval, style: chartType }),
    refetchInterval: 5000
  });

  if (!data || data.length === 0) {
    return (
      <Card className="bg-slate-900 border-slate-800 mb-8 p-8 text-center text-slate-500">
        Waiting for trading data...
      </Card>
    );
  }

  const isLineData = (d: any[]): d is PortfolioPoint[] => chartType === "line" && "total_equity" in d[0];
  const isCandleData = (d: any[]): d is Candle[] => chartType === "candle" && "open" in d[0];

  return (
    <Card className="bg-slate-900 border-slate-800 h-full flex flex-col">
      <CardHeader className="pb-3">
        <div className="flex justify-between items-center flex-wrap gap-4">
          <CardTitle className="flex items-center gap-2 text-slate-100">
            <TrendingUp className="h-5 w-5 text-indigo-500" />
            Portfolio Equity Curve
          </CardTitle>

          <div className="flex gap-2 items-center">
            <div className="flex bg-slate-950 rounded-md border border-slate-800 p-1 gap-1">
              <Button
                variant={chartType === "line" ? "secondary" : "ghost"}
                size="icon-sm"
                className="h-7 w-7"
                onClick={() => setChartType("line")}
              >
                <LineChart className="w-4 h-4" />
              </Button>
              <Button
                variant={chartType === "candle" ? "secondary" : "ghost"}
                size="icon-sm"
                className="h-7 w-7"
                onClick={() => setChartType("candle")}
              >
                <BarChart2 className="w-4 h-4" />
              </Button>
            </div>

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
                <SelectItem value="1m">1m</SelectItem>
                <SelectItem value="5m">5m</SelectItem>
                <SelectItem value="15m">15m</SelectItem>
                <SelectItem value="1h">1h</SelectItem>
                <SelectItem value="4h">4h</SelectItem>
                <SelectItem value="1d">1d</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
      </CardHeader>

      <CardContent className="h-[340px] w-full min-h-0">
        {chartType === "line" && isLineData(data) && (
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
                tickFormatter={(str) => {
                  const date = new Date(str);
                  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
                }}
                stroke="#475569"
                fontSize={12}
                minTickGap={30}
              />
              <YAxis
                domain={['auto', 'auto']}
                stroke="#475569"
                fontSize={12}
                tickFormatter={(val) => `${val.toLocaleString('en-US', { notation: "compact", compactDisplay: "short" })}`}
                width={60}
              />
              <Tooltip
                contentStyle={{ backgroundColor: '#1e293b', borderColor: '#334155' }}
                labelFormatter={(v) => new Date(v).toLocaleString()}
                formatter={(value: number) => [`${value.toFixed(2)}`, "Total Equity"]}
              />
              <Area
                type="monotone"
                dataKey="total_equity"
                stroke="#6366f1"
                fillOpacity={1}
                fill="url(#colorEquity)"
                isAnimationActive={false}
              />
            </AreaChart>
          </ResponsiveContainer>
        )}

        {chartType === "candle" && isCandleData(data) && (
          <CandleChart
            data={data}
            colors={{
              backgroundColor: "transparent",
              textColor: "#94a3b8"
            }}
          />
        )}
      </CardContent>
    </Card>
  );
}

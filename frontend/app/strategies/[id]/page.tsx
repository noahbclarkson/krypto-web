"use client";

import { useQuery } from "@tanstack/react-query";
import { useParams, useRouter } from "next/navigation";
import { api } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ArrowLeft, TrendingUp, Activity, Clock } from "lucide-react";
import { AreaChart, Area, XAxis, YAxis, Tooltip, ResponsiveContainer } from "recharts";
import { format } from "date-fns";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";

export default function StrategyDetailPage() {
  const params = useParams();
  const router = useRouter();
  const id = params.id as string;

  const { data: strategies } = useQuery({ queryKey: ["strategies"], queryFn: api.getStrategies });
  const strategy = strategies?.find(s => s.id === id);

  const { data: sessions } = useQuery({ queryKey: ["sessions"], queryFn: api.getSessions });
  const session = sessions?.find(s => s.strategy_id === id);

  const { data: trades } = useQuery({
    queryKey: ["trades", session?.id],
    queryFn: () => session ? api.getSessionTrades(session.id) : Promise.resolve([]),
    enabled: !!session,
    refetchInterval: 2000
  });

  if (!strategy) return <div className="p-8 text-slate-400">Loading strategy...</div>;

  return (
    <div className="max-w-7xl mx-auto">
      <Button variant="ghost" onClick={() => router.back()} className="mb-4 text-slate-400 hover:text-white">
        <ArrowLeft className="mr-2 h-4 w-4" /> Back to Dashboard
      </Button>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <Card className="lg:col-span-3 bg-slate-900 border-slate-800">
          <CardHeader>
            <div className="flex justify-between items-start">
              <div>
                <CardTitle className="text-3xl text-white flex items-center gap-3">
                  {strategy.symbol}
                  <span className="text-indigo-400 text-lg font-mono px-2 py-1 bg-indigo-950/50 rounded border border-indigo-900">
                    {strategy.strategy_type}
                  </span>
                </CardTitle>
                <div className="text-slate-400 mt-2 flex gap-4">
                  <span className="flex items-center gap-1"><Clock className="w-4 h-4" /> {strategy.interval}</span>
                  <span className="flex items-center gap-1"><TrendingUp className="w-4 h-4" /> Sharpe: {strategy.performance_metrics.sharpe.toFixed(2)}</span>
                </div>
              </div>
              {session && (
                 <Badge variant="outline" className="text-green-400 border-green-900 bg-green-950/30 text-lg py-1 px-3">
                    Active Trading
                 </Badge>
              )}
            </div>
          </CardHeader>
        </Card>

        <Card className="lg:col-span-2 bg-slate-900 border-slate-800">
            <CardHeader><CardTitle className="text-slate-200">Backtest Performance</CardTitle></CardHeader>
            <CardContent className="h-[400px]">
                <ResponsiveContainer width="100%" height="100%">
                    <AreaChart data={strategy.backtest_curve.map((v, i) => ({ i, v }))}>
                        <defs>
                            <linearGradient id="colorB" x1="0" y1="0" x2="0" y2="1">
                                <stop offset="5%" stopColor="#818cf8" stopOpacity={0.3}/>
                                <stop offset="95%" stopColor="#818cf8" stopOpacity={0}/>
                            </linearGradient>
                        </defs>
                        <Tooltip contentStyle={{backgroundColor: '#0f172a', borderColor: '#334155'}} />
                        <Area type="monotone" dataKey="v" stroke="#818cf8" fill="url(#colorB)" />
                    </AreaChart>
                </ResponsiveContainer>
            </CardContent>
        </Card>

        <Card className="bg-slate-900 border-slate-800">
            <CardHeader><CardTitle className="text-slate-200">Configuration</CardTitle></CardHeader>
            <CardContent>
                <div className="space-y-4">
                    {Object.entries(strategy.parameters).map(([key, val]) => (
                        <div key={key} className="flex justify-between border-b border-slate-800 pb-2">
                            <span className="text-slate-400 font-mono text-sm">{key}</span>
                            <span className="text-white font-mono">{String(val)}</span>
                        </div>
                    ))}
                    <div className="pt-4">
                        <div className="text-xs text-slate-500 uppercase font-bold mb-2">Metrics</div>
                        <div className="grid grid-cols-2 gap-4">
                            <div>
                                <div className="text-slate-500 text-xs">Win Rate</div>
                                <div className="text-white font-mono">{strategy.performance_metrics.win_rate.toFixed(1)}%</div>
                            </div>
                            <div>
                                <div className="text-slate-500 text-xs">Max Drawdown</div>
                                <div className="text-red-400 font-mono">{strategy.performance_metrics.max_drawdown_pct.toFixed(1)}%</div>
                            </div>
                        </div>
                    </div>
                </div>
            </CardContent>
        </Card>

        {trades && trades.length > 0 && (
            <Card className="lg:col-span-3 bg-slate-900 border-slate-800">
                <CardHeader><CardTitle className="text-slate-200">Live Trade History</CardTitle></CardHeader>
                <CardContent>
                    <Table>
                        <TableHeader>
                            <TableRow className="border-slate-800">
                                <TableHead>Time</TableHead>
                                <TableHead>Side</TableHead>
                                <TableHead>Price</TableHead>
                                <TableHead>PnL</TableHead>
                                <TableHead>Reason</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {trades.map(t => (
                                <TableRow key={t.id} className="border-slate-800">
                                    <TableCell className="font-mono text-slate-400">{format(new Date(t.timestamp), "yyyy-MM-dd HH:mm:ss")}</TableCell>
                                    <TableCell><Badge variant={t.side === 'BUY' ? 'default' : 'destructive'}>{t.side}</Badge></TableCell>
                                    <TableCell className="font-mono">${t.price.toFixed(4)}</TableCell>
                                    <TableCell className={`font-mono ${t.pnl && t.pnl > 0 ? 'text-green-400' : 'text-red-400'}`}>
                                        {t.pnl ? `$${t.pnl.toFixed(2)}` : '-'}
                                    </TableCell>
                                    <TableCell className="text-slate-500 text-xs">{t.reason}</TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                </CardContent>
            </Card>
        )}
      </div>
    </div>
  );
}

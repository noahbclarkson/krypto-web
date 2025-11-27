"use client";

import { useEffect, useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useRouter } from "next/navigation";
import { api } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Rocket, Trash2, Eye } from "lucide-react";
import { Area, AreaChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { toast } from "sonner";

export function StrategyList() {
  const router = useRouter();
  const { data: strategies } = useQuery({ queryKey: ["strategies"], queryFn: api.getStrategies });
  const { data: sessions } = useQuery({ queryKey: ["sessions"], queryFn: api.getSessions, refetchInterval: 5000 });
  const queryClient = useQueryClient();

  const activeSessions = (sessions || []).filter((s) => s.status === "active");
  const isTrading = activeSessions.length > 0;
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [totalCapital, setTotalCapital] = useState(10000);
  const [resetOpen, setResetOpen] = useState(false);

  useEffect(() => {
    if (strategies && !isTrading) {
      setSelectedIds(new Set(strategies.map((s) => s.id)));
    }
  }, [strategies, isTrading]);

  const toggleSelection = (id: string) => {
    const newSet = new Set(selectedIds);
    if (newSet.has(id)) newSet.delete(id);
    else newSet.add(id);
    setSelectedIds(newSet);
  };

  const deployPortfolio = useMutation({
    mutationFn: async () => {
      const selectedStrats = strategies?.filter(s => selectedIds.has(s.id)) || [];

      const totalKelly = selectedStrats.reduce((sum, s) => sum + (s.kelly_fraction || 0.1), 0);
      const leverageRatio = totalKelly > 1.0 ? 1.0 / totalKelly : 1.0;

      const promises = selectedStrats.map(s => {
        const allocation = totalCapital * (s.kelly_fraction || 0.1) * leverageRatio;
        return api.startSession({
          strategy_id: s.id,
          initial_capital: allocation
        });
      });

      await Promise.all(promises);
    },
    onSuccess: () => {
      toast.success("Portfolio Deployed", { description: `Deployed ${selectedIds.size} strategies.` });
      setSelectedIds(new Set());
      queryClient.invalidateQueries({ queryKey: ["sessions"] });
    }
  });

  const resetSessions = useMutation({
    mutationFn: api.resetSessions,
    onSuccess: () => {
      toast.warning("Trading reset");
      setSelectedIds(new Set());
      queryClient.invalidateQueries({ queryKey: ["sessions"] });
      queryClient.invalidateQueries({ queryKey: ["portfolio_history"] });
      setResetOpen(false);
    }
  });

  const deleteOne = useMutation({
    mutationFn: api.deleteStrategy,
    onSuccess: () => {
      toast.info("Strategy Deleted");
      queryClient.invalidateQueries({ queryKey: ["strategies"] });
    }
  });

  const deleteAll = useMutation({
    mutationFn: api.deleteAllStrategies,
    onSuccess: () => {
      toast.warning("All Strategies Deleted");
      queryClient.invalidateQueries({ queryKey: ["strategies"] });
    }
  });

  if (!strategies) return <div className="text-slate-400">Loading strategies...</div>;

  if (strategies.length === 0) {
    return (
      <div className="text-center py-12 border border-dashed border-slate-800 rounded-xl mt-8">
        <p className="text-slate-400">No strategies generated yet. Run the optimizer to start.</p>
      </div>
    );
  }

  const selectedStrats = strategies?.filter(s => selectedIds.has(s.id)) || [];
  const totalKelly = selectedStrats.reduce((sum, s) => sum + (s.kelly_fraction || 0.1), 0);

  return (
    <div className="mt-8 pb-24">
      <div className="flex justify-between items-center mb-4">
        <h3 className="text-xl font-bold text-slate-200">Optimized Strategies ({strategies.length})</h3>
        {!isTrading ? (
          <div className="flex gap-2">
              <Button
                  variant="destructive"
                  size="sm"
                  onClick={() => deleteAll.mutate()}
                  disabled={deleteAll.isPending}
                  className="bg-red-950 hover:bg-red-900 text-red-200 border border-red-900"
              >
                  <Trash2 className="w-4 h-4 mr-2" /> Clear All
              </Button>
          </div>
        ) : (
          <Button
            variant="outline"
            size="sm"
            onClick={() => setResetOpen(true)}
            className="border-slate-700 text-slate-200"
          >
            Reset Trading
          </Button>
        )}
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {strategies.map((strat) => {
          const isSelected = !isTrading && selectedIds.has(strat.id);
          return (
            <Card
                key={strat.id}
                className={`border transition-all ${isSelected ? 'border-indigo-500 bg-indigo-950/10' : 'border-slate-800 bg-slate-900'} overflow-hidden ${!isTrading ? 'cursor-pointer hover:border-indigo-500/30' : ''} relative group`}
                onClick={!isTrading ? () => toggleSelection(strat.id) : undefined}
            >
            {!isTrading && (
              <Button
                  variant="ghost"
                  size="icon"
                  className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity text-slate-500 hover:text-red-400 z-10"
                  onClick={(e) => {
                      e.stopPropagation();
                      deleteOne.mutate(strat.id);
                  }}
              >
                  <Trash2 className="w-4 h-4" />
              </Button>
            )}

            <CardHeader className="pb-2">
                <div className="flex justify-between items-start pr-8">
                <div className="flex items-start gap-2">
                    {!isTrading && (
                      <Checkbox
                          checked={selectedIds.has(strat.id)}
                          onClick={(e) => e.stopPropagation()}
                          className="mt-1"
                      />
                    )}
                    <div>
                        <CardTitle className="text-lg font-bold text-white">{strat.symbol}</CardTitle>
                        <div className="text-xs text-slate-400 font-mono mt-1">{strat.strategy_type}</div>
                        <div className="flex gap-1 mt-1">
                            <Badge variant="outline" className="text-[10px]">{strat.interval}</Badge>
                            <Badge variant="secondary" className="bg-emerald-950 text-emerald-400 border-emerald-900 text-[10px]">
                                Kelly: {((strat.kelly_fraction || 0.1) * 100).toFixed(1)}%
                            </Badge>
                        </div>
                    </div>
                </div>
                <div className="flex flex-col gap-1 items-end">
                    <Badge variant={strat.performance_metrics.sharpe > 2 ? "default" : "secondary"}>
                        SR: {strat.performance_metrics.sharpe.toFixed(2)}
                    </Badge>
                    <span className="text-[10px] text-slate-500 font-mono">
                        WR: {strat.performance_metrics.win_rate.toFixed(1)}%
                    </span>
                </div>
                </div>
            </CardHeader>
            <CardContent>
                <div className="h-32 w-full mb-4 bg-slate-950/30 rounded-lg pt-2 pr-2">
                <ResponsiveContainer width="100%" height="100%">
                    <AreaChart data={strat.backtest_curve.map((v, i) => ({ i, v }))}>
                        <defs>
                            <linearGradient id={`grad${strat.id}`} x1="0" y1="0" x2="0" y2="1">
                                <stop offset="5%" stopColor="#10b981" stopOpacity={0.3}/>
                                <stop offset="95%" stopColor="#10b981" stopOpacity={0}/>
                            </linearGradient>
                        </defs>
                        <Tooltip
                            contentStyle={{ backgroundColor: '#0f172a', borderColor: '#334155', fontSize: '12px' }}
                            itemStyle={{ color: '#10b981' }}
                            labelStyle={{ display: 'none' }}
                            formatter={(value: number) => [`${value.toFixed(2)}`, 'Equity']}
                        />
                        <YAxis
                            hide={false}
                            domain={['auto', 'auto']}
                            width={40}
                            tick={{fontSize: 10, fill: '#475569'}}
                            axisLine={false}
                            tickLine={false}
                            tickFormatter={(val) => `${(val/1000).toFixed(1)}k`}
                        />
                        <XAxis hide />
                        <Area
                            type="monotone"
                            dataKey="v"
                            stroke="#10b981"
                            fill={`url(#grad${strat.id})`}
                            strokeWidth={2}
                        />
                    </AreaChart>
                </ResponsiveContainer>
                </div>
                <div className="grid grid-cols-3 gap-2 text-xs mb-4 border-t border-slate-800 pt-3">
                <div>
                    <div className="text-slate-500">Net PnL</div>
                    <div className="text-green-400 font-mono font-bold">
                        {strat.performance_metrics.total_return_pct.toFixed(1)}%
                    </div>
                </div>
                <div>
                    <div className="text-slate-500">Max DD</div>
                    <div className="text-red-400 font-mono">
                        {strat.performance_metrics.max_drawdown_pct.toFixed(1)}%
                    </div>
                </div>
                <div>
                    <div className="text-slate-500">Trades</div>
                    <div className="text-slate-300 font-mono">
                        {strat.performance_metrics.trades}
                    </div>
                </div>
                </div>
                <Button
                    variant="outline"
                    size="sm"
                    className="w-full border-slate-700 text-slate-300 hover:text-white hover:border-indigo-500"
                    onClick={(e) => {
                        e.stopPropagation();
                        router.push(`/strategies/${strat.id}`);
                    }}
                >
                    <Eye className="w-4 h-4 mr-2" />
                    View Details
                </Button>
            </CardContent>
            </Card>
          );
        })}
      </div>

      {!isTrading && selectedIds.size > 0 && (
        <div className="fixed bottom-0 left-0 right-0 bg-slate-900/90 backdrop-blur border-t border-slate-800 p-4 z-50">
            <div className="max-w-7xl mx-auto flex items-center justify-between">
                <div className="flex items-center gap-6">
                    <div>
                        <div className="text-xs text-slate-400 uppercase font-bold">Selected</div>
                        <div className="text-2xl font-bold">{selectedIds.size}</div>
                    </div>
                    <div>
                        <div className="text-xs text-slate-400 uppercase font-bold">Effective Leverage</div>
                        <div className={`text-2xl font-bold ${totalKelly > 1.0 ? 'text-orange-400' : 'text-blue-400'}`}>
                            {totalKelly.toFixed(2)}x
                        </div>
                    </div>
                    <div className="w-48">
                        <div className="text-xs text-slate-400 uppercase font-bold mb-1">Total Capital ($)</div>
                        <Input
                            type="number"
                            value={totalCapital}
                            onChange={(e) => setTotalCapital(Number(e.target.value))}
                            className="bg-slate-950 border-slate-700"
                            onClick={(e) => e.stopPropagation()}
                        />
                    </div>
                </div>
                <Button
                    size="lg"
                    className="bg-indigo-600 hover:bg-indigo-700 text-white"
                    onClick={() => deployPortfolio.mutate()}
                    disabled={deployPortfolio.isPending || isTrading}
                >
                    <Rocket className="mr-2 h-5 w-5" />
                    Start Trading
                </Button>
            </div>
        </div>
      )}

      <Dialog open={resetOpen} onOpenChange={setResetOpen}>
        <DialogContent className="bg-slate-950 border-slate-800 text-slate-50">
          <DialogHeader>
            <DialogTitle>Reset Trading?</DialogTitle>
            <DialogDescription className="text-slate-400">
              This will stop all sessions and clear their history. Are you sure you want to reset?
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setResetOpen(false)}>Cancel</Button>
            <Button
              variant="destructive"
              onClick={() => resetSessions.mutate()}
              disabled={resetSessions.isPending}
            >
              Reset
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

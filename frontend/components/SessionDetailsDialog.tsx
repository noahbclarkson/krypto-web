"use client";

import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "@/components/ui/dialog";
import { Session } from "@/lib/types";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Area, AreaChart, ResponsiveContainer, XAxis, YAxis, Tooltip } from "recharts";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { format } from "date-fns";

interface Props {
  session: Session | null;
  isOpen: boolean;
  onClose: () => void;
}

export function SessionDetailsDialog({ session, isOpen, onClose }: Props) {
  const { data: trades } = useQuery({
    queryKey: ["trades", session?.id],
    queryFn: () => api.getSessionTrades(session!.id),
    enabled: !!session,
    refetchInterval: 1000
  });

  const { data: equity } = useQuery({
    queryKey: ["equity", session?.id],
    queryFn: () => api.getEquityCurve(session!.id),
    enabled: !!session,
    refetchInterval: 1000
  });

  if (!session) return null;

  const pnl = session.current_equity - session.initial_capital;
  const pnlPct = (pnl / session.initial_capital) * 100;

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="max-w-4xl bg-slate-950 border-slate-800 text-slate-50 max-h-[90vh] overflow-hidden flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-4 text-xl">
            <span>{session.symbol}</span>
            <Badge variant="outline">{session.interval}</Badge>
            <Badge variant={pnl >= 0 ? "default" : "destructive"}>
              {pnl >= 0 ? "+" : ""}{pnlPct.toFixed(2)}%
            </Badge>
          </DialogTitle>
          <DialogDescription className="text-slate-400">
            Paper Trading Session ID: <span className="font-mono text-xs">{session.id}</span>
          </DialogDescription>
        </DialogHeader>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-4">
          <div className="p-4 bg-slate-900 rounded-lg border border-slate-800">
            <div className="text-slate-400 text-xs">Current Equity</div>
            <div className="text-2xl font-bold font-mono text-green-400">
              ${session.current_equity.toFixed(4)}
            </div>
          </div>
          <div className="p-4 bg-slate-900 rounded-lg border border-slate-800">
            <div className="text-slate-400 text-xs">Position</div>
            <div className="text-2xl font-bold font-mono">
              {session.current_position > 0 ? "LONG" : session.current_position < 0 ? "SHORT" : "FLAT"}
            </div>
          </div>
          <div className="p-4 bg-slate-900 rounded-lg border border-slate-800">
            <div className="text-slate-400 text-xs">Total Trades</div>
            <div className="text-2xl font-bold font-mono">{trades?.length || 0}</div>
          </div>
        </div>

        <div className="h-48 w-full mt-4 bg-slate-900/50 rounded-lg border border-slate-800 p-2">
          {equity && equity.length > 0 ? (
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={equity}>
                <XAxis
                    dataKey="timestamp"
                    hide
                />
                <YAxis
                    domain={['auto', 'auto']}
                    hide
                />
                <Tooltip
                    contentStyle={{ backgroundColor: '#1e293b', borderColor: '#334155' }}
                    labelFormatter={(v) => format(new Date(v), "HH:mm:ss")}
                />
                <Area
                    type="monotone"
                    dataKey="equity"
                    stroke="#6366f1"
                    fill="#6366f1"
                    fillOpacity={0.1}
                />
              </AreaChart>
            </ResponsiveContainer>
          ) : (
            <div className="h-full flex items-center justify-center text-slate-500 text-sm">
              Not enough data for chart
            </div>
          )}
        </div>

        <div className="mt-4 flex-1 overflow-hidden flex flex-col">
          <h3 className="text-sm font-medium mb-2 text-slate-300">Trade History</h3>
          <div className="bg-slate-900 border-slate-800 rounded-md flex-1 overflow-hidden">
            <ScrollArea className="h-[200px]">
              <Table>
                <TableHeader className="bg-slate-900 sticky top-0">
                  <TableRow className="border-slate-800 hover:bg-slate-900">
                    <TableHead className="text-slate-400">Time</TableHead>
                    <TableHead className="text-slate-400">Side</TableHead>
                    <TableHead className="text-slate-400 text-right">Price</TableHead>
                    <TableHead className="text-slate-400 text-right">PnL</TableHead>
                    <TableHead className="text-slate-400">Reason</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {trades?.map((t) => (
                    <TableRow key={t.id} className="border-slate-800 hover:bg-slate-800/50">
                      <TableCell className="font-mono text-xs text-slate-400">
                        {format(new Date(t.timestamp), "MMM dd HH:mm:ss")}
                      </TableCell>
                      <TableCell>
                        <Badge variant={t.side === "BUY" ? "default" : "destructive"} className="text-[10px]">
                          {t.side}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-right font-mono text-slate-300">
                        ${t.price.toFixed(4)}
                      </TableCell>
                      <TableCell className={`text-right font-mono ${(t.pnl || 0) > 0 ? "text-green-400" : (t.pnl || 0) < 0 ? "text-red-400" : "text-slate-500"}`}>
                        {t.pnl ? `${t.pnl.toFixed(4)}` : "-"}
                      </TableCell>
                      <TableCell className="text-xs text-slate-500 max-w-[200px] truncate" title={t.reason || ""}>
                        {t.reason || "-"}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </ScrollArea>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

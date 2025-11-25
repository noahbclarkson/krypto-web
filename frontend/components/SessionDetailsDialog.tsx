"use client";

import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "@/components/ui/dialog";
import { Session } from "@/lib/types";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { format } from "date-fns";
import { CandleChart } from "./CandleChart";

interface Props {
  session: Session | null;
  isOpen: boolean;
  onClose: () => void;
}

export function SessionDetailsDialog({ session, isOpen, onClose }: Props) {
  const { data: sessions } = useQuery({
    queryKey: ["sessions"],
    queryFn: api.getSessions,
    enabled: !!session,
    refetchInterval: 1000,
  });

  const liveSession = sessions?.find((s) => s.id === session?.id) ?? session;

  const { data: trades } = useQuery({
    queryKey: ["trades", liveSession?.id],
    queryFn: () => api.getSessionTrades(liveSession!.id),
    enabled: !!liveSession,
    refetchInterval: 1000
  });

  const { data: candles } = useQuery({
    queryKey: ["candles", liveSession?.id],
    queryFn: () => api.getSessionCandles(liveSession!.id),
    enabled: !!liveSession,
    refetchInterval: 5000
  });

  if (!liveSession) return null;

  const pnl = liveSession.current_equity - liveSession.initial_capital;
  const pnlPct = (pnl / liveSession.initial_capital) * 100;
  const realizedPnl = (trades || []).reduce((acc, t) => acc + (t.pnl || 0), 0);
  const unrealizedPnl = pnl - realizedPnl;

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="w-full max-w-[90vw] lg:max-w-[1400px] bg-slate-950 border-slate-800 text-slate-50 max-h-[90vh] overflow-hidden flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-xl flex-wrap">
            <span>{liveSession.symbol}</span>
            <Badge variant="outline">{liveSession.interval}</Badge>
            <Badge variant={pnl >= 0 ? "default" : "destructive"}>
              {pnl >= 0 ? "+" : ""}{pnlPct.toFixed(2)}%
            </Badge>
            <Badge variant={realizedPnl >= 0 ? "default" : "destructive"}>
              Realized {realizedPnl >= 0 ? "+" : ""}{realizedPnl.toFixed(2)}
            </Badge>
            <Badge variant={unrealizedPnl >= 0 ? "secondary" : "destructive"}>
              Unrealized {unrealizedPnl >= 0 ? "+" : ""}{unrealizedPnl.toFixed(2)}
            </Badge>
          </DialogTitle>
          <DialogDescription className="text-slate-400">
            Paper Trading Session ID: <span className="font-mono text-xs">{liveSession.id}</span>
          </DialogDescription>
        </DialogHeader>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-4">
          <div className="p-4 bg-slate-900 rounded-lg border border-slate-800">
            <div className="text-slate-400 text-xs">Current Equity</div>
            <div className="text-2xl font-bold font-mono text-green-400">
              ${liveSession.current_equity.toFixed(4)}
            </div>
          </div>
          <div className="p-4 bg-slate-900 rounded-lg border border-slate-800">
            <div className="text-slate-400 text-xs">Position</div>
            <div className="text-2xl font-bold font-mono">
              {liveSession.current_position > 0 ? "LONG" : liveSession.current_position < 0 ? "SHORT" : "FLAT"}
            </div>
          </div>
          <div className="p-4 bg-slate-900 rounded-lg border border-slate-800">
            <div className="text-slate-400 text-xs">Total Trades</div>
            <div className="text-2xl font-bold font-mono">{trades?.length || 0}</div>
          </div>
        </div>

        <div className="h-72 w-full mt-4 bg-slate-900/50 rounded-lg border border-slate-800 overflow-hidden">
          <div className="h-full w-full p-3">
            {candles && candles.length > 0 ? (
              <CandleChart
                data={candles}
                colors={{
                  backgroundColor: "#0f172a",
                  textColor: "#94a3b8",
                }}
              />
            ) : (
              <div className="h-full flex items-center justify-center text-slate-500 text-sm">
                Waiting for live market data...
              </div>
            )}
          </div>
        </div>

        <div className="mt-4 flex-1 overflow-hidden flex flex-col">
          <h3 className="text-sm font-medium mb-2 text-slate-300">Trade History</h3>
          <div className="bg-slate-900 border-slate-800 rounded-md flex-1 overflow-hidden">
            <ScrollArea className="h-[240px]">
              <Table>
                <TableHeader className="bg-slate-900 sticky top-0">
                  <TableRow className="border-slate-800 hover:bg-slate-900">
                    <TableHead className="text-slate-400">Time</TableHead>
                    <TableHead className="text-slate-400">Side</TableHead>
                    <TableHead className="text-slate-400 text-right">Price</TableHead>
                    <TableHead className="text-slate-400 text-right">Realized / Fees</TableHead>
                    <TableHead className="text-slate-400 w-[60%]">Reason</TableHead>
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
                      <TableCell className="text-xs text-slate-500 whitespace-normal break-words" title={t.reason || ""}>
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

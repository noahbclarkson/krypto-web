"use client";

import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Activity, Eye } from "lucide-react";
import { useState } from "react";
import { Session } from "@/lib/types";
import { SessionDetailsDialog } from "./SessionDetailsDialog";

export function ActiveSessions() {
  const { data: sessions } = useQuery({
    queryKey: ["sessions"],
    queryFn: api.getSessions,
    refetchInterval: 1000
  });

  const [selectedSession, setSelectedSession] = useState<Session | null>(null);

  if (!sessions || sessions.length === 0) return null;

  return (
    <>
      <Card className="bg-slate-900 border-slate-800 mt-8">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-slate-100">
            <Activity className="h-5 w-5 text-green-500" />
            Live Paper Trading Sessions
          </CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow className="border-slate-800 hover:bg-slate-900">
                <TableHead className="text-slate-400">Symbol</TableHead>
                <TableHead className="text-slate-400">Interval</TableHead>
                <TableHead className="text-slate-400">Equity</TableHead>
                <TableHead className="text-slate-400">PnL</TableHead>
                <TableHead className="text-slate-400">Position</TableHead>
                <TableHead className="text-slate-400 text-right">Action</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {sessions.map((s) => {
                const pnl = s.current_equity - s.initial_capital;
                return (
                  <TableRow key={s.id} className="border-slate-800 hover:bg-slate-800/50">
                    <TableCell className="font-medium text-white">{s.symbol}</TableCell>
                    <TableCell className="text-slate-400">{s.interval}</TableCell>
                    <TableCell className="font-mono text-slate-200">
                      ${s.current_equity.toFixed(4)}
                    </TableCell>
                    <TableCell className={`font-mono ${pnl >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                      {pnl >= 0 ? "+" : ""}{pnl.toFixed(4)}
                    </TableCell>
                    <TableCell>
                      <Badge variant={s.current_position > 0 ? "default" : s.current_position < 0 ? "destructive" : "secondary"}>
                        {s.current_position > 0 ? "LONG" : s.current_position < 0 ? "SHORT" : "FLAT"}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-right">
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => setSelectedSession(s)}
                        className="hover:bg-slate-800"
                      >
                        <Eye className="h-4 w-4" />
                      </Button>
                    </TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <SessionDetailsDialog
        session={selectedSession}
        isOpen={!!selectedSession}
        onClose={() => setSelectedSession(null)}
      />
    </>
  );
}

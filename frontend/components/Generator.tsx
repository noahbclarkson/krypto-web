"use client";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Slider } from "@/components/ui/slider";
import { toast } from "sonner";
import { api } from "@/lib/api";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Loader2, Zap, Settings2 } from "lucide-react";
import { useState } from "react";

const AVAILABLE_INTERVALS = ["15m", "30m", "1h", "4h", "12h", "1d"];

export function Generator() {
  const queryClient = useQueryClient();

  const [symbolsInput, setSymbolsInput] = useState("BTCUSDT, ETHUSDT, SOLUSDT, DOGEUSDT, XRPUSDT, ADAUSDT, BNBUSDT, MATICUSDT, LINKUSDT, DOTUSDT");
  const [selectedIntervals, setSelectedIntervals] = useState<string[]>(["1h", "4h"]);
  const [historyLimit, setHistoryLimit] = useState([1000]);
  const [iterations, setIterations] = useState([50]);
  const [topN, setTopN] = useState(10);

  const mutation = useMutation({
    mutationFn: () => {
      const symbols = symbolsInput
        .split(",")
        .map(s => s.trim().toUpperCase())
        .filter(s => s.length > 0);

      if (symbols.length === 0) throw new Error("Please enter at least one symbol");
      if (selectedIntervals.length === 0) throw new Error("Please select at least one interval");

      return api.generateStrategies(
        symbols,
        selectedIntervals,
        topN,
        historyLimit[0],
        iterations[0]
      );
    },
    onSuccess: (data) => {
      toast.success("Optimization Complete", {
        description: `Generated ${data.strategies_created} strategies.`,
      });
      queryClient.invalidateQueries({ queryKey: ["strategies"] });
    },
    onError: (error: any) => {
      toast.error("Optimization Failed", {
        description: error.message,
      });
    }
  });

  const toggleInterval = (interval: string) => {
    setSelectedIntervals(prev =>
      prev.includes(interval)
        ? prev.filter(i => i !== interval)
        : [...prev, interval]
    );
  };

  return (
    <Card className="bg-slate-900 border-slate-800 sticky top-4">
      <CardHeader className="pb-4">
        <CardTitle className="flex items-center gap-2 text-slate-100">
          <Settings2 className="h-5 w-5 text-indigo-500" />
          Optimizer Config
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">

        <div className="space-y-2">
          <Label className="text-slate-400 text-xs uppercase tracking-wider font-bold">
            Tickers (Comma Separated)
          </Label>
          <div className="relative">
            <Input
              value={symbolsInput}
              onChange={(e) => setSymbolsInput(e.target.value)}
              className="bg-slate-950 border-slate-800 text-slate-200 font-mono h-16"
              placeholder="BTCUSDT, ETHUSDT..."
            />
          </div>
        </div>

        <div className="space-y-3">
          <Label className="text-slate-400 text-xs uppercase tracking-wider font-bold">
            Timeframes
          </Label>
          <div className="flex flex-wrap gap-2">
            {AVAILABLE_INTERVALS.map((interval) => (
              <Badge
                key={interval}
                variant={selectedIntervals.includes(interval) ? "default" : "outline"}
                className={`cursor-pointer hover:bg-indigo-500/20 transition-all ${selectedIntervals.includes(interval) ? 'bg-indigo-600 hover:bg-indigo-700' : 'border-slate-700 text-slate-400'}`}
                onClick={() => toggleInterval(interval)}
              >
                {interval}
              </Badge>
            ))}
          </div>
        </div>

        <div className="grid grid-cols-1 gap-6">
            <div className="space-y-4">
            <div className="flex justify-between items-center">
                <Label className="text-slate-400 text-xs uppercase tracking-wider font-bold">
                History Depth (Candles)
                </Label>
                <span className="text-xs font-mono text-indigo-400">{historyLimit[0]}</span>
            </div>
            <Slider
                value={historyLimit}
                onValueChange={setHistoryLimit}
                max={5000}
                min={200}
                step={100}
                className="py-2"
            />
            </div>

            <div className="space-y-4">
            <div className="flex justify-between items-center">
                <Label className="text-slate-400 text-xs uppercase tracking-wider font-bold">
                Optimizer Iterations
                </Label>
                <span className="text-xs font-mono text-purple-400">{iterations[0]}</span>
            </div>
            <Slider
                value={iterations}
                onValueChange={setIterations}
                max={500}
                min={10}
                step={10}
                className="py-2"
            />
            <p className="text-[10px] text-slate-500">
                Generations of parameters to test per strategy type.
            </p>
            </div>
        </div>

        <div className="space-y-2">
          <Label className="text-slate-400 text-xs uppercase tracking-wider font-bold">
            Strategies to Keep (Top N)
          </Label>
          <Input
            type="number"
            value={topN}
            onChange={(e) => setTopN(parseInt(e.target.value) || 1)}
            className="bg-slate-950 border-slate-800 text-slate-200 font-mono"
            min={1}
            max={50}
          />
        </div>

        <Button
            onClick={() => mutation.mutate()}
            disabled={mutation.isPending}
            className="w-full bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-700 hover:to-purple-700 text-white font-bold shadow-lg shadow-indigo-900/20"
        >
          {mutation.isPending ? (
            <>
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              Running Genetic Algo...
            </>
          ) : (
            <>
              <Zap className="mr-2 h-4 w-4" />
              Run Optimizer
            </>
          )}
        </Button>
      </CardContent>
    </Card>
  );
}

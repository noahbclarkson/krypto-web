import { Generator } from "@/components/Generator";
import { StrategyList } from "@/components/StrategyList";
import { ActiveSessions } from "@/components/ActiveSessions";
import { PortfolioChart } from "@/components/PortfolioChart";

export default function Home() {
  return (
    <div className="max-w-7xl mx-auto">
      <header className="mb-10 border-b border-slate-800 pb-6">
        <h1 className="text-4xl font-extrabold tracking-tight text-white">
          KRYPTO <span className="text-indigo-500">V6</span>
        </h1>
        <p className="text-slate-400 mt-2">Institutional Algo-Trading Engine & Backtester</p>
      </header>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
        {/* Left Sidebar / Controls */}
        <div className="lg:col-span-1 space-y-6">
          <Generator />
        </div>

        {/* Main Content */}
        <div className="lg:col-span-3">
          <PortfolioChart />
          <ActiveSessions />
          <h2 className="text-2xl font-bold text-white mt-10 mb-4">Strategy Pool</h2>
          <StrategyList />
        </div>
      </div>
    </div>
  );
}

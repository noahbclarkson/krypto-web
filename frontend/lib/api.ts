import axios from "axios";
import { Strategy, Session, Trade, EquitySnapshot, PortfolioPoint, Candle } from "./types";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";

export const api = {
  // Generators
  generateStrategies: async (symbols: string[], intervals: string[], top_n: number, limit: number, iterations: number) => {
    const res = await axios.post(`${API_URL}/strategies/generate`, {
      symbols,
      intervals,
      top_n,
      limit,
      iterations
    });
    return res.data;
  },

  // Strategies
  getStrategies: async (): Promise<Strategy[]> => {
    const res = await axios.get(`${API_URL}/strategies`);
    return res.data;
  },

  deleteStrategy: async (id: string) => {
    const res = await axios.delete(`${API_URL}/strategies/${id}`);
    return res.data;
  },

  deleteAllStrategies: async () => {
    const res = await axios.delete(`${API_URL}/strategies`);
    return res.data;
  },

  // Sessions
  startSession: async (params: { strategy_id: string; initial_capital: number; execution_mode?: string }) => {
    const res = await axios.post(`${API_URL}/sessions`, params);
    return res.data;
  },

  startSessionBulk: async (strategy_ids: string[]) => {
    const res = await axios.post(`${API_URL}/sessions/bulk`, { strategy_ids });
    return res.data;
  },

  getSessions: async (): Promise<Session[]> => {
    const res = await axios.get(`${API_URL}/sessions`);
    return res.data;
  },

  resetSessions: async () => {
    const res = await axios.post(`${API_URL}/sessions/reset`);
    return res.data;
  },

  getSessionTrades: async (id: string): Promise<Trade[]> => {
    const res = await axios.get(`${API_URL}/sessions/${id}/trades`);
    return res.data;
  },

  getEquityCurve: async (id: string): Promise<EquitySnapshot[]> => {
    const res = await axios.get(`${API_URL}/sessions/${id}/equity`);
    return res.data;
  },

  getSessionCandles: async (id: string): Promise<Candle[]> => {
    const res = await axios.get(`${API_URL}/sessions/${id}/candles`);
    return res.data;
  },

  // Portfolio
  getPortfolioHistory: async (params?: { rangeDays?: number; interval?: string }): Promise<PortfolioPoint[]> => {
    const res = await axios.get(`${API_URL}/portfolio/history`, {
      params: {
        range_days: params?.rangeDays,
        interval: params?.interval
      }
    });
    return res.data;
  },
};

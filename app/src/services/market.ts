import api from './api';
import type { OrderBook, Ticker, KlineParams } from '../types';

export const marketService = {
  async getDepth(marketId: number, outcomeId: number, limit: number = 20): Promise<OrderBook> {
    const response = await api.get('/market/depth', {
      params: { market_id: marketId, outcome_id: outcomeId, limit },
    });
    return response.data;
  },

  async getTicker(marketId: number, outcomeId: number): Promise<Ticker> {
    const response = await api.get('/market/ticker', {
      params: { market_id: marketId, outcome_id: outcomeId },
    });
    return response.data;
  },

  async getKline(params: KlineParams): Promise<unknown> {
    const response = await api.get('/market/kline', { params });
    return response.data;
  },

  async getRecentTrades(marketId: number, outcomeId: number, limit: number = 50): Promise<unknown> {
    const response = await api.get('/market/trades', {
      params: { market_id: marketId, outcome_id: outcomeId, limit },
    });
    return response.data;
  },
};

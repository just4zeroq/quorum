/**
 * Market Service - 行情相关 API
 */
import api from './api';

export const marketService = {
  /**
   * 获取订单簿深度
   * @param {number} marketId
   * @param {number} outcomeId
   * @param {number} limit
   */
  async getDepth(marketId, outcomeId, limit = 20) {
    const response = await api.get('/market/depth', {
      params: { market_id: marketId, outcome_id: outcomeId, limit },
    });
    return response.data;
  },

  /**
   * 获取行情 ticker
   * @param {number} marketId
   * @param {number} outcomeId
   */
  async getTicker(marketId, outcomeId) {
    const response = await api.get('/market/ticker', {
      params: { market_id: marketId, outcome_id: outcomeId },
    });
    return response.data;
  },

  /**
   * 获取 K 线数据
   * @param {object} params
   */
  async getKline(params) {
    const response = await api.get('/market/kline', { params });
    return response.data;
  },

  /**
   * 获取最近成交
   * @param {number} marketId
   * @param {number} outcomeId
   * @param {number} limit
   */
  async getRecentTrades(marketId, outcomeId, limit = 50) {
    const response = await api.get('/market/trades', {
      params: { market_id: marketId, outcome_id: outcomeId, limit },
    });
    return response.data;
  },
};

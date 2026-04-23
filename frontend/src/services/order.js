/**
 * Order Service - 订单相关 API
 */
import api from './api';

export const orderService = {
  /**
   * 创建订单
   * @param {object} orderData
   */
  async createOrder(orderData) {
    const response = await api.post('/orders', orderData);
    return response.data;
  },

  /**
   * 获取订单列表
   * @param {object} params
   */
  async getOrders(params = {}) {
    const response = await api.get('/orders', { params });
    return response.data;
  },

  /**
   * 获取订单详情
   * @param {string} orderId
   */
  async getOrder(orderId) {
    const response = await api.get(`/orders/${orderId}`);
    return response.data;
  },

  /**
   * 取消订单
   * @param {string} orderId
   */
  async cancelOrder(orderId) {
    const response = await api.delete(`/orders/${orderId}`);
    return response.data;
  },

  /**
   * 获取持仓
   */
  async getPositions() {
    const response = await api.get('/positions');
    return response.data;
  },
};

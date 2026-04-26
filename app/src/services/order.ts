import api from './api';
import type { Order, CreateOrderParams, Position } from '../types';

export const orderService = {
  async createOrder(orderData: CreateOrderParams): Promise<Order> {
    const response = await api.post('/orders', orderData);
    return response.data;
  },

  async getOrders(params: Record<string, unknown> = {}): Promise<Order[]> {
    const response = await api.get('/orders', { params });
    return response.data;
  },

  async getOrder(orderId: string): Promise<Order> {
    const response = await api.get(`/orders/${orderId}`);
    return response.data;
  },

  async cancelOrder(orderId: string): Promise<void> {
    const response = await api.delete(`/orders/${orderId}`);
    return response.data;
  },

  async getPositions(): Promise<Position[]> {
    const response = await api.get('/positions');
    return response.data;
  },
};

import api from './api';
import type { Balance, WalletHistory } from '../types';

export const walletService = {
  async getDepositAddress(asset: string, network: string): Promise<{ address: string }> {
    const response = await api.get('/wallet/deposit/address', {
      params: { asset, network },
    });
    return response.data;
  },

  async withdraw(asset: string, amount: string, address: string, network: string): Promise<void> {
    const response = await api.post('/wallet/withdraw', { asset, amount, address, network });
    return response.data;
  },

  async getHistory(params: Record<string, unknown> = {}): Promise<WalletHistory> {
    const response = await api.get('/wallet/history', { params });
    return response.data;
  },

  async getBalance(): Promise<Balance> {
    const response = await api.get('/accounts/balance');
    return response.data;
  },
};

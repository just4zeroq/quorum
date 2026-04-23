/**
 * Wallet Service - 钱包相关 API
 */
import api from './api';

export const walletService = {
  /**
   * 获取充值地址
   * @param {string} asset - 资产类型 (USDT, TRX, etc.)
   * @param {string} network - 网络类型 (TRC20, ERC20, etc.)
   */
  async getDepositAddress(asset, network) {
    const response = await api.get('/wallet/deposit/address', {
      params: { asset, network },
    });
    return response.data;
  },

  /**
   * 申请取现
   * @param {string} asset - 资产类型
   * @param {string} amount - 金额
   * @param {string} address - 目标地址
   * @param {string} network - 网络类型
   */
  async withdraw(asset, amount, address, network) {
    const response = await api.post('/wallet/withdraw', {
      asset,
      amount,
      address,
      network,
    });
    return response.data;
  },

  /**
   * 获取钱包历史
   * @param {object} params - 查询参数
   */
  async getHistory(params = {}) {
    const response = await api.get('/wallet/history', { params });
    return response.data;
  },

  /**
   * 获取账户余额
   */
  async getBalance() {
    const response = await api.get('/accounts/balance');
    return response.data;
  },
};

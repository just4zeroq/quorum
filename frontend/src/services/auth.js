/**
 * Auth Service - 认证相关 API
 */
import api from './api';

export const authService = {
  /**
   * 用户注册
   * @param {string} email
   * @param {string} password
   * @param {string} username
   */
  async register(email, password, username) {
    const response = await api.post('/users/register', {
      email,
      password,
      username,
    });
    return response.data;
  },

  /**
   * 用户登录
   * @param {string} email
   * @param {string} password
   */
  async login(email, password) {
    const response = await api.post('/users/login', {
      email,
      password,
    });
    return response.data;
  },

  /**
   * 获取当前用户信息
   */
  async getCurrentUser() {
    const response = await api.get('/users/me');
    return response.data;
  },

  /**
   * 更新用户信息
   * @param {object} data
   */
  async updateProfile(data) {
    const response = await api.put('/users/me', data);
    return response.data;
  },

  /**
   * 刷新 Token
   * @param {string} refreshToken
   */
  async refreshToken(refreshToken) {
    const response = await api.post('/users/refresh', {
      refresh_token: refreshToken,
    });
    return response.data;
  },

  /**
   * 登出
   */
  async logout() {
    const response = await api.post('/users/logout');
    return response.data;
  },
};

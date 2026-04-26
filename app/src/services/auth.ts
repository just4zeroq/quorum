import api from './api';
import type { LoginResponse, User } from '../types';

export const authService = {
  async register(email: string, password: string, username: string): Promise<void> {
    const response = await api.post('/users/register', { email, password, username });
    return response.data;
  },

  async login(email: string, password: string): Promise<LoginResponse> {
    const response = await api.post('/users/login', { email, password });
    return response.data;
  },

  async getCurrentUser(): Promise<User> {
    const response = await api.get('/users/me');
    return response.data;
  },

  async updateProfile(data: Partial<User>): Promise<User> {
    const response = await api.put('/users/me', data);
    return response.data;
  },

  async refreshToken(refreshToken: string): Promise<LoginResponse> {
    const response = await api.post('/users/refresh', { refresh_token: refreshToken });
    return response.data;
  },

  async logout(): Promise<void> {
    const response = await api.post('/users/logout');
    return response.data;
  },
};

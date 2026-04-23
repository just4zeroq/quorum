/**
 * Auth Context - 认证状态管理
 */
import { createContext, useContext, useState, useEffect, useCallback } from 'react';
import { authService } from '../services/auth';

const AuthContext = createContext(null);

export function AuthProvider({ children }) {
  const [user, setUser] = useState(null);
  const [loading, setLoading] = useState(true);
  const [token, setToken] = useState(localStorage.getItem('token'));

  // 初始化 - 检查是否已登录
  useEffect(() => {
    const initAuth = async () => {
      const storedToken = localStorage.getItem('token');
      if (storedToken) {
        try {
          const userData = await authService.getCurrentUser();
          setUser(userData);
          setToken(storedToken);
        } catch (error) {
          console.error('Failed to get current user:', error);
          localStorage.removeItem('token');
          localStorage.removeItem('refresh_token');
          setToken(null);
        }
      }
      setLoading(false);
    };
    initAuth();
  }, []);

  // 登录
  const login = useCallback(async (email, password) => {
    const response = await authService.login(email, password);
    const { token: newToken, refresh_token } = response;

    localStorage.setItem('token', newToken);
    localStorage.setItem('refresh_token', refresh_token);
    setToken(newToken);

    // 获取用户信息
    const userData = await authService.getCurrentUser();
    setUser(userData);

    return response;
  }, []);

  // 注册
  const register = useCallback(async (email, password, username) => {
    const response = await authService.register(email, password, username);
    return response;
  }, []);

  // 登出
  const logout = useCallback(async () => {
    try {
      await authService.logout();
    } catch (error) {
      console.error('Logout error:', error);
    } finally {
      localStorage.removeItem('token');
      localStorage.removeItem('refresh_token');
      setToken(null);
      setUser(null);
    }
  }, []);

  // 更新用户信息
  const updateUser = useCallback((userData) => {
    setUser(userData);
  }, []);

  const value = {
    user,
    token,
    loading,
    isAuthenticated: !!token,
    login,
    register,
    logout,
    updateUser,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within AuthProvider');
  }
  return context;
}

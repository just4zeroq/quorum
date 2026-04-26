// Types for Auth
export interface User {
  id?: number;
  user_id?: number;
  username?: string;
  email?: string;
}

export interface LoginResponse {
  token: string;
  refresh_token: string;
}

export interface AuthContextType {
  user: User | null;
  token: string | null;
  loading: boolean;
  isAuthenticated: boolean;
  login: (email: string, password: string) => Promise<LoginResponse>;
  register: (email: string, password: string, username: string) => Promise<void>;
  logout: () => Promise<void>;
  updateUser: (userData: User) => void;
}

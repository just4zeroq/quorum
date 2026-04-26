import { Outlet, Link, useNavigate } from 'react-router-dom';
import { useAuth } from '../../context/AuthContext';

export default function AppLayout() {
  const { user, logout, isAuthenticated } = useAuth();
  const navigate = useNavigate();

  const handleLogout = async () => {
    await logout();
    navigate('/login');
  };

  return (
    <div className="min-h-screen bg-gray-900 flex flex-col">
      {/* Header */}
      <header className="bg-gray-800 border-b border-gray-700">
        <div className="max-w-7xl mx-auto px-4">
          <div className="flex items-center justify-between h-16">
            {/* Logo */}
            <Link to="/" className="flex items-center space-x-2">
              <span className="text-2xl">🎯</span>
              <span className="text-xl font-bold text-white">Prediction Market</span>
            </Link>

            {/* Navigation */}
            <nav className="flex items-center space-x-6">
              {isAuthenticated ? (
                <>
                  <Link to="/dashboard" className="text-gray-300 hover:text-white">
                    Dashboard
                  </Link>
                  <Link to="/markets" className="text-gray-300 hover:text-white">
                    Markets
                  </Link>
                  <Link to="/wallet" className="text-gray-300 hover:text-white">
                    Wallet
                  </Link>

                  {/* User Menu */}
                  <div className="flex items-center space-x-4 ml-4">
                    <span className="text-gray-300">{user?.username || user?.email}</span>
                    <button
                      onClick={handleLogout}
                      className="px-4 py-2 bg-gray-700 text-gray-300 rounded hover:bg-gray-600 transition"
                    >
                      Logout
                    </button>
                  </div>
                </>
              ) : (
                <>
                  <Link
                    to="/login"
                    className="px-4 py-2 text-gray-300 hover:text-white transition"
                  >
                    Login
                  </Link>
                  <Link
                    to="/register"
                    className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition"
                  >
                    Sign Up
                  </Link>
                </>
              )}
            </nav>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 py-8 flex-1">
        <Outlet />
      </main>

      {/* Footer */}
      <footer className="bg-gray-800 border-t border-gray-700 mt-auto">
        <div className="max-w-7xl mx-auto px-4 py-4 text-center text-gray-400">
          &copy; 2024 Prediction Market. All rights reserved.
        </div>
      </footer>
    </div>
  );
}

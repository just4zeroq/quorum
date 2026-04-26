import { useAuth } from '../context/AuthContext';
import { useEffect, useState } from 'react';
import { walletService } from '../services/wallet';
import { orderService } from '../services/order';
import type { Balance, Order, Position } from '../types';

export default function DashboardPage() {
  const { user } = useAuth();
  const [balance, setBalance] = useState<Balance>({ available: '0', frozen: '0' });
  const [positions, setPositions] = useState<Position[]>([]);
  const [recentOrders, setRecentOrders] = useState<Order[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [balanceData, positionsData, ordersData] = await Promise.all([
          walletService.getBalance(),
          orderService.getPositions(),
          orderService.getOrders({ limit: 5 }),
        ]);
        setBalance(balanceData);
        setPositions(positionsData || []);
        setRecentOrders(ordersData || []);
      } catch (error) {
        console.error('Failed to fetch dashboard data:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, []);

  return (
    <div className="space-y-6">
      <h1 className="text-3xl font-bold text-white">Dashboard</h1>

      {/* Welcome Section */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h2 className="text-xl text-white mb-2">Welcome, {user?.username || user?.email}!</h2>
        <p className="text-gray-400">
          User ID: {user?.id || user?.user_id}
        </p>
      </div>

      {/* Balance Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="bg-gray-800 rounded-lg p-6">
          <h3 className="text-gray-400 mb-2">Available Balance</h3>
          <p className="text-3xl font-bold text-green-400">
            ${balance.available || '0.00'}
          </p>
        </div>
        <div className="bg-gray-800 rounded-lg p-6">
          <h3 className="text-gray-400 mb-2">Frozen Balance</h3>
          <p className="text-3xl font-bold text-yellow-400">
            ${balance.frozen || '0.00'}
          </p>
        </div>
        <div className="bg-gray-800 rounded-lg p-6">
          <h3 className="text-gray-400 mb-2">Total Equity</h3>
          <p className="text-3xl font-bold text-blue-400">
            ${Number(balance.available || 0) + Number(balance.frozen || 0)}
          </p>
        </div>
      </div>

      {/* Recent Orders */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-xl text-white mb-4">Recent Orders</h3>
        {loading ? (
          <p className="text-gray-400">Loading...</p>
        ) : recentOrders.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="text-gray-400 border-b border-gray-700">
                  <th className="text-left py-3">Market</th>
                  <th className="text-left py-3">Side</th>
                  <th className="text-left py-3">Price</th>
                  <th className="text-left py-3">Quantity</th>
                  <th className="text-left py-3">Status</th>
                </tr>
              </thead>
              <tbody>
                {recentOrders.map((order) => (
                  <tr key={order.order_id} className="text-white border-b border-gray-700">
                    <td className="py-3">{order.market_id}</td>
                    <td className="py-3">
                      <span
                        className={`px-2 py-1 rounded ${
                          order.side === 'YES' ? 'bg-green-600' : 'bg-red-600'
                        }`}
                      >
                        {order.side}
                      </span>
                    </td>
                    <td className="py-3">{order.price}</td>
                    <td className="py-3">{order.quantity}</td>
                    <td className="py-3">{order.status}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="text-gray-400">No recent orders</p>
        )}
      </div>

      {/* Positions */}
      <div className="bg-gray-800 rounded-lg p-6">
        <h3 className="text-xl text-white mb-4">Your Positions</h3>
        {loading ? (
          <p className="text-gray-400">Loading...</p>
        ) : positions.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="text-gray-400 border-b border-gray-700">
                  <th className="text-left py-3">Market</th>
                  <th className="text-left py-3">Side</th>
                  <th className="text-left py-3">Size</th>
                  <th className="text-left py-3">Entry Price</th>
                  <th className="text-left py-3">P&amp;L</th>
                </tr>
              </thead>
              <tbody>
                {positions.map((position) => (
                  <tr key={position.position_id} className="text-white border-b border-gray-700">
                    <td className="py-3">{position.market_id}</td>
                    <td className="py-3">{position.side}</td>
                    <td className="py-3">{position.size}</td>
                    <td className="py-3">{position.entry_price}</td>
                    <td className={`py-3 ${position.pnl >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                      {position.pnl >= 0 ? '+' : ''}{position.pnl}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="text-gray-400">No open positions</p>
        )}
      </div>
    </div>
  );
}

import { useState, useEffect } from 'react';
import { orderService } from '../services/order';
import { marketService } from '../services/market';
import toast from 'react-hot-toast';
import type { OrderBook, Ticker, Order } from '../types';

export default function TradingPage() {
  const [orderType, setOrderType] = useState<'limit' | 'market'>('limit');
  const [side, setSide] = useState<'YES' | 'NO'>('YES');
  const [price, setPrice] = useState('');
  const [quantity, setQuantity] = useState('');
  const [marketId, setMarketId] = useState(1);
  const [outcomeId, setOutcomeId] = useState(1);
  const [orderbook, setOrderbook] = useState<OrderBook>({ asks: [], bids: [] });
  const [ticker, setTicker] = useState<Ticker | null>(null);
  const [loading, setLoading] = useState(false);
  const [userOrders, setUserOrders] = useState<Order[]>([]);

  const fetchMarketData = async () => {
    try {
      const [depthData, tickerData] = await Promise.all([
        marketService.getDepth(marketId, outcomeId),
        marketService.getTicker(marketId, outcomeId),
      ]);
      setOrderbook(depthData);
      setTicker(tickerData);
    } catch (error) {
      console.error('Failed to fetch market data:', error);
    }
  };

  const fetchUserOrders = async () => {
    try {
      const orders = await orderService.getOrders({ limit: 10 });
      setUserOrders(orders || []);
    } catch (error) {
      console.error('Failed to fetch orders:', error);
    }
  };

  useEffect(() => {
    fetchMarketData();
    fetchUserOrders();

    const interval = setInterval(() => {
      fetchMarketData();
      fetchUserOrders();
    }, 5000);

    return () => clearInterval(interval);
  }, [marketId, outcomeId]);

  const handlePlaceOrder = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);

    try {
      const orderData = {
        market_id: marketId,
        outcome_id: outcomeId,
        side,
        order_type: orderType,
        price: orderType === 'market' ? '0' : price,
        quantity,
      };

      await orderService.createOrder(orderData);
      toast.success('Order placed successfully!');
      setPrice('');
      setQuantity('');
      fetchUserOrders();
    } catch (error: unknown) {
      const err = error as { response?: { data?: { message?: string } } };
      const message = err.response?.data?.message || 'Failed to place order';
      toast.error(message);
    } finally {
      setLoading(false);
    }
  };

  const handleCancelOrder = async (orderId: string) => {
    try {
      await orderService.cancelOrder(orderId);
      toast.success('Order cancelled!');
      fetchUserOrders();
    } catch {
      toast.error('Failed to cancel order');
    }
  };

  return (
    <div className="space-y-6">
      <h1 className="text-3xl font-bold text-white">Trade</h1>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Order Form */}
        <div className="lg:col-span-1">
          <div className="bg-gray-800 rounded-lg p-6">
            <h3 className="text-xl text-white mb-6">Place Order</h3>

            {/* Order Type */}
            <div className="flex space-x-2 mb-6">
              <button
                onClick={() => setOrderType('limit')}
                className={`flex-1 py-2 rounded ${
                  orderType === 'limit' ? 'bg-blue-600 text-white' : 'bg-gray-700 text-gray-300'
                }`}
              >
                Limit
              </button>
              <button
                onClick={() => setOrderType('market')}
                className={`flex-1 py-2 rounded ${
                  orderType === 'market' ? 'bg-blue-600 text-white' : 'bg-gray-700 text-gray-300'
                }`}
              >
                Market
              </button>
            </div>

            {/* Side */}
            <div className="flex space-x-2 mb-6">
              <button
                onClick={() => setSide('YES')}
                className={`flex-1 py-3 rounded font-bold ${
                  side === 'YES'
                    ? 'bg-green-600 text-white'
                    : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                }`}
              >
                YES
              </button>
              <button
                onClick={() => setSide('NO')}
                className={`flex-1 py-3 rounded font-bold ${
                  side === 'NO'
                    ? 'bg-red-600 text-white'
                    : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                }`}
              >
                NO
              </button>
            </div>

            <form onSubmit={handlePlaceOrder} className="space-y-4">
              {orderType === 'limit' && (
                <div>
                  <label className="block text-gray-400 mb-2">Price (Probability)</label>
                  <input
                    type="number"
                    value={price}
                    onChange={(e) => setPrice(e.target.value)}
                    className="w-full px-4 py-3 bg-gray-700 text-white rounded focus:outline-none"
                    placeholder="0.50"
                    min="0"
                    max="1"
                    step="0.01"
                    required
                  />
                </div>
              )}

              <div>
                <label className="block text-gray-400 mb-2">Quantity (Shares)</label>
                <input
                  type="number"
                  value={quantity}
                  onChange={(e) => setQuantity(e.target.value)}
                  className="w-full px-4 py-3 bg-gray-700 text-white rounded focus:outline-none"
                  placeholder="100"
                  min="1"
                  step="1"
                  required
                />
              </div>

              {price && quantity && (
                <div className="bg-gray-700 rounded p-4">
                  <div className="flex justify-between text-gray-400">
                    <span>Potential P&amp;L:</span>
                    <span className={side === 'YES' ? 'text-green-400' : 'text-red-400'}>
                      ${(Number(quantity) * (1 - Number(price))).toFixed(2)}
                    </span>
                  </div>
                  <div className="flex justify-between text-gray-400 mt-2">
                    <span>Max Win:</span>
                    <span className="text-green-400">
                      ${(Number(quantity) * Number(price)).toFixed(2)}
                    </span>
                  </div>
                </div>
              )}

              <button
                type="submit"
                disabled={loading}
                className={`w-full py-3 rounded font-bold ${
                  side === 'YES'
                    ? 'bg-green-600 hover:bg-green-700'
                    : 'bg-red-600 hover:bg-red-700'
                } text-white disabled:opacity-50`}
              >
                {loading ? 'Placing...' : `Buy ${side}`}
              </button>
            </form>
          </div>
        </div>

        {/* Order Book */}
        <div className="lg:col-span-1">
          <div className="bg-gray-800 rounded-lg p-6">
            <h3 className="text-xl text-white mb-4">Order Book</h3>

            {/* Asks (Sells) */}
            <div className="space-y-1 mb-4">
              <div className="flex text-gray-400 text-sm">
                <span className="flex-1">Price</span>
                <span className="flex-1 text-right">Shares</span>
              </div>
              {orderbook.asks?.slice(0, 5).map((ask, i) => (
                <div key={`ask-${i}`} className="flex items-center">
                  <div
                    className="bg-red-600 h-6"
                    style={{ width: `${(Number(ask[1]) / 1000) * 100}%` }}
                  />
                  <span className="flex-1 text-red-400 ml-2">{ask[0]}</span>
                  <span className="flex-1 text-right text-white">{ask[1]}</span>
                </div>
              ))}
            </div>

            {/* Spread */}
            <div className="border-y border-gray-700 py-2 mb-4">
              <div className="flex justify-between">
                <span className="text-gray-400">Spread</span>
                <span className="text-white">
                  {orderbook.asks?.[0] && orderbook.bids?.[0]
                    ? (Number(orderbook.asks[0][0]) - Number(orderbook.bids[0][0])).toFixed(4)
                    : '0'}
                </span>
              </div>
            </div>

            {/* Bids (Buys) */}
            <div className="space-y-1">
              {orderbook.bids?.slice(0, 5).map((bid, i) => (
                <div key={`bid-${i}`} className="flex items-center">
                  <div
                    className="bg-green-600 h-6"
                    style={{ width: `${(Number(bid[1]) / 1000) * 100}%` }}
                  />
                  <span className="flex-1 text-green-400 ml-2">{bid[0]}</span>
                  <span className="flex-1 text-right text-white">{bid[1]}</span>
                </div>
              ))}
            </div>
          </div>

          {/* Ticker */}
          {ticker && (
            <div className="bg-gray-800 rounded-lg p-6 mt-6">
              <h3 className="text-xl text-white mb-4">Market Stats</h3>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <span className="text-gray-400">Last Price</span>
                  <p className="text-2xl font-bold text-white">{ticker.last_price}</p>
                </div>
                <div>
                  <span className="text-gray-400">24h Change</span>
                  <p className={`text-2xl font-bold ${Number(ticker.price_change) >= 0 ? 'text-green-400' : 'text-red-400'}`}>
                    {Number(ticker.price_change) >= 0 ? '+' : ''}{ticker.price_change}
                  </p>
                </div>
                <div>
                  <span className="text-gray-400">24h High</span>
                  <p className="text-white">{ticker.high_price}</p>
                </div>
                <div>
                  <span className="text-gray-400">24h Low</span>
                  <p className="text-white">{ticker.low_price}</p>
                </div>
                <div>
                  <span className="text-gray-400">24h Volume</span>
                  <p className="text-white">{ticker.volume}</p>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* User Orders */}
        <div className="lg:col-span-1">
          <div className="bg-gray-800 rounded-lg p-6">
            <h3 className="text-xl text-white mb-4">My Orders</h3>

            {userOrders.length > 0 ? (
              <div className="space-y-3">
                {userOrders.map((order) => (
                  <div key={order.order_id} className="bg-gray-700 rounded p-4">
                    <div className="flex justify-between items-start mb-2">
                      <div>
                        <span className={`px-2 py-1 rounded text-sm ${
                          order.side === 'YES' ? 'bg-green-600' : 'bg-red-600'
                        }`}>
                          {order.side}
                        </span>
                        <span className="ml-2 text-gray-400 text-sm">
                          {order.order_type}
                        </span>
                      </div>
                      <button
                        onClick={() => handleCancelOrder(order.order_id)}
                        className="text-red-400 hover:text-red-300 text-sm"
                      >
                        Cancel
                      </button>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span className="text-gray-400">Price: {order.price}</span>
                      <span className="text-gray-400">Qty: {order.quantity}</span>
                    </div>
                    <div className="flex justify-between text-sm mt-1">
                      <span className="text-gray-400">Filled: {order.filled_quantity || 0}</span>
                      <span className={`px-2 py-0.5 rounded text-xs ${
                        order.status === 'filled' ? 'bg-green-600' :
                        order.status === 'pending' ? 'bg-yellow-600' : 'bg-gray-600'
                      }`}>
                        {order.status}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-gray-400 text-center py-8">No open orders</p>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

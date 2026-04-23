/**
 * Wallet Page - Deposit & Withdraw
 */
import { useState, useEffect } from 'react';
import { walletService } from '../services/wallet';
import toast from 'react-hot-toast';

const ASSETS = [
  { symbol: 'USDT', name: 'Tether', networks: ['TRC20', 'ERC20'] },
  { symbol: 'TRX', name: 'TRON', networks: ['TRC20'] },
  { symbol: 'BTC', name: 'Bitcoin', networks: ['BTC', 'Lightning'] },
];

export default function WalletPage() {
  const [activeTab, setActiveTab] = useState('deposit');
  const [selectedAsset, setSelectedAsset] = useState('USDT');
  const [selectedNetwork, setSelectedNetwork] = useState('TRC20');
  const [depositAddress, setDepositAddress] = useState('');
  const [history, setHistory] = useState({ deposits: [], withdrawals: [] });
  const [loading, setLoading] = useState(false);
  const [balance, setBalance] = useState({ available: '0', frozen: '0' });

  // Fetch deposit address
  const fetchDepositAddress = async () => {
    setLoading(true);
    try {
      const addressData = await walletService.getDepositAddress(selectedAsset, selectedNetwork);
      setDepositAddress(addressData.address);
    } catch (error) {
      console.error('Failed to fetch deposit address:', error);
      toast.error('Failed to get deposit address');
    } finally {
      setLoading(false);
    }
  };

  // Fetch history
  const fetchHistory = async () => {
    try {
      const historyData = await walletService.getHistory();
      setHistory(historyData);
    } catch (error) {
      console.error('Failed to fetch history:', error);
    }
  };

  // Fetch balance
  const fetchBalance = async () => {
    try {
      const balanceData = await walletService.getBalance();
      setBalance(balanceData);
    } catch (error) {
      console.error('Failed to fetch balance:', error);
    }
  };

  useEffect(() => {
    fetchBalance();
    fetchHistory();
  }, []);

  useEffect(() => {
    if (activeTab === 'deposit') {
      fetchDepositAddress();
    }
  }, [selectedAsset, selectedNetwork, activeTab]);

  const copyAddress = () => {
    navigator.clipboard.writeText(depositAddress);
    toast.success('Address copied to clipboard!');
  };

  return (
    <div className="space-y-6">
      <h1 className="text-3xl font-bold text-white">Wallet</h1>

      {/* Balance Overview */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
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
      </div>

      {/* Tabs */}
      <div className="flex space-x-4 border-b border-gray-700">
        <button
          onClick={() => setActiveTab('deposit')}
          className={`px-6 py-3 text-white border-b-2 ${
            activeTab === 'deposit'
              ? 'border-blue-500 text-blue-400'
              : 'border-transparent text-gray-400 hover:text-white'
          }`}
        >
          Deposit
        </button>
        <button
          onClick={() => setActiveTab('withdraw')}
          className={`px-6 py-3 text-white border-b-2 ${
            activeTab === 'withdraw'
              ? 'border-blue-500 text-blue-400'
              : 'border-transparent text-gray-400 hover:text-white'
          }`}
        >
          Withdraw
        </button>
        <button
          onClick={() => setActiveTab('history')}
          className={`px-6 py-3 text-white border-b-2 ${
            activeTab === 'history'
              ? 'border-blue-500 text-blue-400'
              : 'border-transparent text-gray-400 hover:text-white'
          }`}
        >
          History
        </button>
      </div>

      {/* Deposit Tab */}
      {activeTab === 'deposit' && (
        <div className="bg-gray-800 rounded-lg p-6">
          <h3 className="text-xl text-white mb-6">Deposit Crypto</h3>

          {/* Asset Selection */}
          <div className="mb-6">
            <label className="block text-gray-400 mb-2">Select Asset</label>
            <div className="flex space-x-4">
              {ASSETS.map((asset) => (
                <button
                  key={asset.symbol}
                  onClick={() => {
                    setSelectedAsset(asset.symbol);
                    setSelectedNetwork(asset.networks[0]);
                  }}
                  className={`px-6 py-3 rounded-lg ${
                    selectedAsset === asset.symbol
                      ? 'bg-blue-600 text-white'
                      : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                  }`}
                >
                  {asset.symbol}
                </button>
              ))}
            </div>
          </div>

          {/* Network Selection */}
          <div className="mb-6">
            <label className="block text-gray-400 mb-2">Select Network</label>
            <div className="flex space-x-4">
              {ASSETS.find((a) => a.symbol === selectedAsset)?.networks.map((network) => (
                <button
                  key={network}
                  onClick={() => setSelectedNetwork(network)}
                  className={`px-6 py-3 rounded-lg ${
                    selectedNetwork === network
                      ? 'bg-blue-600 text-white'
                      : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                  }`}
                >
                  {network}
                </button>
              ))}
            </div>
          </div>

          {/* Deposit Address */}
          <div className="mb-6">
            <label className="block text-gray-400 mb-2">Deposit Address</label>
            <div className="flex space-x-4">
              <input
                type="text"
                value={depositAddress}
                readOnly
                className="flex-1 px-4 py-3 bg-gray-700 text-white rounded focus:outline-none"
                placeholder="Loading..."
              />
              <button
                onClick={copyAddress}
                disabled={!depositAddress}
                className="px-6 py-3 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
              >
                Copy
              </button>
            </div>
          </div>

          {/* QR Code placeholder */}
          <div className="bg-gray-700 w-48 h-48 rounded-lg flex items-center justify-center">
            <span className="text-gray-400">QR Code</span>
          </div>

          <p className="text-yellow-400 mt-4 text-sm">
            ⚠️ Send only {selectedAsset} ({selectedNetwork}) to this address. Do not send other assets.
          </p>
        </div>
      )}

      {/* Withdraw Tab */}
      {activeTab === 'withdraw' && (
        <WithdrawForm assets={ASSETS} />
      )}

      {/* History Tab */}
      {activeTab === 'history' && (
        <HistoryTab history={history} />
      )}
    </div>
  );
}

/**
 * Withdraw Form Component
 */
function WithdrawForm({ assets }) {
  const [asset, setAsset] = useState('USDT');
  const [network, setNetwork] = useState('TRC20');
  const [address, setAddress] = useState('');
  const [amount, setAmount] = useState('');
  const [loading, setLoading] = useState(false);

  const handleWithdraw = async (e) => {
    e.preventDefault();
    setLoading(true);

    try {
      await walletService.withdraw(asset, amount, address, network);
      toast.success('Withdrawal request submitted!');
      setAddress('');
      setAmount('');
    } catch (error) {
      const message = error.response?.data?.message || 'Withdrawal failed';
      toast.error(message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="bg-gray-800 rounded-lg p-6">
      <h3 className="text-xl text-white mb-6">Withdraw Crypto</h3>

      <form onSubmit={handleWithdraw} className="space-y-6">
        <div>
          <label className="block text-gray-400 mb-2">Asset</label>
          <select
            value={asset}
            onChange={(e) => setAsset(e.target.value)}
            className="w-full px-4 py-3 bg-gray-700 text-white rounded focus:outline-none"
          >
            {assets.map((a) => (
              <option key={a.symbol} value={a.symbol}>
                {a.symbol} - {a.name}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className="block text-gray-400 mb-2">Network</label>
          <select
            value={network}
            onChange={(e) => setNetwork(e.target.value)}
            className="w-full px-4 py-3 bg-gray-700 text-white rounded focus:outline-none"
          >
            {assets.find((a) => a.symbol === asset)?.networks.map((n) => (
              <option key={n} value={n}>{n}</option>
            ))}
          </select>
        </div>

        <div>
          <label className="block text-gray-400 mb-2">Recipient Address</label>
          <input
            type="text"
            value={address}
            onChange={(e) => setAddress(e.target.value)}
            className="w-full px-4 py-3 bg-gray-700 text-white rounded focus:outline-none"
            placeholder="Enter recipient address"
            required
          />
        </div>

        <div>
          <label className="block text-gray-400 mb-2">Amount</label>
          <input
            type="number"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            className="w-full px-4 py-3 bg-gray-700 text-white rounded focus:outline-none"
            placeholder="0.00"
            min="0"
            step="0.01"
            required
          />
        </div>

        <button
          type="submit"
          disabled={loading}
          className="w-full py-3 bg-blue-600 text-white rounded font-semibold hover:bg-blue-700 disabled:opacity-50"
        >
          {loading ? 'Processing...' : 'Withdraw'}
        </button>
      </form>
    </div>
  );
}

/**
 * History Tab Component
 */
function HistoryTab({ history }) {
  const [tab, setTab] = useState('deposits');

  return (
    <div className="bg-gray-800 rounded-lg p-6">
      <div className="flex space-x-4 mb-6">
        <button
          onClick={() => setTab('deposits')}
          className={`px-4 py-2 rounded ${
            tab === 'deposits' ? 'bg-blue-600 text-white' : 'bg-gray-700 text-gray-300'
          }`}
        >
          Deposits
        </button>
        <button
          onClick={() => setTab('withdrawals')}
          className={`px-4 py-2 rounded ${
            tab === 'withdrawals' ? 'bg-blue-600 text-white' : 'bg-gray-700 text-gray-300'
          }`}
        >
          Withdrawals
        </button>
      </div>

      {tab === 'deposits' && (
        <div className="overflow-x-auto">
          {history.deposits?.length > 0 ? (
            <table className="w-full">
              <thead>
                <tr className="text-gray-400 border-b border-gray-700">
                  <th className="text-left py-3">Time</th>
                  <th className="text-left py-3">Asset</th>
                  <th className="text-left py-3">Amount</th>
                  <th className="text-left py-3">Status</th>
                </tr>
              </thead>
              <tbody>
                {history.deposits.map((d) => (
                  <tr key={d.id} className="text-white border-b border-gray-700">
                    <td className="py-3">{new Date(d.created_at).toLocaleString()}</td>
                    <td className="py-3">{d.asset}</td>
                    <td className="py-3">{d.amount}</td>
                    <td className="py-3">
                      <span className={`px-2 py-1 rounded text-sm ${
                        d.status === 'confirmed' ? 'bg-green-600' :
                        d.status === 'pending' ? 'bg-yellow-600' : 'bg-gray-600'
                      }`}>
                        {d.status}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          ) : (
            <p className="text-gray-400 text-center py-8">No deposit history</p>
          )}
        </div>
      )}

      {tab === 'withdrawals' && (
        <div className="overflow-x-auto">
          {history.withdrawals?.length > 0 ? (
            <table className="w-full">
              <thead>
                <tr className="text-gray-400 border-b border-gray-700">
                  <th className="text-left py-3">Time</th>
                  <th className="text-left py-3">Asset</th>
                  <th className="text-left py-3">Amount</th>
                  <th className="text-left py-3">Fee</th>
                  <th className="text-left py-3">Status</th>
                </tr>
              </thead>
              <tbody>
                {history.withdrawals.map((w) => (
                  <tr key={w.id} className="text-white border-b border-gray-700">
                    <td className="py-3">{new Date(w.created_at).toLocaleString()}</td>
                    <td className="py-3">{w.asset}</td>
                    <td className="py-3">{w.amount}</td>
                    <td className="py-3">{w.fee}</td>
                    <td className="py-3">
                      <span className={`px-2 py-1 rounded text-sm ${
                        w.status === 'confirmed' ? 'bg-green-600' :
                        w.status === 'pending' ? 'bg-yellow-600' : 'bg-gray-600'
                      }`}>
                        {w.status}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          ) : (
            <p className="text-gray-400 text-center py-8">No withdrawal history</p>
          )}
        </div>
      )}
    </div>
  );
}

export interface Balance {
  available: string;
  frozen: string;
}

export interface Asset {
  symbol: string;
  name: string;
  networks: string[];
}

export interface DepositRecord {
  id: string;
  asset: string;
  amount: string;
  status: string;
  created_at: string;
}

export interface WithdrawalRecord {
  id: string;
  asset: string;
  amount: string;
  fee: string;
  status: string;
  created_at: string;
}

export interface WalletHistory {
  deposits: DepositRecord[];
  withdrawals: WithdrawalRecord[];
}

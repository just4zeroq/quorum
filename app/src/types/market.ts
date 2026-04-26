export interface OrderBookEntry {
  price: string;
  quantity: string;
}

export interface OrderBook {
  asks: [string, string][];
  bids: [string, string][];
}

export interface Ticker {
  last_price: string;
  price_change: string;
  high_price: string;
  low_price: string;
  volume: string;
}

export interface KlineParams {
  market_id?: number;
  outcome_id?: number;
  interval?: string;
  limit?: number;
}

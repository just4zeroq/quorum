export interface Order {
  order_id: string;
  market_id: number;
  side: 'YES' | 'NO';
  order_type: 'limit' | 'market';
  price: string;
  quantity: string;
  status: string;
  filled_quantity?: string;
}

export interface CreateOrderParams {
  market_id: number;
  outcome_id: number;
  side: string;
  order_type: string;
  price: string;
  quantity: string;
}

export interface Position {
  position_id: string;
  market_id: number;
  side: string;
  size: string;
  entry_price: string;
  pnl: number;
}

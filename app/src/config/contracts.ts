export const CONTRACTS = {
  // Base Mainnet
  [8453]: {
    marketFactory: '0x0000000000000000000000000000000000000000',
    conditionFactory: '0x0000000000000000000000000000000000000000',
    collateralToken: '0x0000000000000000000000000000000000000000',
  },
  // Base Sepolia Testnet
  [84532]: {
    marketFactory: '0x0000000000000000000000000000000000000000',
    conditionFactory: '0x0000000000000000000000000000000000000000',
    collateralToken: '0x0000000000000000000000000000000000000000',
  },
} as const

export const SUPPORTED_CHAIN_IDS = [8453, 84532] as const

export type SupportedChainId = typeof SUPPORTED_CHAIN_IDS[number]

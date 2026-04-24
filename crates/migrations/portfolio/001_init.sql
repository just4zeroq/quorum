-- Portfolio Service 初始迁移
-- 账户、持仓、结算、账本表

-- 账户表
CREATE TABLE IF NOT EXISTS accounts (
    id              VARCHAR(64) PRIMARY KEY,
    user_id         VARCHAR(64) NOT NULL,
    asset           VARCHAR(16) NOT NULL,
    account_type    VARCHAR(16) NOT NULL DEFAULT 'spot',

    available       DECIMAL(36, 18) NOT NULL DEFAULT 0,
    frozen          DECIMAL(36, 18) NOT NULL DEFAULT 0,
    version         BIGINT NOT NULL DEFAULT 0,

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(user_id, asset, account_type)
);

CREATE INDEX IF NOT EXISTS idx_accounts_user_id ON accounts(user_id);

-- 持仓表
CREATE TABLE IF NOT EXISTS positions (
    id              VARCHAR(64) PRIMARY KEY,
    user_id         VARCHAR(64) NOT NULL,
    market_id       BIGINT NOT NULL,
    outcome_id      BIGINT NOT NULL,
    side            VARCHAR(8) NOT NULL,
    size            DECIMAL(36, 18) NOT NULL DEFAULT 0,
    entry_price     DECIMAL(36, 18) NOT NULL DEFAULT 0,
    version         BIGINT NOT NULL DEFAULT 0,

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(user_id, market_id, outcome_id, side)
);

CREATE INDEX IF NOT EXISTS idx_positions_user_id ON positions(user_id);
CREATE INDEX IF NOT EXISTS idx_positions_market_id ON positions(market_id);

-- 结算表
CREATE TABLE IF NOT EXISTS settlements (
    id              VARCHAR(64) PRIMARY KEY,
    trade_id        VARCHAR(64) NOT NULL,
    market_id       BIGINT NOT NULL,
    user_id         VARCHAR(64) NOT NULL,
    outcome_id      BIGINT NOT NULL,
    side            VARCHAR(8) NOT NULL,
    amount          DECIMAL(36, 18) NOT NULL,
    fee             DECIMAL(36, 18) NOT NULL,
    payout          DECIMAL(36, 18) NOT NULL DEFAULT 0,
    status          VARCHAR(16) NOT NULL DEFAULT 'pending',

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_settlements_trade_id ON settlements(trade_id);
CREATE INDEX IF NOT EXISTS idx_settlements_user_id ON settlements(user_id);
CREATE INDEX IF NOT EXISTS idx_settlements_market_id ON settlements(market_id);

-- 账本流水表 (append-only)
CREATE TABLE IF NOT EXISTS ledger_entries (
    id              VARCHAR(64) PRIMARY KEY,
    user_id         VARCHAR(64) NOT NULL,
    account_id      VARCHAR(64) NOT NULL,
    ledger_type     VARCHAR(16) NOT NULL,
    asset           VARCHAR(16) NOT NULL,
    amount          DECIMAL(36, 18) NOT NULL,
    balance_after   DECIMAL(36, 18) NOT NULL,
    reference_id    VARCHAR(64) NOT NULL,
    reference_type  VARCHAR(16) NOT NULL,

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ledger_user_id ON ledger_entries(user_id);
CREATE INDEX IF NOT EXISTS idx_ledger_account_id ON ledger_entries(account_id);
CREATE INDEX IF NOT EXISTS idx_ledger_reference ON ledger_entries(reference_id, reference_type);
CREATE INDEX IF NOT EXISTS idx_ledger_created_at ON ledger_entries(created_at);

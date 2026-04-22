//! Prediction Market Service SQL Scripts

/// Prediction Market 表 - SQLite
pub const CREATE_PREDICTION_MARKETS_TABLE_SQLITE: &str = r#"
CREATE TABLE IF NOT EXISTS prediction_markets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question TEXT NOT NULL,
    description TEXT,
    category TEXT NOT NULL,
    image_url TEXT,
    start_time INTEGER NOT NULL,
    end_time INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'open',
    resolved_outcome_id INTEGER,
    resolved_at INTEGER,
    total_volume TEXT NOT NULL DEFAULT '0',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
)
"#;

/// Prediction Market 表 - PostgreSQL
pub const CREATE_PREDICTION_MARKETS_TABLE_POSTGRES: &str = r#"
CREATE TABLE IF NOT EXISTS prediction_markets (
    id BIGSERIAL PRIMARY KEY,
    question TEXT NOT NULL,
    description TEXT,
    category TEXT NOT NULL,
    image_url TEXT,
    start_time BIGINT NOT NULL,
    end_time BIGINT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open',
    resolved_outcome_id BIGINT,
    resolved_at BIGINT,
    total_volume TEXT NOT NULL DEFAULT '0',
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
)
"#;

/// Market Outcome 表 - SQLite
pub const CREATE_MARKET_OUTCOMES_TABLE_SQLITE: &str = r#"
CREATE TABLE IF NOT EXISTS market_outcomes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    market_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    image_url TEXT,
    price TEXT NOT NULL DEFAULT '0.5',
    volume TEXT NOT NULL DEFAULT '0',
    probability TEXT NOT NULL DEFAULT '0',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (market_id) REFERENCES prediction_markets(id)
)
"#;

/// Market Outcome 表 - PostgreSQL
pub const CREATE_MARKET_OUTCOMES_TABLE_POSTGRES: &str = r#"
CREATE TABLE IF NOT EXISTS market_outcomes (
    id BIGSERIAL PRIMARY KEY,
    market_id BIGINT NOT NULL REFERENCES prediction_markets(id),
    name TEXT NOT NULL,
    description TEXT,
    image_url TEXT,
    price TEXT NOT NULL DEFAULT '0.5',
    volume TEXT NOT NULL DEFAULT '0',
    probability TEXT NOT NULL DEFAULT '0',
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
)
"#;

/// User Position 表 - SQLite
pub const CREATE_USER_POSITIONS_TABLE_SQLITE: &str = r#"
CREATE TABLE IF NOT EXISTS user_positions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    market_id INTEGER NOT NULL,
    outcome_id INTEGER NOT NULL,
    quantity TEXT NOT NULL,
    avg_price TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (market_id) REFERENCES prediction_markets(id),
    FOREIGN KEY (outcome_id) REFERENCES market_outcomes(id),
    UNIQUE(user_id, market_id, outcome_id)
)
"#;

/// User Position 表 - PostgreSQL
pub const CREATE_USER_POSITIONS_TABLE_POSTGRES: &str = r#"
CREATE TABLE IF NOT EXISTS user_positions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    market_id BIGINT NOT NULL REFERENCES prediction_markets(id),
    outcome_id BIGINT NOT NULL REFERENCES market_outcomes(id),
    quantity TEXT NOT NULL,
    avg_price TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    UNIQUE(user_id, market_id, outcome_id)
)
"#;

/// Market Trade 表 - SQLite
pub const CREATE_MARKET_TRADES_TABLE_SQLITE: &str = r#"
CREATE TABLE IF NOT EXISTS market_trades (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    market_id INTEGER NOT NULL,
    outcome_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    side TEXT NOT NULL,
    price TEXT NOT NULL,
    quantity TEXT NOT NULL,
    amount TEXT NOT NULL,
    fee TEXT NOT NULL DEFAULT '0',
    created_at INTEGER NOT NULL,
    FOREIGN KEY (market_id) REFERENCES prediction_markets(id),
    FOREIGN KEY (outcome_id) REFERENCES market_outcomes(id)
)
"#;

/// Market Trade 表 - PostgreSQL
pub const CREATE_MARKET_TRADES_TABLE_POSTGRES: &str = r#"
CREATE TABLE IF NOT EXISTS market_trades (
    id BIGSERIAL PRIMARY KEY,
    market_id BIGINT NOT NULL REFERENCES prediction_markets(id),
    outcome_id BIGINT NOT NULL REFERENCES market_outcomes(id),
    user_id BIGINT NOT NULL,
    side TEXT NOT NULL,
    price TEXT NOT NULL,
    quantity TEXT NOT NULL,
    amount TEXT NOT NULL,
    fee TEXT NOT NULL DEFAULT '0',
    created_at BIGINT NOT NULL
)
"#;

/// Market Resolution 表 - SQLite
pub const CREATE_MARKET_RESOLUTIONS_TABLE_SQLITE: &str = r#"
CREATE TABLE IF NOT EXISTS market_resolutions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    market_id INTEGER NOT NULL UNIQUE,
    outcome_id INTEGER NOT NULL,
    total_payout TEXT NOT NULL,
    winning_quantity TEXT NOT NULL,
    payout_ratio TEXT NOT NULL,
    resolved_at INTEGER NOT NULL,
    FOREIGN KEY (market_id) REFERENCES prediction_markets(id),
    FOREIGN KEY (outcome_id) REFERENCES market_outcomes(id)
)
"#;

/// Market Resolution 表 - PostgreSQL
pub const CREATE_MARKET_RESOLUTIONS_TABLE_POSTGRES: &str = r#"
CREATE TABLE IF NOT EXISTS market_resolutions (
    id BIGSERIAL PRIMARY KEY,
    market_id BIGINT NOT NULL UNIQUE REFERENCES prediction_markets(id),
    outcome_id BIGINT NOT NULL REFERENCES market_outcomes(id),
    total_payout TEXT NOT NULL,
    winning_quantity TEXT NOT NULL,
    payout_ratio TEXT NOT NULL,
    resolved_at BIGINT NOT NULL
)
"#;

/// Prediction Market 索引 - SQLite
pub const CREATE_PREDICTION_MARKET_INDEXES_SQLITE: &[&str] = &[
    "CREATE INDEX IF NOT EXISTS idx_markets_category ON prediction_markets(category)",
    "CREATE INDEX IF NOT EXISTS idx_markets_status ON prediction_markets(status)",
    "CREATE INDEX IF NOT EXISTS idx_markets_end_time ON prediction_markets(end_time)",
    "CREATE INDEX IF NOT EXISTS idx_outcomes_market_id ON market_outcomes(market_id)",
    "CREATE INDEX IF NOT EXISTS idx_positions_user_id ON user_positions(user_id)",
    "CREATE INDEX IF NOT EXISTS idx_positions_market_id ON user_positions(market_id)",
    "CREATE INDEX IF NOT EXISTS idx_trades_market_id ON market_trades(market_id)",
    "CREATE INDEX IF NOT EXISTS idx_trades_user_id ON market_trades(user_id)",
    "CREATE INDEX IF NOT EXISTS idx_trades_created_at ON market_trades(created_at)",
];

/// Prediction Market 索引 - PostgreSQL
pub const CREATE_PREDICTION_MARKET_INDEXES_POSTGRES: &[&str] = &[
    "CREATE INDEX IF NOT EXISTS idx_markets_category ON prediction_markets(category)",
    "CREATE INDEX IF NOT EXISTS idx_markets_status ON prediction_markets(status)",
    "CREATE INDEX IF NOT EXISTS idx_markets_end_time ON prediction_markets(end_time)",
    "CREATE INDEX IF NOT EXISTS idx_outcomes_market_id ON market_outcomes(market_id)",
    "CREATE INDEX IF NOT EXISTS idx_positions_user_id ON user_positions(user_id)",
    "CREATE INDEX IF NOT EXISTS idx_positions_market_id ON user_positions(market_id)",
    "CREATE INDEX IF NOT EXISTS idx_trades_market_id ON market_trades(market_id)",
    "CREATE INDEX IF NOT EXISTS idx_trades_user_id ON market_trades(user_id)",
    "CREATE INDEX IF NOT EXISTS idx_trades_created_at ON market_trades(created_at)",
];
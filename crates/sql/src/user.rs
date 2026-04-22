//! User Service SQL Scripts

/// User 表 - SQLite
pub const CREATE_USERS_TABLE_SQLITE: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT,
    phone TEXT,
    kyc_status TEXT NOT NULL DEFAULT 'none',
    kyc_level INTEGER NOT NULL DEFAULT 0,
    kyc_submitted_at TEXT,
    kyc_verified_at TEXT,
    two_factor_enabled INTEGER NOT NULL DEFAULT 0,
    two_factor_secret TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    status_reason TEXT,
    frozen_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_login_at TEXT
)
"#;

/// User 表 - PostgreSQL
pub const CREATE_USERS_TABLE_POSTGRES: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255),
    phone VARCHAR(50),
    kyc_status VARCHAR(20) NOT NULL DEFAULT 'none',
    kyc_level INTEGER NOT NULL DEFAULT 0,
    kyc_submitted_at TIMESTAMP,
    kyc_verified_at TIMESTAMP,
    two_factor_enabled BOOLEAN NOT NULL DEFAULT false,
    two_factor_secret VARCHAR(255),
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    status_reason TEXT,
    frozen_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMP
)
"#;

/// Wallet Address 表 - SQLite
pub const CREATE_WALLET_ADDRESSES_TABLE_SQLITE: &str = r#"
CREATE TABLE IF NOT EXISTS wallet_addresses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    wallet_address TEXT NOT NULL,
    wallet_type TEXT NOT NULL,
    chain_type TEXT NOT NULL,
    is_primary INTEGER NOT NULL DEFAULT 0,
    verified_at TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
)
"#;

/// Wallet Address 表 - PostgreSQL
pub const CREATE_WALLET_ADDRESSES_TABLE_POSTGRES: &str = r#"
CREATE TABLE IF NOT EXISTS wallet_addresses (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    wallet_address VARCHAR(100) NOT NULL,
    wallet_type VARCHAR(20) NOT NULL,
    chain_type VARCHAR(20) NOT NULL,
    is_primary BOOLEAN NOT NULL DEFAULT false,
    verified_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
)
"#;

/// User Session 表 - SQLite
pub const CREATE_USER_SESSIONS_TABLE_SQLITE: &str = r#"
CREATE TABLE IF NOT EXISTS user_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    token TEXT NOT NULL UNIQUE,
    refresh_token TEXT,
    ip_address TEXT,
    user_agent TEXT,
    device_id TEXT,
    login_method TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    last_active_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
)
"#;

/// User Session 表 - PostgreSQL
pub const CREATE_USER_SESSIONS_TABLE_POSTGRES: &str = r#"
CREATE TABLE IF NOT EXISTS user_sessions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    token TEXT NOT NULL UNIQUE,
    refresh_token TEXT,
    ip_address VARCHAR(50),
    user_agent TEXT,
    device_id VARCHAR(100),
    login_method VARCHAR(20) NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    last_active_at TIMESTAMP NOT NULL DEFAULT NOW()
)
"#;

/// User 索引 - SQLite
pub const CREATE_USER_INDEXES_SQLITE: &[&str] = &[
    "CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)",
    "CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)",
    "CREATE INDEX IF NOT EXISTS idx_users_status ON users(status)",
];

/// User 索引 - PostgreSQL
pub const CREATE_USER_INDEXES_POSTGRES: &[&str] = &[
    "CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)",
    "CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)",
    "CREATE INDEX IF NOT EXISTS idx_users_status ON users(status)",
];
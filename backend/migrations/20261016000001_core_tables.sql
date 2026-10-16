-- Core identity/brand tables (port of pp_admin, pp_browser_log, pp_permission, pp_brands, pp_env)
-- NOTE: no admin row is ever seeded here; the initial admin is created via /api/setup.

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    user_id VARCHAR(40) NOT NULL UNIQUE,
    full_name TEXT NOT NULL DEFAULT '',
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    temp_password_hash TEXT,
    reset_limit SMALLINT NOT NULL DEFAULT 3,
    status VARCHAR(10) NOT NULL DEFAULT 'active' CHECK (status IN ('active','suspend')),
    role VARCHAR(10) NOT NULL DEFAULT 'staff' CHECK (role IN ('admin','staff')),
    totp_status VARCHAR(10) NOT NULL DEFAULT 'disable' CHECK (totp_status IN ('enable','disable')),
    totp_secret VARCHAR(64) NOT NULL DEFAULT '--',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE sessions (
    id BIGSERIAL PRIMARY KEY,
    user_id VARCHAR(40) NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    token_hash CHAR(64) NOT NULL UNIQUE,
    browser TEXT NOT NULL DEFAULT '--',
    device TEXT NOT NULL DEFAULT '--',
    ip VARCHAR(64) NOT NULL DEFAULT '--',
    status VARCHAR(10) NOT NULL DEFAULT 'active' CHECK (status IN ('active','expired')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_sessions_user ON sessions(user_id);

CREATE TABLE permissions (
    id BIGSERIAL PRIMARY KEY,
    brand_id VARCHAR(40) NOT NULL,
    user_id VARCHAR(40) NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    permission JSONB NOT NULL,
    status VARCHAR(10) NOT NULL DEFAULT 'active' CHECK (status IN ('active','suspend')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (brand_id, user_id)
);

CREATE TABLE brands (
    id BIGSERIAL PRIMARY KEY,
    brand_id VARCHAR(40) NOT NULL UNIQUE,
    favicon TEXT,
    logo TEXT,
    identify_name VARCHAR(50) NOT NULL DEFAULT 'Default',
    name TEXT,
    support_email_address TEXT,
    support_phone_number TEXT,
    support_website TEXT,
    whatsapp_number TEXT,
    telegram TEXT,
    facebook_messenger TEXT,
    facebook_page TEXT,
    theme VARCHAR(120) NOT NULL DEFAULT 'default',
    street_address TEXT,
    city_town TEXT,
    postal_code TEXT,
    country TEXT,
    timezone VARCHAR(150) NOT NULL DEFAULT 'Asia/Dhaka',
    language VARCHAR(150) NOT NULL DEFAULT 'en',
    currency_code VARCHAR(10) NOT NULL DEFAULT 'BDT',
    payment_tolerance NUMERIC(20,8) NOT NULL DEFAULT 0,
    -- notification settings (per-brand, PRD 4.8)
    notify_email_enabled BOOLEAN NOT NULL DEFAULT false,
    notify_telegram_enabled BOOLEAN NOT NULL DEFAULT false,
    notify_telegram_chat_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- pp_env port: per-brand or global ('both') key/value settings
CREATE TABLE settings (
    id BIGSERIAL PRIMARY KEY,
    brand_id VARCHAR(40) NOT NULL DEFAULT 'both',
    option_name VARCHAR(100) NOT NULL,
    value TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (brand_id, option_name)
);

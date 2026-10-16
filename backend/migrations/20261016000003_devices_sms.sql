-- Devices, SMS data, balance verification, rules, webhooks, domains, faqs
-- (ports of pp_device, pp_sms_data, pp_balance_verification, pp_webhook_log,
--  pp_domain, pp_faq; plus new table-driven sms_rules + sender_whitelist)

CREATE TABLE devices (
    id BIGSERIAL PRIMARY KEY,
    user_id VARCHAR(40) NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    device_id VARCHAR(40) NOT NULL UNIQUE,
    -- PHP parity: the otp column holds the pairing OTP first, then the
    -- long-lived token replaces it on login (status flips processing -> used).
    otp VARCHAR(40) NOT NULL UNIQUE,
    name TEXT,
    model TEXT,
    android_level TEXT,
    app_version TEXT,
    status VARCHAR(10) NOT NULL DEFAULT 'processing' CHECK (status IN ('processing','used')),
    last_sync TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_devices_user ON devices(user_id, status);

CREATE TABLE sms_data (
    id BIGSERIAL PRIMARY KEY,
    source VARCHAR(6) NOT NULL DEFAULT 'app' CHECK (source IN ('app','web')),
    device_id VARCHAR(40) NOT NULL,
    sender VARCHAR(50) NOT NULL DEFAULT '--',
    sender_key VARCHAR(50) NOT NULL DEFAULT '--',
    simslot VARCHAR(10) NOT NULL DEFAULT '--',
    number VARCHAR(30) NOT NULL DEFAULT '--',
    amount NUMERIC(20,8) NOT NULL DEFAULT 0,
    currency VARCHAR(10) NOT NULL DEFAULT 'BDT',
    trx_id VARCHAR(100) NOT NULL DEFAULT '--',
    balance NUMERIC(20,8),
    message TEXT,
    reason TEXT,
    type VARCHAR(10) NOT NULL DEFAULT 'Personal' CHECK (type IN ('Personal','Agent','Merchant')),
    entry_type VARCHAR(10) NOT NULL DEFAULT 'automatic' CHECK (entry_type IN ('manual','automatic')),
    edit_status VARCHAR(10) NOT NULL DEFAULT 'pending' CHECK (edit_status IN ('done','pending')),
    status VARCHAR(20) NOT NULL DEFAULT 'approved'
        CHECK (status IN ('approved','awaiting-review','used','error')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_sms_match ON sms_data(sender_key, type, trx_id, status);
CREATE INDEX idx_sms_device ON sms_data(device_id, status);

CREATE TABLE balance_verification (
    id BIGSERIAL PRIMARY KEY,
    device_id VARCHAR(40) NOT NULL,
    sender_key VARCHAR(50) NOT NULL,
    type VARCHAR(10) NOT NULL CHECK (type IN ('Personal','Agent','Merchant')),
    current_balance NUMERIC(20,8) NOT NULL DEFAULT 0,
    simslot VARCHAR(6) NOT NULL DEFAULT 'Any' CHECK (simslot IN ('Sim1','Sim2','Any')),
    status VARCHAR(10) NOT NULL DEFAULT 'active' CHECK (status IN ('active','inactive')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (device_id, sender_key, type)
);

-- Table-driven SMS parse rules (PRD 6): regex + field map, priority DESC wins.
-- field_map: JSON array mapping capture-group index -> field name
-- (amount, sender, ref, fee, balance, trxid, datetime, receiver).
CREATE TABLE sms_rules (
    id BIGSERIAL PRIMARY KEY,
    provider_key VARCHAR(30) NOT NULL,
    mfs_type VARCHAR(10) NOT NULL CHECK (mfs_type IN ('Personal','Agent','Merchant')),
    priority INT NOT NULL DEFAULT 100,
    pattern TEXT NOT NULL,
    field_map JSONB NOT NULL,
    active BOOLEAN NOT NULL DEFAULT true
);
CREATE INDEX idx_sms_rules_provider ON sms_rules(provider_key, active);

-- Sender whitelist (port of senderWhitelist() provider table).
CREATE TABLE sender_whitelist (
    id BIGSERIAL PRIMARY KEY,
    sender VARCHAR(30) NOT NULL UNIQUE,   -- lowercased SMS originating address
    provider_key VARCHAR(30) NOT NULL,
    name TEXT NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'BDT',
    balance_verify BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE webhook_log (
    id BIGSERIAL PRIMARY KEY,
    ref VARCHAR(30) NOT NULL,
    brand_id VARCHAR(40) NOT NULL,
    payload JSONB NOT NULL,
    url TEXT NOT NULL,
    attempts INT NOT NULL DEFAULT 0,
    response_body TEXT,
    http_code INT,
    status VARCHAR(10) NOT NULL DEFAULT 'pending' CHECK (status IN ('completed','pending','canceled')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_webhook_pending ON webhook_log(status) WHERE status = 'pending';

CREATE TABLE domains (
    id BIGSERIAL PRIMARY KEY,
    domain VARCHAR(100) NOT NULL UNIQUE,
    status VARCHAR(10) NOT NULL DEFAULT 'active' CHECK (status IN ('active','inactive')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE faqs (
    id BIGSERIAL PRIMARY KEY,
    brand_id VARCHAR(40) NOT NULL REFERENCES brands(brand_id) ON DELETE CASCADE,
    question TEXT NOT NULL,
    answer TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

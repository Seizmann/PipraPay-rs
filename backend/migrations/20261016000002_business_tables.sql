-- Business tables: gateways, customers, invoices, payment links, transactions
-- (ports of pp_gateways, pp_gateways_parameter, pp_customer, pp_invoice,
--  pp_invoice_items, pp_payment_link, pp_payment_link_field, pp_transaction)

CREATE TABLE gateways (
    id BIGSERIAL PRIMARY KEY,
    gateway_id VARCHAR(40) NOT NULL UNIQUE,
    brand_id VARCHAR(40) NOT NULL REFERENCES brands(brand_id) ON DELETE CASCADE,
    slug VARCHAR(40) NOT NULL,              -- e.g. 'bkash-personal'
    name TEXT,
    display TEXT,
    logo TEXT,
    currency VARCHAR(10) NOT NULL DEFAULT 'BDT',
    min_allow NUMERIC(20,8) NOT NULL DEFAULT 0,
    max_allow NUMERIC(20,8) NOT NULL DEFAULT 0,
    fixed_discount NUMERIC(20,8) NOT NULL DEFAULT 0,
    percentage_discount NUMERIC(20,8) NOT NULL DEFAULT 0,
    fixed_charge NUMERIC(20,8) NOT NULL DEFAULT 0,
    percentage_charge NUMERIC(20,8) NOT NULL DEFAULT 0,
    receive_number VARCHAR(30) NOT NULL DEFAULT '--',   -- send-money number shown to customers
    instructions TEXT,                                   -- payment-instructions list (JSON array of strings)
    qr_image TEXT,                                       -- data URI or path (PHP 'qr_code' field)
    primary_color TEXT,
    text_color TEXT,
    btn_color TEXT,
    btn_text_color TEXT,
    tab VARCHAR(10) NOT NULL DEFAULT 'mfs' CHECK (tab IN ('mfs','bank','global')),
    status VARCHAR(10) NOT NULL DEFAULT 'active' CHECK (status IN ('active','inactive')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_gateways_brand ON gateways(brand_id, status);

CREATE TABLE gateway_parameters (
    id BIGSERIAL PRIMARY KEY,
    gateway_id VARCHAR(40) NOT NULL REFERENCES gateways(gateway_id) ON DELETE CASCADE,
    option_name VARCHAR(100) NOT NULL,
    value TEXT NOT NULL DEFAULT ''
);

CREATE TABLE customers (
    id BIGSERIAL PRIMARY KEY,
    ref VARCHAR(30) NOT NULL UNIQUE,
    brand_id VARCHAR(40) NOT NULL REFERENCES brands(brand_id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    email VARCHAR(100) NOT NULL DEFAULT '--',
    mobile VARCHAR(20) NOT NULL DEFAULT '--',
    status VARCHAR(10) NOT NULL DEFAULT 'active' CHECK (status IN ('active','suspend')),
    suspend_reason TEXT,
    inserted_via VARCHAR(10) NOT NULL DEFAULT 'manual' CHECK (inserted_via IN ('manual','checkout')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_customers_brand ON customers(brand_id);

CREATE TABLE invoices (
    id BIGSERIAL PRIMARY KEY,
    ref VARCHAR(30) NOT NULL UNIQUE,
    brand_id VARCHAR(40) NOT NULL REFERENCES brands(brand_id) ON DELETE CASCADE,
    customer_ref VARCHAR(30) REFERENCES customers(ref) ON DELETE SET NULL,
    customer_info JSONB NOT NULL DEFAULT '{}',  -- PHP parity: denormalized snapshot {name,email,mobile}
    gateway_id VARCHAR(40) NOT NULL DEFAULT '--',
    currency VARCHAR(10) NOT NULL DEFAULT 'BDT',
    due_date DATE,
    shipping NUMERIC(20,8) NOT NULL DEFAULT 0,
    status VARCHAR(10) NOT NULL DEFAULT 'unpaid' CHECK (status IN ('paid','unpaid','refunded','canceled')),
    note TEXT,
    private_note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_invoices_brand ON invoices(brand_id, status);

CREATE TABLE invoice_items (
    id BIGSERIAL PRIMARY KEY,
    invoice_ref VARCHAR(30) NOT NULL REFERENCES invoices(ref) ON DELETE CASCADE,
    description TEXT,
    amount NUMERIC(20,8) NOT NULL DEFAULT 0,
    quantity INT NOT NULL DEFAULT 1,
    discount NUMERIC(20,8) NOT NULL DEFAULT 0,
    vat NUMERIC(20,8) NOT NULL DEFAULT 0
);

CREATE TABLE payment_links (
    id BIGSERIAL PRIMARY KEY,
    ref VARCHAR(30) NOT NULL UNIQUE,
    brand_id VARCHAR(40) NOT NULL REFERENCES brands(brand_id) ON DELETE CASCADE,
    product_info JSONB NOT NULL DEFAULT '{}',   -- {title, description}
    amount NUMERIC(20,8) NOT NULL DEFAULT 0,    -- 0 => pay-what-you-want
    quantity INT NOT NULL DEFAULT 0,            -- 0 => unlimited stock
    currency VARCHAR(10) NOT NULL DEFAULT 'BDT',
    expired_date DATE,
    status VARCHAR(10) NOT NULL DEFAULT 'active' CHECK (status IN ('active','inactive')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE payment_link_fields (
    id BIGSERIAL PRIMARY KEY,
    payment_link_ref VARCHAR(30) NOT NULL REFERENCES payment_links(ref) ON DELETE CASCADE,
    form_type TEXT NOT NULL,   -- text | textarea | select | checkbox | radio | file
    field_name TEXT NOT NULL,
    value TEXT NOT NULL DEFAULT '',
    required BOOLEAN NOT NULL DEFAULT true
);

-- Port of pp_transaction. source mirrors PHP enum values.
CREATE TABLE transactions (
    id BIGSERIAL PRIMARY KEY,
    ref VARCHAR(30) NOT NULL UNIQUE,
    brand_id VARCHAR(40) NOT NULL REFERENCES brands(brand_id) ON DELETE CASCADE,
    source VARCHAR(30) NOT NULL DEFAULT 'api'
        CHECK (source IN ('invoice','payment-link','payment-link-default','api')),
    customer_ref VARCHAR(30) REFERENCES customers(ref) ON DELETE SET NULL,
    customer_info JSONB NOT NULL DEFAULT '{}',
    amount NUMERIC(20,8) NOT NULL DEFAULT 0,
    processing_fee NUMERIC(20,8) NOT NULL DEFAULT 0,
    discount_amount NUMERIC(20,8) NOT NULL DEFAULT 0,
    local_net_amount NUMERIC(20,8) NOT NULL DEFAULT 0,
    currency VARCHAR(10) NOT NULL DEFAULT 'BDT',
    local_currency VARCHAR(10) NOT NULL DEFAULT 'BDT',
    sender VARCHAR(50) NOT NULL DEFAULT '--',
    trx_id VARCHAR(100) NOT NULL DEFAULT '--',
    trx_slip TEXT,
    gateway_id VARCHAR(50) NOT NULL DEFAULT '--',
    sender_key VARCHAR(50) NOT NULL DEFAULT '--',   -- provider key, e.g. 'bkash'
    sender_type VARCHAR(11) NOT NULL DEFAULT 'Personal',
    source_info JSONB,
    metadata JSONB,
    status VARCHAR(12) NOT NULL DEFAULT 'initiated'
        CHECK (status IN ('completed','pending','refunded','initiated','canceled')),
    return_url TEXT,
    webhook_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_transactions_brand ON transactions(brand_id, status);
CREATE UNIQUE INDEX idx_transactions_trx ON transactions(trx_id) WHERE trx_id <> '--';

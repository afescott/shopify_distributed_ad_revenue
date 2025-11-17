-- Enable UUID generation
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- merchants: one row per Shopify store; shop identity and metadata
CREATE TABLE merchants (
	id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	shop_domain         TEXT NOT NULL UNIQUE,
	shop_name           TEXT,
	shop_currency       TEXT,
	timezone            TEXT,
	created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	deleted_at          TIMESTAMPTZ
);

-- shopify_installs: installation records and scopes per merchant
CREATE TABLE shopify_installs (
	id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	merchant_id         UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
	access_scopes       TEXT NOT NULL,
	installed_at        TIMESTAMPTZ NOT NULL,
	uninstalled_at      TIMESTAMPTZ,
	status              TEXT NOT NULL, -- active|uninstalled
	created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


-- app_settings: per-merchant configuration for calculations and sync behavior
CREATE TABLE app_settings (
	id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	merchant_id         UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
	revenue_basis       TEXT NOT NULL DEFAULT 'subtotal', -- subtotal|total
	include_taxes       BOOLEAN NOT NULL DEFAULT FALSE,
	include_shipping    BOOLEAN NOT NULL DEFAULT FALSE,
	default_currency    TEXT,
	multi_currency_mode TEXT NOT NULL DEFAULT 'warn', -- warn|convert
	sync_lookback_days  INTEGER NOT NULL DEFAULT 120,
	auto_refresh_cron   TEXT,
	created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);



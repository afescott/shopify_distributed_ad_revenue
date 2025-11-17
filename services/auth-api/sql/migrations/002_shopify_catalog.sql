-- 0002_shopify_catalog.sql
CREATE TABLE products (
	id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	merchant_id             UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
	shopify_product_id      BIGINT NOT NULL,
	title                   TEXT,
	product_type            TEXT,
	status                  TEXT,
	created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	deleted_at              TIMESTAMPTZ
);
CREATE UNIQUE INDEX ux_products_shopify ON products(merchant_id, shopify_product_id);
CREATE INDEX idx_products_type ON products(merchant_id, product_type);

CREATE TABLE variants (
	id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	merchant_id             UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
	shopify_variant_id      BIGINT NOT NULL,
	shopify_product_id      BIGINT NOT NULL,
	sku                     TEXT,
	title                   TEXT,
	barcode                 TEXT,
	weight                  NUMERIC(14,4),
	weight_unit             TEXT,
	created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX ux_variants_shopify ON variants(merchant_id, shopify_variant_id);
CREATE INDEX idx_variants_product ON variants(merchant_id, shopify_product_id);

CREATE TABLE inventory_items (
	id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	merchant_id                 UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
	shopify_inventory_item_id   BIGINT NOT NULL,
	shopify_variant_id          BIGINT,
	created_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
/* CREATE UNIQUE INDEX ux_inventory_items_shopify ON inventory_items(merchant_id, shopify_inventory_item_id);
CREATE INDEX idx_inventory_items_variant ON inventory_items(merchant_id, shopify_variant_id);

-- Historical COGS tracking: pick latest <= sale time when computing metrics
CREATE TABLE inventory_cost_history (
	id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	merchant_id                 UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
	shopify_inventory_item_id   BIGINT NOT NULL,
	cost                        NUMERIC(14,4) NOT NULL,
	currency                    TEXT NOT NULL,
	effective_at                TIMESTAMPTZ NOT NULL,
	source                      TEXT NOT NULL DEFAULT 'shopify', -- shopify|manual
	created_at                  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX ux_inventory_cost_hist_key ON inventory_cost_history(merchant_id, shopify_inventory_item_id, effective_at);
CREATE INDEX idx_inventory_cost_hist_lookup ON inventory_cost_history(merchant_id, shopify_inventory_item_id, effective_at DESC); */

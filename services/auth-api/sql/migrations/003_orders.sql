-- 0003_orders.sql
CREATE TABLE orders (
	id                              BIGSERIAL PRIMARY KEY,
	merchant_id                     UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
	shopify_order_id                BIGINT NOT NULL,
	name                            TEXT,
	processed_at                    TIMESTAMPTZ,
	currency                        TEXT,
	subtotal_price                  NUMERIC(14,4),
	total_price                     NUMERIC(14,4),
	total_discounts                 NUMERIC(14,4),
	total_shipping_price_set_amount NUMERIC(14,4),
	total_tax                       NUMERIC(14,4),
	financial_status                TEXT,
	cancelled_at                    TIMESTAMPTZ,
	created_at                      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at                      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

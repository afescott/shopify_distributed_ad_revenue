-- 004_users.sql
-- User authentication table (JWT-based auth, no session tracking)

-- users: individual users who can access the dashboard
CREATE TABLE users (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id         UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    email               TEXT NOT NULL,
    password_hash       TEXT,  -- SHA-256 hash, NULL for OAuth-only users
    display_name        TEXT,
    role                TEXT NOT NULL DEFAULT 'viewer',  -- admin|manager|viewer
    shopify_user_id     BIGINT,  -- if authenticated via Shopify
    last_login_at       TIMESTAMPTZ,
    is_active           BOOLEAN NOT NULL DEFAULT TRUE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(merchant_id, email)
);

-- Create indexes for faster lookups
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_merchant_id ON users(merchant_id);
CREATE INDEX idx_users_shopify_user_id ON users(shopify_user_id) WHERE shopify_user_id IS NOT NULL;

-- Add a function to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply the trigger to users table
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Add comment documentation
COMMENT ON TABLE users IS 'Individual users who can access the merchant dashboard. Authentication via JWT tokens.';
COMMENT ON COLUMN users.role IS 'User role: admin (full control), manager (edit products/orders), viewer (read-only)';
COMMENT ON COLUMN users.is_active IS 'Whether user can log in. Set to false for offboarding instead of deleting.';
COMMENT ON COLUMN users.password_hash IS 'SHA-256 password hash. NULL for OAuth-only users.';

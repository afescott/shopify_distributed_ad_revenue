# Shopify Sync

A standalone binary that syncs products and orders from Shopify API to the database via the auth API.

## Usage

```bash
# Sync products
cargo run -- \
  --store-name "your-store" \
  --access-token "your-token" \
  --merchant-id "merchant-uuid" \
  --products \
  --limit 100

# Sync orders
cargo run -- \
  --store-name "your-store" \
  --access-token "your-token" \
  --merchant-id "merchant-uuid" \
  --orders \
  --limit 100

# Sync both
cargo run -- \
  --store-name "your-store" \
  --access-token "your-token" \
  --merchant-id "merchant-uuid" \
  --products \
  --orders
```

## Environment Variables

All arguments can also be provided via environment variables:

- `SHOPIFY_STORE_NAME`
- `SHOPIFY_ACCESS_TOKEN`
- `SHOPIFY_API_VERSION` (default: "2025-10")
- `AUTH_API_URL` (default: "http://localhost:8080")
- `MERCHANT_ID`

## How It Works

1. Fetches data from Shopify Admin API using the `ShopifyClient`
2. Converts Shopify data to the format expected by the auth API
3. Calls the auth API endpoints (`/api/v1/products` and `/api/v1/orders`) to insert data
4. Handles duplicates gracefully (skips if already exists)

Both binaries share the same database schema (SQL migrations in `../auth_module/sql/migrations/`).



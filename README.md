# Shopify Margin Cost Dashboard

A distributed system for managing Shopify store data, calculating profit margins, and syncing product/order information.

## Architecture

This is a monorepo containing multiple microservices and shared libraries:

```
shopify-margin-cost-dashboard/
├── services/
│   ├── auth-api/          # Authentication & Authorization API
│   ├── shopify-consumer/  # Shopify data sync service (HTTP API + scheduler)
│   └── profit-engine/    # Profit calculation engine
├── libs/
│   └── lib-shopify/      # Shared Shopify & Kafka utilities
└── docker-compose.yml    # Local development setup
```

## Services

### Auth API (`services/auth-api`)
- **Port**: 8080
- **Purpose**: Authentication, authorization, user management, and data API
- **Database**: PostgreSQL
- **Features**:
  - JWT-based authentication
  - User management
  - Product & order management
  - Inventory tracking

### Shopify Consumer (`services/shopify-consumer`)
- **Port**: 8081
- **Purpose**: Syncs data from Shopify to the auth API
- **Features**:
  - HTTP API for on-demand syncs
  - Cron-based scheduled syncs
  - Kafka integration for event streaming
  - Product & order synchronization

### Profit Engine (`services/profit-engine`)
- **Purpose**: Calculates profit margins and cost analysis
- **Status**: In development

### Lib Shopify (`libs/lib-shopify`)
- **Purpose**: Shared library for Shopify API client and Kafka producer
- **Used by**: All services that interact with Shopify or Kafka

## Getting Started

### Prerequisites

- Rust (latest stable)
- Docker & Docker Compose
- PostgreSQL (or use Docker)
- Kafka (or use Docker)

### Local Development

1. **Start infrastructure services:**
   ```bash
   docker-compose up -d postgres kafka zookeeper
   ```

2. **Set up database:**
   ```bash
   cd services/auth-api
   ./setup_db.sh
   ```

3. **Run services:**

   **Auth API:**
   ```bash
   cd services/auth-api
   cargo run
   ```

   **Shopify Consumer:**
   ```bash
   cd services/shopify-consumer
   cargo run -- \
     --store-name YOUR_STORE \
     --access-token YOUR_TOKEN \
     --merchant-id YOUR_UUID \
     --products-sync-cron "0 */6 * * *" \
     --orders-sync-cron "0 */1 * * *"
   ```

### Environment Variables

#### Auth API
- `DATABASE_URL` - PostgreSQL connection string
- `JWT_PRIVATE_KEY` - JWT private key (optional, auto-generated if not provided)
- `JWT_PUBLIC_KEY` - JWT public key (optional)
- `JWT_EXPIRATION_HOURS` - JWT token expiration (default: 24)

#### Shopify Consumer
- `SHOPIFY_STORE_NAME` - Your Shopify store name
- `SHOPIFY_ACCESS_TOKEN` - Shopify Admin API access token
- `SHOPIFY_API_VERSION` - API version (default: 2025-10)
- `MERCHANT_ID` - UUID of your merchant account
- `AUTH_API_URL` - Auth API base URL (default: http://localhost:8080)
- `KAFKA_BROKERS` - Kafka broker addresses (comma-separated)
- `HTTP_PORT` - HTTP server port (default: 8081)
- `PRODUCTS_SYNC_CRON` - Cron expression for product syncs
- `ORDERS_SYNC_CRON` - Cron expression for order syncs

## API Endpoints

### Auth API (Port 8080)
- `POST /api/v1/login` - User login
- `GET /api/v1/products` - List products
- `POST /api/v1/products` - Create product
- `GET /api/v1/orders` - List orders
- `POST /api/v1/orders` - Create order

### Shopify Consumer (Port 8081)
- `GET /health` - Health check
- `POST /api/v1/sync/products` - Trigger product sync
- `POST /api/v1/sync/orders` - Trigger order sync

## Development

### Building

```bash
# Build all services
cargo build --workspace

# Build specific service
cd services/auth-api && cargo build
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for specific service
cd services/auth-api && cargo test
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Lint
cargo clippy --all-targets --all-features -- -D warnings
```

## Docker Compose

Full stack with all services:

```bash
docker-compose up
```

This will start:
- PostgreSQL (port 5432)
- Kafka (port 9092)
- Zookeeper (port 2181)
- Auth API (port 8080)
- Shopify Consumer (port 8081)

## CI/CD

GitHub Actions workflows are configured for:
- Automated testing on push/PR
- Code formatting checks
- Clippy linting
- Build verification

## License

[Your License Here]

